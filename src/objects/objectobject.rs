use super::{
    boolobject, finalize_type, intobject, stringobject, MethodType, MethodValue, Object, TypeObject,
};
use crate::interpreter::VM;
use trc::Trc;

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

pub fn init(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("object"),
        bases: vec![super::ObjectBase::Object(vm.clone())],
        typeid: vm.types.n_types,

        new: Some(object_new),
        del: Some(|_| {}),

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
    vm.types.n_types += 1;

    finalize_type(tp);
}
