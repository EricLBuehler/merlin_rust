use crate::{interpreter::VM, trc::Trc};

use super::{
    boolobject, finalize_type, intobject, stringobject, MethodType, MethodValue, Object, TypeObject,
};

fn object_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn object_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        "object".to_string(),
    ))
}
fn object_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        Trc::ptr_eq(&selfv, &other),
    ))
}
fn object_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), -1))
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("object"),
        bases: vec![super::ObjectBase::Object(vm.clone())],
        vm: vm.clone(),

        new: Some(object_new),

        repr: Some(object_repr),
        str: Some(object_repr),
        abs: None,
        neg: None,
        hash_fn: Some(object_hash),

        eq: Some(object_eq),
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

    vm.types.objecttp = Some(tp.clone());

    finalize_type(tp);
}
