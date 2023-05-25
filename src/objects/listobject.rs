use std::{sync::Arc};
use crate::objects::{stringobject, noneobject, ObjectInternals, boolobject};

use super::{RawObject, Object, get_type, add_type, MethodValue, utils, finalize_type, is_instance, intobject, create_object_from_type};


pub fn list_from(raw: Vec<Object>) -> Object {
    let mut tp = create_object_from_type(get_type("list"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Arr(raw);
    tp
}

fn list_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn list_repr(selfv: Object) -> MethodValue<Object, Object> {
    let mut res = String::from("[");
    for item in selfv.internals.get_arr().unwrap() {
        let repr = utils::object_repr_safe(item);
        if !repr.is_some() {
            return MethodValue::NotImplemented;
        }
        res += &repr.unwrap();
        res += ", ";
    }
    if res.len() > 1 {
        res.pop();
        res.pop();
    }
    res += "]";
    MethodValue::Some(stringobject::string_from(res))
}

fn list_get(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&other, &get_type("int")));
    //NEGATIVE INDEX IS CONVERTED TO +
    let out = selfv.internals.get_arr().unwrap().get(other.internals.get_int().unwrap().clone().abs() as usize);
    debug_assert!(out.is_some());
    MethodValue::Some(out.unwrap().clone())
}
fn list_set(selfv: Object, other: Object, value: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&other, &get_type("int")));
    //NEGATIVE INDEX IS CONVERTED TO +
    debug_assert!((other.internals.get_int().unwrap().clone().abs() as usize) < selfv.internals.get_arr().unwrap().len());
    let mut arr = selfv.internals.get_arr().unwrap().clone();
    arr[other.internals.get_int().unwrap().clone().abs() as usize] = value;
    
    unsafe {
        let refr = Arc::into_raw(selfv.clone()) as *mut RawObject;
        (*refr).internals = ObjectInternals::Arr(arr.to_vec());
    }
    
    MethodValue::Some(noneobject::none_from())
}
fn list_len(selfv: Object) -> MethodValue<Object, Object> {
    let convert: Result<i128, _> = selfv.internals.get_arr().unwrap().len().try_into();
    debug_assert!(convert.is_ok());
    MethodValue::Some(intobject::int_from(convert.unwrap()))
}

fn list_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    debug_assert!(selfv.internals.get_arr().unwrap().len() == other.internals.get_arr().unwrap().len());
    for idx in 0..selfv.internals.get_arr().unwrap().len() {
        debug_assert!(selfv.internals.get_arr().unwrap().get(idx).unwrap().eq.is_some());
        let v = selfv.internals.get_arr().unwrap().get(idx).unwrap();
        let res = (v.eq.unwrap())(v.clone(), other.internals.get_arr().unwrap().get(idx).unwrap().clone());
        debug_assert!(res.is_some());
        debug_assert!(is_instance(&res.unwrap(), &get_type("bool")));
        if *res.unwrap().internals.get_bool().unwrap() {
            return MethodValue::Some(boolobject::bool_from(false));
        }
    }
    MethodValue::Some(boolobject::bool_from(true))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("list"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(list_new),

        repr: Some(list_repr),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(list_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,
        
        get: Some(list_get),
        set: Some(list_set),
        len: Some(list_len),
    });

    add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}