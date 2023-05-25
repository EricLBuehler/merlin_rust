use std::{sync::Arc};
use crate::{objects::{stringobject, ObjectInternals, boolobject}};

use super::{RawObject, Object, get_type, add_type, MethodValue, finalize_type, create_object_from_type};


pub fn fn_from(code: Object, args: Vec<Object>, name: String) -> Object {
    let mut tp = create_object_from_type(get_type("fn"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Fn(super::FnData { code, args, name });
    tp
}

fn fn_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn fn_repr(selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(stringobject::string_from(format!("<fn '{}' @ 0x{:x}>",selfv.internals.get_fn().unwrap().name, Arc::as_ptr(&selfv) as i128)))
}
fn fn_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(boolobject::bool_from(selfv.internals.get_fn().unwrap() == other.internals.get_fn().unwrap()))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("fn"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

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
    });

    add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}