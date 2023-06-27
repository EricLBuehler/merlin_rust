use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;
use unicode_segmentation::UnicodeSegmentation;

use crate::interpreter::VM;
use crate::is_type_exact;
use crate::objects::exceptionobject::valueexc_from_str;
use crate::objects::{boolobject, intobject};
use crate::parser::Position;
use crate::unwrap_fast;
use trc::Trc;

use super::exceptionobject::typemismatchexc_from_str;
use super::{
    create_object_from_type, finalize_type, finalize_type_dict, MethodType, MethodValue, Object,
    ObjectInternals, TypeObject,
};

const MFBH_MAX_LEN: usize = 256;

pub fn string_from(vm: Trc<VM<'_>>, raw: String) -> Object<'_> {
    let mut tp = create_object_from_type(unwrap_fast!(vm.types.strtp.as_ref()).clone(), vm, None);
    tp.internals = ObjectInternals {
        str: ManuallyDrop::new(raw),
    };
    tp
}

fn string_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn string_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(string_from(
        selfv.vm.clone(),
        "\"".to_owned() + unsafe { &selfv.internals.str } + "\"",
    ))
}
fn string_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(string_from(
        selfv.vm.clone(),
        unsafe { &selfv.internals.str }.to_string(),
    ))
}
fn string_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        unsafe { &selfv.internals.str } == unsafe { &other.internals.str },
    ))
}

fn string_get<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&other, unwrap_fast!(selfv.vm.types.inttp.as_ref()).clone()) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            &format!("Expected 'int' index, got '{}'", other.tp.typename),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    //NEGATIVE INDEX IS CONVERTED TO +
    let out = UnicodeSegmentation::graphemes(unsafe { &selfv.internals.str }.as_str(), true)
        .nth(unsafe { other.internals.int }.unsigned_abs());

    if out.is_none() {
        let exc = valueexc_from_str(
            selfv.vm.clone(),
            &format!(
                "Index out of range: maximum index is '{}', but got '{}'",
                unsafe { &selfv.internals.str }.len(),
                unsafe { &other.internals.int }.unsigned_abs()
            ),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }
    MethodValue::Some(string_from(selfv.vm.clone(), unwrap_fast!(out).to_string()))
}
fn string_len(selfv: Object<'_>) -> MethodType<'_> {
    let convert = unsafe { &selfv.internals.str }.len().try_into();
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), unwrap_fast!(convert)))
}

#[inline]
fn string_hash(selfv: Object<'_>) -> MethodType<'_> {
    //Use DefaultHasher for long data:
    //https://www.reddit.com/r/rust/comments/hsbai0/default_hasher_for_u8_unexpectedly_expensive/
    //jschievink: ...DefaultHasher is an implementation of SipHash...   ...pretty fast on long data, for short data this hash tends to be very slow ...
    //Use bytes[0] + bytes[len-1] + len for len > 1, bytes[0] for len==1, 0 for len==0

    let bytes = unsafe { &selfv.internals.str }[..].as_bytes();

    if bytes.len() > MFBH_MAX_LEN {
        let mut hasher = DefaultHasher::new();
        unsafe { &selfv.internals.str }.hash(&mut hasher);
        return MethodValue::Some(intobject::int_from(
            selfv.vm.clone(),
            hasher.finish() as isize,
        ));
    }

    let len = bytes.len() as isize;
    if len == 0 {
        return MethodValue::Some(intobject::int_from(selfv.vm.clone(), 0));
    } else if len == 1 {
        return MethodValue::Some(intobject::int_from(selfv.vm.clone(), bytes[0] as isize));
    }

    let res = bytes[0] as isize + bytes[bytes.len() - 1] as isize;

    MethodValue::Some(intobject::int_from(selfv.vm.clone(), res + len))
}

pub fn init(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("str"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(string_new),
        del: Some(|mut selfv| unsafe { ManuallyDrop::drop(&mut selfv.internals.str) }),

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
    vm.types.n_types += 1;

    finalize_type(tp.clone());
    finalize_type_dict(tp);
}
