use super::exceptionobject::{typemismatchexc_from_str, zerodivexc_from_str};
use super::{
    boolobject, create_object_from_type, finalize_type, finalize_type_dict, stringobject,
    MethodType, MethodValue, Object, ObjectInternals, TypeObject,
};

use crate::is_type_exact;
use crate::unwrap_fast;
use crate::{
    interpreter::{INT_CACHE_OFFSET, INT_CACHE_SIZE, MAX_INT_CACHE, MIN_INT_CACHE, VM},
    objects::exceptionobject::overflowexc_from_str,
    parser::Position,
};
use std::collections::hash_map::DefaultHasher;
use trc::Trc;

use std::hash::{Hash, Hasher};

#[inline]
pub fn int_from(vm: Trc<VM<'_>>, raw: isize) -> Object<'_> {
    if (MIN_INT_CACHE..=MAX_INT_CACHE).contains(&raw) {
        return unwrap_fast! {vm.cache.int_cache[(raw + INT_CACHE_OFFSET) as usize]
        .as_ref()}
        .clone();
    }
    let mut tp = create_object_from_type(unwrap_fast!(vm.types.inttp.as_ref()).clone(), vm, None);
    tp.internals = ObjectInternals { int: raw };
    tp
}
pub fn int_from_str(vm: Trc<VM<'_>>, raw: String) -> MethodType<'_> {
    let convert = raw.parse::<isize>();
    if matches!(convert, Result::Err(_)) {
        let exc = overflowexc_from_str(
            vm.clone(),
            &("int literal is invalid (".to_owned() + &convert.err().unwrap().to_string() + ")"),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }
    MethodValue::Some(int_from(vm, *unwrap_fast!(convert.as_ref())))
}

fn int_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}

fn int_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        unsafe { selfv.internals.int }.to_string(),
    ))
}
fn int_abs(selfv: Object<'_>) -> MethodType<'_> {
    let res = unsafe { selfv.internals.int }.checked_abs();
    if res.is_none() {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int absolute value overflow (value is i128 minimum)",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), unwrap_fast!(res)))
}
fn int_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        unsafe { selfv.internals.int } == unsafe { other.internals.int },
    ))
}

fn int_neg(selfv: Object<'_>) -> MethodType<'_> {
    let res = unsafe { selfv.internals.int }.checked_neg();
    if res.is_none() {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int negation overflow (value is i128 minimum)",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), unwrap_fast!(res)))
}
fn int_add<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = unsafe { other.internals.int };

    let res = unsafe { selfv.internals.int }.checked_add(otherv);
    if res.is_none() {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int addition overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), unwrap_fast!(res)))
}
fn int_sub<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = unsafe { other.internals.int };

    let res = unsafe { selfv.internals.int }.checked_sub(otherv);
    if res.is_none() {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int subtraction overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), unwrap_fast!(res)))
}
fn int_mul<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = unsafe { other.internals.int };

    let res = unsafe { selfv.internals.int }.checked_mul(otherv);
    if res.is_none() {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int multiplication overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), unwrap_fast!(res)))
}
fn int_div<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = unsafe { other.internals.int };
    if otherv == 0 {
        let exc = zerodivexc_from_str(
            selfv.vm.clone(),
            "Division by 0",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let res = unsafe { selfv.internals.int }.checked_div(otherv);
    if res.is_none() {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int division overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), unwrap_fast!(res)))
}
fn int_pow<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let otherv = unsafe { other.internals.int };

    if otherv >= std::u32::MAX as isize {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "Power is too large",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let res = unsafe { selfv.internals.int }.checked_pow(otherv as u32);
    if res.is_none() {
        let exc = overflowexc_from_str(
            selfv.vm.clone(),
            "int power overflow",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(int_from(selfv.vm.clone(), unwrap_fast!(res)))
}
fn int_hash(selfv: Object<'_>) -> MethodType<'_> {
    let mut hasher = DefaultHasher::new();
    unsafe { selfv.internals.int }.hash(&mut hasher);
    return MethodValue::Some(int_from(selfv.vm.clone(), hasher.finish() as isize));
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
    vm: Trc<VM<'a>>,
    int: Trc<TypeObject<'a>>,
    arr: *mut [Option<Object<'a>>; INT_CACHE_SIZE as usize],
) {
    unsafe {
        let mut i = MIN_INT_CACHE;
        for item in &mut (*arr)[..] {
            let mut tp = create_object_from_type(int.clone(), vm.clone(), None);
            tp.internals = ObjectInternals { int: i };
            std::ptr::write(item, Some(tp));
            i += 1;
        }
    }
}

pub fn init(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("int"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

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

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.inttp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp.clone());
    finalize_type_dict(tp);
}
