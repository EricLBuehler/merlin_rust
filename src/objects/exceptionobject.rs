use std::{sync::Arc};
use crate::{interpreter::VM, parser::Position};

use super::{RawObject, finalize_type, Object, stringobject, MethodType, MethodValue, intobject, boolobject, is_instance, utils, ObjectInternals, create_object_from_type, ExcData};


fn exc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn exc_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), String::from("Exception<>")))
}
fn exc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), -2))
}
fn exc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), is_instance(&selfv, &other)))
}


pub fn init_exc<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("Exception"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(exc_new),

        repr: Some(exc_repr),
        str: Some(exc_repr),
        abs: None,
        neg: None,
        hash_fn: Some(exc_hash),

        eq: Some(exc_eq),
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


// =====================

#[allow(dead_code)]
pub fn nameexc_from_obj<'a>(vm: Arc<VM<'a>>, obj: Object<'a>, start: Position, end: Position) -> Object<'a> {
    let tp = create_object_from_type(vm.get_type("NameExc"));
    unsafe {
        let refr = Arc::into_raw(tp.clone()) as *mut RawObject<'a>;
        (*refr).internals = ObjectInternals::Exc(ExcData {obj, start, end});
    }
    tp
}
pub fn nameexc_from_str<'a>(vm: Arc<VM<'a>>, raw: &str, start: Position, end: Position) -> Object<'a> {
    let tp = create_object_from_type(vm.get_type("NameExc"));
    unsafe {
        let refr = Arc::into_raw(tp.clone()) as *mut RawObject<'a>;
        (*refr).internals = ObjectInternals::Exc(ExcData {obj: stringobject::string_from(vm.clone(), raw.to_string()), start, end});
    }
    tp
}

fn nameexc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn nameexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = utils::object_str_safe(&selfv.internals.get_exc().expect("Expected exc internal value").obj);
    if !repr.is_some() {
        return MethodValue::NotImplemented;
    }
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), format!("NameException: \"{}\"", repr.unwrap())))
}
fn nameexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(selfv.internals.get_exc().expect("Expected exc internal value").obj)
}
fn nameexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), -2))
}
fn nameexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), is_instance(&selfv, &other)))
}


pub fn init_nameexc<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("NameExc"),
        bases: vec![super::ObjectBase::Other(vm.get_type("Exception")), super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(nameexc_new),

        repr: Some(nameexc_repr),
        str: Some(nameexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(nameexc_hash),

        eq: Some(nameexc_eq),
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