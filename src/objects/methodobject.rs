use std::mem::ManuallyDrop;

use trc::Trc;

use crate::{interpreter::VM, is_type_exact, parser::Position, unwrap_fast};

use super::{
    boolobject, create_object_from_type, exceptionobject::methodnotdefinedexc_from_str,
    finalize_type, finalize_type_dict, listobject, stringobject, MethodType, MethodValue, Object,
    ObjectInternals, RawObject, TypeObject,
};

pub fn method_from<'a>(vm: Trc<VM<'a>>, fun: Object<'a>, instance: Object<'a>) -> Object<'a> {
    let mut tp =
        create_object_from_type(unwrap_fast!(vm.types.methodtp.as_ref()).clone(), vm, None);
    tp.internals = ObjectInternals {
        fn_wrapper: ManuallyDrop::new(super::FnWrapper { fun, instance }),
    };
    tp
}

fn method_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn method_repr(selfv: Object<'_>) -> MethodType<'_> {
    if unsafe { &selfv.internals.fn_wrapper }.fun.tp.repr.is_none() {
        let exc = methodnotdefinedexc_from_str(
            selfv.vm.clone(),
            &format!(
                "Method 'repr' is not defined for '{}' type",
                unsafe { &selfv.internals.fn_wrapper }.fun.tp.typename
            ),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let repr_result =
        RawObject::object_repr_safe(unsafe { &selfv.internals.fn_wrapper }.fun.clone());
    if repr_result.is_error() {
        return MethodValue::Error(repr_result.unwrap_err());
    }

    let inst_tp_result =
        RawObject::object_repr_safe(unsafe { &selfv.internals.fn_wrapper }.instance.clone());
    if inst_tp_result.is_error() {
        return MethodValue::Error(inst_tp_result.unwrap_err());
    }

    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!(
            "<method '{}' of '{}' @ 0x{:x}>",
            repr_result.unwrap(),
            inst_tp_result.unwrap(),
            Trc::as_ptr(&selfv) as usize
        ),
    ))
}
fn method_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        unsafe { &selfv.internals.fun } == unsafe { &other.internals.fun },
    ))
}

fn method_call<'a>(selfv: Object<'a>, args: Object<'a>) -> MethodType<'a> {
    let mdata = unsafe { &selfv.internals.fn_wrapper };
    if mdata.fun.tp.call.is_none() {
        let exc = methodnotdefinedexc_from_str(
            selfv.vm.clone(),
            &format!(
                "Method 'call' is not defined for '{}' type",
                mdata.fun.tp.typename
            ),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }
    let mut args = unsafe { &args.internals.arr }.clone();

    args.insert(0, mdata.instance.clone());

    mdata.fun.tp.call.unwrap()(
        mdata.fun.clone(),
        listobject::list_from(selfv.vm.clone(), args.to_vec()),
    )
}

pub fn init(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("fn"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(method_new),

        repr: Some(method_repr),
        str: Some(method_repr),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(method_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: Some(method_call),

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.methodtp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp.clone());
    finalize_type_dict(tp);
}
