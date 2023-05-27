use std::{sync::Arc};
use crate::{objects::{stringobject, is_instance, boolobject}, interpreter::VM};

use super::{RawObject, Object,MethodType, MethodValue, ObjectInternals, create_object_from_type, finalize_type, intobject};


pub fn bool_from(vm: Arc<VM<'_>>, raw: bool) -> Object<'_> {
    let mut tp = create_object_from_type(vm.get_type("bool"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Bool(raw);
    tp
}   

fn bool_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn bool_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), selfv.internals.get_bool().expect("Expected bool internal value").to_string()))
}
fn bool_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), selfv.internals.get_bool().expect("Expected bool internal value") == other.internals.get_bool().expect("Expected bool internal value")))
}
fn bool_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), *selfv.internals.get_bool().expect("Expected bool internal value") as i128))
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("bool"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(bool_new),

        repr: Some(bool_repr),
        abs: None,
        neg: None,
        hash_fn: Some(bool_hash),

        eq: Some(bool_eq),
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