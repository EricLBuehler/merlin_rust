use std::{sync::Arc, collections::HashMap};
use crate::objects::{stringobject, noneobject, ObjectInternals, boolobject};

use super::{RawObject, Object, get_type, add_type, MethodValue, utils, finalize_type, is_instance, intobject, create_object_from_type};


pub fn dict_from(raw: HashMap<Object, Object>) -> Object {
    let tp = create_object_from_type(get_type("dict"));
    unsafe {
        let refr = Arc::into_raw(tp.clone()) as *mut RawObject;
        (*refr).internals = ObjectInternals::Map(raw);
    }
    tp
}

fn dict_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn dict_repr(selfv: Object) -> MethodValue<Object, Object> {
    let mut res = String::from("{");
    for (key, value) in selfv.internals.get_map().unwrap() {
        let repr = utils::object_repr_safe(key);
        if !repr.is_some() {
            return MethodValue::NotImplemented;
        }
        res += &repr.unwrap();
        res += ": ";
        let repr = utils::object_repr_safe(value);
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
    res += "}";
    MethodValue::Some(stringobject::string_from(res))
}

fn dict_get(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    is_instance(&other, &get_type("int"));
    //NEGATIVE INDEX IS CONVERTED TO +
    let out = selfv.internals.get_map().unwrap().get(&other);
    debug_assert!(out.is_some());
    MethodValue::Some(out.unwrap().clone())
}
fn dict_set(selfv: Object, other: Object, value: Object) -> MethodValue<Object, Object> {
    //DEBUG check for hash here!
    let mut map = selfv.internals.get_map().unwrap().clone();
    map.insert(other, value);

    unsafe {
        let refr = Arc::into_raw(selfv.clone()) as *mut RawObject;
        (*refr).internals = ObjectInternals::Map(map);
    }

    MethodValue::Some(noneobject::none_from())
}
fn dict_len(selfv: Object) -> MethodValue<Object, Object> {
    let convert: Result<i128, _> = selfv.internals.get_map().unwrap().len().try_into();
    debug_assert!(convert.is_ok());
    MethodValue::Some(intobject::int_from(convert.unwrap()))
}

fn dict_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    debug_assert!(selfv.internals.get_map().unwrap().len() == other.internals.get_map().unwrap().len());
    for ((key1, value1), (key2, value2)) in std::iter::zip(selfv.internals.get_map().unwrap(), other.internals.get_map().unwrap()) {
        debug_assert!(key1.eq.is_some());
        debug_assert!(value1.eq.is_some());
        debug_assert!(key2.eq.is_some());
        debug_assert!(value2.eq.is_some());
        
        let res = (key1.eq.unwrap())(key1.clone(), key2.clone());
        debug_assert!(res.is_some());
        debug_assert!(is_instance(&res.unwrap(), &get_type("bool")));
        if *res.unwrap().internals.get_bool().unwrap() {
            return MethodValue::Some(boolobject::bool_from(false));
        }
        
        let res: MethodValue<Arc<RawObject>, Arc<RawObject>> = (value1.eq.unwrap())(value1.clone(), value2.clone());
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
        typename: String::from("dict"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(dict_new),

        repr: Some(dict_repr),
        abs: None,
        neg: None,
        hash_fn: None,

        eq: Some(dict_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,
        
        get: Some(dict_get),
        set: Some(dict_set),
        len: Some(dict_len),
    });

    add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}