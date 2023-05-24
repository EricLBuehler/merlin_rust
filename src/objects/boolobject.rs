use std::{sync::Arc};
use crate::objects::{stringobject, is_instance, boolobject};

use super::{RawObject, Object, get_type, add_type, MethodValue, ObjectInternals, create_object_from_type, finalize_type};


pub fn bool_from(raw: bool) -> Object {
    let mut tp = create_object_from_type(get_type("bool"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Bool(raw);
    tp
}

fn bool_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn bool_repr(selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(stringobject::string_from(selfv.internals.get_bool().unwrap().to_string()))
}
fn bool_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    MethodValue::Some(boolobject::bool_from(selfv.internals.get_bool().unwrap() == other.internals.get_bool().unwrap()))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("bool"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(bool_new),

        repr: Some(bool_repr),
        abs: None,
        neg: None,

        eq: Some(bool_eq),
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