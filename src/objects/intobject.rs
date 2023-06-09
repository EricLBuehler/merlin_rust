use super::exceptionobject::{typemismatchexc_from_str, zerodivexc_from_str};
use super::{
    boolobject, create_object_from_type, finalize_type, stringobject, MethodType, MethodValue,
    Object, ObjectInternals, RawObject,
};

use crate::is_type_exact;
use crate::trc::Trc;
use crate::{
    interpreter::{INT_CACHE_SIZE, MAX_INT_CACHE, MIN_INT_CACHE, VM},
    objects::exceptionobject::overflowexc_from_str,
    parser::Position,
};
use std::collections::hash_map::DefaultHasher;

use std::hash::{Hash, Hasher};

pub fn int_from(vm: Trc<VM<'_>>, raw: i128) -> Object<'_> {
    if (MIN_INT_CACHE..=MAX_INT_CACHE).contains(&raw) {
        return vm.cache.int_cache[(raw + MIN_INT_CACHE.abs()) as usize]
            .as_ref()
            .unwrap()
            .clone();
    }
    let mut tp = create_object_from_type(vm.types.inttp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Int(raw);
    tp
}
pub fn int_from_str(vm: Trc<VM<'_>>, raw: String) -> MethodType<'_> {
    let convert = raw.parse::<i128>();
    if matches!(convert, Result::Err(_)) {
        let exc = overflowexc_from_str(
            vm.clone(),
            &("int literal is invalid (".to_owned() + &convert.err().unwrap().to_string() + ")"),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }
    if convert.as_ref().unwrap() >= &MIN_INT_CACHE && convert.as_ref().unwrap() <= &MAX_INT_CACHE {
        return MethodValue::Some(
            vm.cache.int_cache[(convert.unwrap() + MIN_INT_CACHE.abs()) as usize]
                .as_ref()
                .unwrap()
                .clone(),
        );
    }
    let mut tp = create_object_from_type(vm.types.inttp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Int(convert.unwrap());
    MethodValue::Some(tp)
}

fn int_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}

fn int_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        selfv
            .internals
            .get_int()
            .expect("Expected int internal value")
            .to_string(),
    ))
}
fn int_abs(selfv: Object<'_>) -> MethodType<'_> {
    let res = selfv
        .internals
        .get_int()
        .expect("Expected int internal value")
        .checked_abs();
    if res.is_none() {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int absolute value overflow (value is i128 minimum)",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, &other) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        selfv
            .internals
            .get_int()
            .expect("Expected int internal value")
            == other
                .internals
                .get_int()
                .expect("Expected int internal value"),
    ))
}

fn int_neg(selfv: Object<'_>) -> MethodType<'_> {
    let res = selfv
        .internals
        .get_int()
        .expect("Expected int internal value")
        .checked_neg();
    if matches!(res, Option::None) {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int negation overflow (value is i128 minimum)",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_add<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, &other) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = *other
        .internals
        .get_int()
        .expect("Expected int internal value");

    let res = selfv
        .internals
        .get_int()
        .expect("Expected int internal value")
        .checked_add(otherv);
    if matches!(res, Option::None) {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int addition overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_sub<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, &other) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = *other
        .internals
        .get_int()
        .expect("Expected int internal value");

    let res = selfv
        .internals
        .get_int()
        .expect("Expected int internal value")
        .checked_sub(otherv);
    if matches!(res, Option::None) {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int subtraction overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_mul<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, &other) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = *other
        .internals
        .get_int()
        .expect("Expected int internal value");

    let res = selfv
        .internals
        .get_int()
        .expect("Expected int internal value")
        .checked_mul(otherv);
    if matches!(res, Option::None) {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int multiplication overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_div<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, &other) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = *other
        .internals
        .get_int()
        .expect("Expected int internal value");
    if otherv == 0 {
        let exc = zerodivexc_from_str(
            selfv.vm.clone(),
            "Divison by 0",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let res = selfv
        .internals
        .get_int()
        .expect("Expected int internal value")
        .checked_div(otherv);
    if matches!(res, Option::None) {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int division overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_pow<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, &other) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = *other
        .internals
        .get_int()
        .expect("Expected int internal value");

    if otherv >= std::u32::MAX as i128 {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "Power is too large",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let res = selfv
        .internals
        .get_int()
        .expect("Expected int internal value")
        .checked_pow(otherv as u32);
    if matches!(res, Option::None) {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int power overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), res.unwrap()))
}
fn int_hash(selfv: Object<'_>) -> MethodType<'_> {
    let mut hasher = DefaultHasher::new();
    selfv
        .internals
        .get_int()
        .expect("Expected int internal value")
        .hash(&mut hasher);
    return MethodValue::Some(int_from(selfv.vm.clone(), hasher.finish() as i128));
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

pub fn generate_cache<'a>(
    int: Object<'a>,
    arr: *mut [Option<Object<'a>>; INT_CACHE_SIZE as usize],
) {
    unsafe {
        let mut i = MIN_INT_CACHE;
        for item in &mut (*arr)[..] {
            let mut tp = create_object_from_type(int.clone());
            tp.internals = ObjectInternals::Int(i);
            std::ptr::write(item, Some(tp));
            i += 1;
        }
    }
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<RawObject<'a>> = Trc::new(RawObject {
        tp: super::ObjectType::Other(vm.types.typetp.as_ref().unwrap().clone()),
        internals: super::ObjectInternals::No,
        typename: String::from("int"),
        bases: vec![super::ObjectBase::Other(
            vm.types.objecttp.as_ref().unwrap().clone(),
        )],
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

    vm.types.inttp = Some(tp.clone());

    finalize_type(tp);
}
