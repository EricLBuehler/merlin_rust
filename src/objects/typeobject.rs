use std::mem::ManuallyDrop;

use crate::interpreter::VM;
use trc::Trc;

use super::{
    boolobject, finalize_type, finalize_type_dict, intobject, stringobject, unwrap_fast,
    MethodType, MethodValue, Object, TypeObject,
};

fn type_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}

fn type_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("<class '{}'>", unsafe { &selfv.internals.typ }.typename),
    ))
}
fn type_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        selfv.tp == other.tp,
    ))
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("type"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(type_new),
        del: Some(|mut selfv| unsafe { ManuallyDrop::drop(&mut selfv.internals.typ) }),

        repr: Some(type_repr),
        str: Some(type_repr),
        abs: None,
        neg: None,
        hash_fn: Some(|selfv: Object<'a>| {
            MethodValue::Some(intobject::int_from(selfv.vm.clone(), -3))
        }),

        eq: Some(type_eq),
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

    vm.types.typetp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp.clone());
    finalize_type_dict(tp);
}
