use std::{sync::Arc};

use crate::objects::is_instance;

use super::{RawObject, Object, get_type, add_type, MethodValue, ObjectInternals, create_object_from_type, stringobject, boolobject};

pub fn int_from(raw: i128) -> Object {
    let mut tp = create_object_from_type(get_type("int"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Int(raw);
    tp
}
pub fn int_from_str(raw: String) -> MethodValue<Object, Object> {
    let convert = raw.parse::<i128>();
    debug_assert!(convert.is_ok());
    let mut tp = create_object_from_type(get_type("int"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Int(convert.unwrap());
    MethodValue::Some(tp)
}


fn int_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}

fn int_repr(selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(stringobject::string_from(selfv.internals.get_int().unwrap().to_string()))
}
fn int_abs(selfv: Object) -> MethodValue<Object, Object> {
    let res = selfv.internals.get_int().unwrap().checked_abs();
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(res.unwrap()))
}
fn int_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    MethodValue::Some(boolobject::bool_from(selfv.internals.get_int().unwrap() == other.internals.get_int().unwrap()))
}


fn int_neg(selfv: Object) -> MethodValue<Object, Object> {
    let res = selfv.internals.get_int().unwrap().checked_neg();
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(res.unwrap()))
}
fn int_add(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().unwrap();

    let res = selfv.internals.get_int().unwrap().checked_add(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(res.unwrap()))
}
fn int_sub(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));

    let otherv = *other.internals.get_int().unwrap();

    let res = selfv.internals.get_int().unwrap().checked_sub(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(res.unwrap()))
}
fn int_mul(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().unwrap();

    let res = selfv.internals.get_int().unwrap().checked_mul(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(res.unwrap()))
}
fn int_div(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().unwrap();
    debug_assert!(otherv != 0);

    let res = selfv.internals.get_int().unwrap().checked_div(otherv);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(res.unwrap()))
}
fn int_pow(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    let otherv = *other.internals.get_int().unwrap();

    debug_assert!(otherv < std::u32::MAX as i128);

    let res = selfv.internals.get_int().unwrap().checked_pow(otherv as u32);
    debug_assert!(res.is_some());

    MethodValue::Some(int_from(res.unwrap()))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("int"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(int_new),

        repr: Some(int_repr),
        abs: Some(int_abs),
        neg: Some(int_neg),

        eq: Some(int_eq),
        add: Some(int_add),
        sub: Some(int_sub),
        mul: Some(int_mul),
        div: Some(int_div),
        pow: Some(int_pow),
    });

    add_type(&tp.clone().typename, tp);
}