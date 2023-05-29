use std::{sync::Arc};

use crate::{objects::is_instance, interpreter::{VM, INT_CACHE_SIZE, MIN_INT_CACHE, MAX_INT_CACHE}};

use super::{RawObject, Object,MethodType, MethodValue, ObjectInternals, create_object_from_type, stringobject, boolobject, finalize_type};

pub fn int_from(vm: Arc<VM<'_>>, raw: i128) -> Object<'_> {
    if (MIN_INT_CACHE..=MAX_INT_CACHE).contains(&raw) {
        return vm.cache.int_cache[(raw + MIN_INT_CACHE.abs()) as usize].as_ref().unwrap().clone();
    }
    let mut tp = create_object_from_type(vm.get_type("int"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Int(raw);
    tp
}
pub fn int_from_str(vm: Arc<VM<'_>>, raw: String) -> MethodType<'_> {
    let convert = raw.parse::<i128>();
    debug_assert!(convert.is_ok());
    if convert.as_ref().unwrap() >= &MIN_INT_CACHE && convert.as_ref().unwrap() <= &MAX_INT_CACHE {
        return MethodValue::Some(vm.cache.int_cache[(convert.unwrap() + MIN_INT_CACHE.abs()) as usize].as_ref().unwrap().clone());
    }
    let mut tp = create_object_from_type(vm.get_type("int"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Int(convert.unwrap());
    MethodValue::Some(tp)
}


fn int_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}

fn int_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), selfv.internals.get_int().expect("Expected int internal value").to_string()))
}
fn int_abs(selfv: Object<'_>) -> MethodType<'_> {
    let res = selfv.internals.get_int().expect("Expected int internal value").checked_abs();
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), selfv.internals.get_int().expect("Expected int internal value") == other.internals.get_int().expect("Expected int internal value")))
}


fn int_neg(selfv: Object<'_>) -> MethodType<'_> {
    let res = selfv.internals.get_int().expect("Expected int internal value").checked_neg();
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_add<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().expect("Expected int internal value");

    let res = selfv.internals.get_int().expect("Expected int internal value").checked_add(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_sub<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));

    let otherv = *other.internals.get_int().expect("Expected int internal value");

    let res = selfv.internals.get_int().expect("Expected int internal value").checked_sub(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_mul<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().expect("Expected int internal value");

    let res = selfv.internals.get_int().expect("Expected int internal value").checked_mul(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_div<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().expect("Expected int internal value");
    debug_assert!(otherv != 0);

    let res = selfv.internals.get_int().expect("Expected int internal value").checked_div(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_pow<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().expect("Expected int internal value");

    debug_assert!(otherv < std::u32::MAX as i128);

    let res = selfv.internals.get_int().expect("Expected int internal value").checked_pow(otherv as u32);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(int_from(selfv.vm.clone(), *selfv.internals.get_int().expect("Expected int internal value")))
}

pub fn init_cache<'a>() -> [Option<Object<'a>>; INT_CACHE_SIZE as usize] {
    unsafe {
        let init = std::mem::MaybeUninit::uninit();
        let mut arr: [Option<Object>; INT_CACHE_SIZE as usize] = init.assume_init();

        for item in &mut arr[..] {
            std::ptr::write(item, None);
        }

        arr
    }
}

pub fn generate_cache<'a>(int: Object<'a>, arr: *mut [Option<Object<'a>>; INT_CACHE_SIZE as usize]) {
    unsafe {
        let mut i = MIN_INT_CACHE;
        for item in &mut (*arr)[..] {
            let mut tp = create_object_from_type(int.clone());
            let mut refr = Arc::make_mut(&mut tp);
            refr.internals = ObjectInternals::Int(i);
            std::ptr::write(item, Some(tp));
            i+=1;
        }
    }
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("int"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(int_new),

        repr: Some(int_repr),
        str: Some(int_repr),
        abs: Some(int_abs),
        neg: Some(int_neg),
        hash_fn: Some(int_hash),

        eq: Some(int_eq),
        add: Some(int_add),
        sub: Some(int_sub),
        mul: Some(int_mul),
        div: Some(int_div),
        pow: Some(int_pow),
        
        get: None,
        set: None,
        len: None,

        call: None,
    });

    vm.clone().add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}