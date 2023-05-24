use std::{sync::Arc, collections::hash_map::DefaultHasher};
use unicode_segmentation::UnicodeSegmentation;
use std::hash::Hash;
use std::hash::Hasher;

use crate::objects::{is_instance, boolobject, intobject};

use super::{RawObject, Object, get_type, add_type, MethodValue, ObjectInternals, create_object_from_type, finalize_type};


pub fn string_from(raw: String) -> Object {
    let mut tp = create_object_from_type(get_type("str"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Str(raw);
    tp
}

fn string_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn string_repr(selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(selfv)
}
fn string_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    MethodValue::Some(boolobject::bool_from(selfv.internals.get_str().unwrap() == other.internals.get_str().unwrap()))
}

fn string_get(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    is_instance(&other, &get_type("int"));
    //NEGATIVE INDEX IS CONVERTED TO +
    let out = UnicodeSegmentation::graphemes(selfv.internals.get_str().unwrap().as_str(), true).nth(other.internals.get_int().unwrap().clone().abs() as usize);
    debug_assert!(out.is_some());
    MethodValue::Some(string_from(out.unwrap().to_string()))
}
fn string_len(selfv: Object) -> MethodValue<Object, Object> {
    let convert: Result<i128, _> = selfv.internals.get_str().unwrap().len().try_into();
    debug_assert!(convert.is_ok());
    MethodValue::Some(intobject::int_from(convert.unwrap()))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("str"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(string_new),

        repr: Some(string_repr),
        abs: None,
        neg: None,
        hash_fn: Some(|selfv: Object| {
            let mut hasher = DefaultHasher::new();
            selfv.internals.get_str().unwrap().hash(&mut hasher);
            
            MethodValue::Some(intobject::int_from(hasher.finish() as i128))
        }),

        eq: Some(string_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,
        
        get: Some(string_get),
        set: None,
        len: Some(string_len),
    });

    add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}