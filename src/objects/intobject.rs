use std::{sync::Arc};

use crate::{objects::is_instance, interpreter::VM};

use super::{RawObject, Object,MethodValue, ObjectInternals, create_object_from_type, stringobject, boolobject, finalize_type};

pub fn int_from<'a>(vm: Arc<VM<'a>>, raw: i128) -> Object<'a> {
    let mut tp = create_object_from_type(vm.get_type("int"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Int(raw);
    tp
}
pub fn int_from_str<'a>(vm: Arc<VM<'a>>, raw: String) -> MethodValue<Object<'a>, Object<'a>> {
    let convert = raw.parse::<i128>();
    debug_assert!(convert.is_ok());
    let mut tp = create_object_from_type(vm.get_type("int"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Int(convert.unwrap());
    MethodValue::Some(tp)
}


fn int_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    unimplemented!();
}

fn int_repr<'a>(selfv: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), selfv.internals.get_int().unwrap().to_string()))
}
fn int_abs<'a>(selfv: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    let res = selfv.internals.get_int().unwrap().checked_abs();
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    debug_assert!(is_instance(&selfv, &other));
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), selfv.internals.get_int().unwrap() == other.internals.get_int().unwrap()))
}


fn int_neg<'a>(selfv: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    let res = selfv.internals.get_int().unwrap().checked_neg();
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_add<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().unwrap();

    let res = selfv.internals.get_int().unwrap().checked_add(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_sub<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    debug_assert!(is_instance(&selfv, &other));

    let otherv = *other.internals.get_int().unwrap();

    let res = selfv.internals.get_int().unwrap().checked_sub(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_mul<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().unwrap();

    let res = selfv.internals.get_int().unwrap().checked_mul(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_div<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().unwrap();
    debug_assert!(otherv != 0);

    let res = selfv.internals.get_int().unwrap().checked_div(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_pow<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().unwrap();

    debug_assert!(otherv < std::u32::MAX as i128);

    let res = selfv.internals.get_int().unwrap().checked_pow(otherv as u32);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_hash<'a>(selfv: Object<'a>) -> MethodValue<Object<'a>, Object<'a>> {
    MethodValue::Some(int_from(selfv.vm.clone(), selfv.internals.get_int().unwrap().clone()))
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