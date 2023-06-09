use crate::{interpreter::VM, trc::Trc};

use super::{
    boolobject, finalize_type, intobject, stringobject, MethodType, MethodValue, Object, TypeObject,
};

fn type_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}

fn type_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("<class '{}'>", selfv.tp.typename),
    ))
}
fn type_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        selfv.tp == other.tp,
    ))
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("type"),
        bases: vec![super::ObjectBase::Other(
            vm.types.objecttp.as_ref().unwrap().clone(),
        )],
        vm: vm.clone(),

        new: Some(type_new),

        repr: Some(type_repr),
        str: Some(type_repr),
        abs: None,
        neg: None,
        hash_fn: Some(|selfv: Object<'a>| {
            MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -3))
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
