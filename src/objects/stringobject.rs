use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use unicode_segmentation::UnicodeSegmentation;

use crate::interpreter::VM;
use crate::is_type_exact;
use crate::objects::exceptionobject::valueexc_from_str;
use crate::objects::{boolobject, intobject};
use crate::parser::Position;
use crate::trc::Trc;

use super::exceptionobject::typemismatchexc_from_str;
use super::{
    create_object_from_type, finalize_type, MethodType, MethodValue, Object, ObjectInternals,
    RawObject,
};

const MFBH_MAX_LEN: usize = 256;

pub fn string_from(vm: Trc<VM<'_>>, raw: String) -> Object<'_> {
    let mut tp = create_object_from_type(vm.types.strtp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Str(raw);
    tp
}

fn string_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn string_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(string_from(
        selfv.vm.clone(),
        "\"".to_owned()
            + selfv
                .internals
                .get_str()
                .expect("Expected str internal value")
            + "\"",
    ))
}
fn string_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(string_from(
        selfv.vm.clone(),
        selfv
            .internals
            .get_str()
            .expect("Expected str internal value")
            .to_string(),
    ))
}
fn string_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
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
            .get_str()
            .expect("Expected str internal value")
            == other
                .internals
                .get_str()
                .expect("Expected str internal value"),
    ))
}

fn string_get<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&other, &selfv.vm.types.inttp.as_ref().unwrap().clone()) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            &format!("Expected 'int' index, got '{}'", other.typename),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    //NEGATIVE INDEX IS CONVERTED TO +
    let out = UnicodeSegmentation::graphemes(
        selfv
            .internals
            .get_str()
            .expect("Expected str internal value")
            .as_str(),
        true,
    )
    .nth(
        (*other
            .internals
            .get_int()
            .expect("Expected int internal value"))
        .unsigned_abs() as usize,
    );

    if out.is_none() {
        let exc = valueexc_from_str(
            selfv.vm.clone(),
            &format!(
                "Index out of range: maximum index is '{}', but got '{}'",
                selfv
                    .internals
                    .get_str()
                    .expect("Expected str internal value")
                    .len(),
                (*other
                    .internals
                    .get_int()
                    .expect("Expected int internal value"))
                .unsigned_abs()
            ),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }
    MethodValue::Some(string_from(selfv.vm.clone(), out.unwrap().to_string()))
}
fn string_len(selfv: Object<'_>) -> MethodType<'_> {
    let convert: Result<i128, _> = selfv
        .internals
        .get_str()
        .expect("Expected str internal value")
        .len()
        .try_into();
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), convert.unwrap()))
}

#[inline]
fn string_hash(selfv: Object<'_>) -> MethodType<'_> {
    //Use DefaultHasher for long data:
    //https://www.reddit.com/r/rust/comments/hsbai0/default_hasher_for_u8_unexpectedly_expensive/
    //jschievink: ...DefaultHasher is an implementation of SipHash...   ...pretty fast on long data, for short data this hash tends to be very slow ...

    let bytes = selfv
        .internals
        .get_str()
        .expect("Expected str internal value")[..].as_bytes();

    if bytes.len() > MFBH_MAX_LEN {
        let mut hasher = DefaultHasher::new();
        selfv
            .internals
            .get_str()
            .expect("Expected str internal value")
            .hash(&mut hasher);
        return MethodValue::Some(intobject::int_from(
            selfv.vm.clone(),
            hasher.finish() as i128,
        ));
    }

    let len = bytes.len() as i128;
    if len == 0 {
        return MethodValue::Some(intobject::int_from(selfv.vm.clone(), 0))
    }
    else if len == 1 {
        return MethodValue::Some(intobject::int_from(selfv.vm.clone(), bytes[0] as i128))
    }

    let res = bytes[0] as i128 + bytes[bytes.len()-1] as i128;

    MethodValue::Some(intobject::int_from(selfv.vm.clone(), res+len))
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<RawObject<'a>> = Trc::new(RawObject {
        tp: super::ObjectType::Other(vm.types.typetp.as_ref().unwrap().clone()),
        internals: super::ObjectInternals::No,
        typename: String::from("str"),
        bases: vec![super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone())],
        vm: vm.clone(),

        new: Some(string_new),

        repr: Some(string_repr),
        str: Some(string_str),
        abs: None,
        neg: None,
        hash_fn: Some(string_hash),

        eq: Some(string_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: Some(string_get),
        set: None,
        len: Some(string_len),

        call: None,
    });

    vm.types.strtp = Some(tp.clone()); 

    finalize_type(tp);
}
