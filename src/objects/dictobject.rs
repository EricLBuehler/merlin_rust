use std::{sync::Arc};
use crate::{objects::{stringobject, noneobject, ObjectInternals, boolobject}, interpreter::VM};

use super::{RawObject, Object,MethodType, MethodValue, utils, finalize_type, is_instance, intobject, create_object_from_type};

use ahash::AHashMap;

pub fn dict_from<'a>(vm: Arc<VM<'a>>, raw: AHashMap<Object<'a>, Object<'a>>) -> Object<'a> {
    let tp = create_object_from_type(vm.get_type("dict"));
    unsafe {
        let refr = Arc::into_raw(tp.clone()) as *mut RawObject<'a>;
        (*refr).internals = ObjectInternals::Map(raw);
    }
    tp
}

fn dict_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn dict_repr(selfv: Object<'_>) -> MethodType<'_> {
    let mut res = String::from("{");
    for (key, value) in selfv.internals.get_map().expect("Expected map internal value") {
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
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), res))
}

fn dict_get<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    is_instance(&other, &selfv.vm.get_type("int"));
    //NEGATIVE INDEX IS CONVERTED TO +
    let out = selfv.internals.get_map().expect("Expected map internal value").get(&other);
    debug_assert!(out.is_some());
    MethodValue::Some(out.unwrap().clone())
}
#[inline(always)]
fn dict_set<'a>(selfv: Object<'a>, other: Object<'a>, value: Object<'a>) -> MethodType<'a> {
    ////println!("FCALL");
    //DEBUG check for hash here!
    let mut map = selfv.internals.get_map().expect("Expected map internal value").clone();
    ////println!("START");
    map.insert(other, value);
    ////println!("ENDING");

    unsafe {
        let refr = Arc::into_raw(selfv.clone()) as *mut RawObject<'a>;
        (*refr).internals = ObjectInternals::Map(map);
    }

    MethodValue::Some(noneobject::none_from(selfv.vm.clone()))
}
fn dict_len(selfv: Object<'_>) -> MethodType<'_> {
    let convert: Result<i128, _> = selfv.internals.get_map().expect("Expected map internal value").len().try_into();
    debug_assert!(convert.is_ok());
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), convert.unwrap()))
}

fn dict_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    debug_assert!(selfv.internals.get_map().expect("Expected map internal value").len() == other.internals.get_map().expect("Expected map internal value").len());
    for ((key1, value1), (key2, value2)) in std::iter::zip(selfv.internals.get_map().expect("Expected map internal value"), other.internals.get_map().expect("Expected map internal value")) {
        debug_assert!(key1.eq.is_some());
        debug_assert!(value1.eq.is_some());
        debug_assert!(key2.eq.is_some());
        debug_assert!(value2.eq.is_some());
        
        let res = (key1.eq.expect("Method is not defined"))(key1.clone(), key2.clone());
        debug_assert!(res.is_some());
        debug_assert!(is_instance(&res.unwrap(), &selfv.vm.get_type("bool")));
        if *res.unwrap().internals.get_bool().expect("Expected bool internal value") {
            return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
        }
        
        let res: MethodValue<Arc<RawObject<'a>>, Arc<RawObject<'a>>> = (value1.eq.expect("Method is not defined"))(value1.clone(), value2.clone());
        debug_assert!(res.is_some());
        debug_assert!(is_instance(&res.unwrap(), &selfv.vm.get_type("bool")));
        if *res.unwrap().internals.get_bool().expect("Expected bool internal value") {
            return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
        }
    }
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), true))
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("dict"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(dict_new),

        repr: Some(dict_repr),
        str: Some(dict_repr),
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
        
        call: None,
    });

    vm.clone().add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}