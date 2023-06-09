use crate::{interpreter::VM, trc::Trc};

use super::{
    boolobject, create_object_from_type, finalize_type, intobject, stringobject, MethodType,
    MethodValue, Object, RawObject,
};

fn type_new<'a>(selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(create_object_from_type(selfv))
}
fn type_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("<class '{}'>", selfv.typename),
    ))
}
fn type_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        selfv.typename == other.typename,
    ))
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<RawObject<'a>> = Trc::new(RawObject {
        tp: super::ObjectType::Type(vm.clone()),
        internals: super::ObjectInternals::No,
        typename: String::from("type"),
        bases: vec![super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone())],
        vm: vm.clone(),

        new: Some(type_new),

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
    });

    vm.types.typetp = Some(tp.clone()); 

    finalize_type(tp);
}
