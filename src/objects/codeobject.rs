use std::mem::ManuallyDrop;

use super::finalize_type_dict;
use super::{
    create_object_from_type, finalize_type, MethodType, MethodValue, Object, RawObject, TypeObject,
};
use crate::is_type_exact;
use crate::unwrap_fast;
use crate::{
    compiler::Bytecode,
    interpreter::VM,
    objects::{boolobject, stringobject, ObjectInternals},
};
use trc::Trc;

pub fn code_from<'a>(vm: Trc<VM<'a>>, bytecode: Trc<Bytecode<'a>>) -> Object<'a> {
    let mut tp: Trc<RawObject> =
        create_object_from_type(unwrap_fast!(vm.types.codetp.as_ref()).clone(), vm, None);
    tp.internals = ObjectInternals {
        code: ManuallyDrop::new(bytecode),
    };
    tp
}

fn code_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn code_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("<code object @ 0x{:x}>", Trc::as_ptr(&selfv) as usize),
    ))
}
fn code_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        unsafe { &selfv.internals.code } == unsafe { &other.internals.code },
    ))
}

pub fn init(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("code"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(code_new),

        repr: Some(code_repr),
        str: Some(code_repr),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(code_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.codetp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp.clone());
    finalize_type_dict(tp);
}
