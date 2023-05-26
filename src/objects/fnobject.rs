use std::{sync::Arc};
use crate::{objects::{stringobject, ObjectInternals, boolobject}, interpreter::VM};

use super::{RawObject, Object,MethodValue, finalize_type, create_object_from_type};


pub fn fn_from<'a>(vm: Arc<VM<'a>>, code: Object<'a>, args: Vec<Object<'a>>, name: String) -> Object<'a> {
    let mut tp = create_object_from_type(vm.get_type("fn"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Fn(super::FnData { code, args, name });
    tp
}

fn fn_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    unimplemented!();
}
fn fn_repr<'a>(selfv: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), format!("<fn '{}' @ 0x{:x}>",selfv.internals.get_fn().unwrap().name, Arc::as_ptr(&selfv) as i128)))
}
fn fn_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), selfv.internals.get_fn().unwrap() == other.internals.get_fn().unwrap()))
}

fn fn_call<'a>(selfv: Object<'a>, args: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    unimplemented!();
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("fn"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(fn_new),

        repr: Some(fn_repr),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(fn_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,
    
        get: None,
        set: None,
        len: None,

        call: Some(fn_call),
    });

    vm.clone().add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}