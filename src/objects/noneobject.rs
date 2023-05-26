use std::{sync::Arc};
use crate::{objects::stringobject, interpreter::VM};

use super::{RawObject, Object,MethodType, MethodValue, create_object_from_type, finalize_type, is_instance, boolobject, intobject};

pub fn none_from(vm: Arc<VM<'_>>) -> Object<'_> {
    create_object_from_type(vm.get_type("NoneType"))
}

fn none_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn none_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), String::from("None")))
}
fn none_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), -2))
}
fn none_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), is_instance(&selfv, &other)))
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("NoneType"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(none_new),

        repr: Some(none_repr),
        abs: None,
        neg: None,
        hash_fn: Some(none_hash),

        eq: Some(none_eq),
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

    vm.clone().add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}