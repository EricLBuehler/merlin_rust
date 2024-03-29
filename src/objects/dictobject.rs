use std::mem::ManuallyDrop;

use super::mhash::HashMap;
use super::{
    create_object_from_type, finalize_type, finalize_type_dict, intobject, MethodType, MethodValue,
    Object, RawObject, TypeObject,
};

use crate::is_type_exact;
use crate::objects::exceptionobject::{methodnotdefinedexc_from_str, typemismatchexc_from_str};
use crate::parser::Position;
use crate::unwrap_fast;
use crate::{
    interpreter::VM,
    objects::{boolobject, stringobject, ObjectInternals},
};
use trc::Trc;

#[allow(dead_code)]
pub fn dict_from<'a>(vm: Trc<VM<'a>>, raw: HashMap<'a>) -> Object<'a> {
    let mut tp = create_object_from_type(unwrap_fast!(vm.types.dicttp.as_ref()).clone(), vm, None);
    tp.internals = ObjectInternals {
        map: ManuallyDrop::new(raw),
    };
    tp
}

fn dict_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn dict_repr(selfv: Object<'_>) -> MethodType<'_> {
    let mut res = String::from("{");
    let sf = selfv.clone();
    let map = unsafe { &sf.internals.map }.clone();
    for (key, value) in map.into_iter() {
        let repr = RawObject::object_repr_safe(key);
        if repr.is_error() {
            return MethodValue::Error(repr.unwrap_err());
        }
        res += &unwrap_fast!(repr);
        res += ": ";
        let repr = RawObject::object_repr_safe(value);
        if !repr.is_some() {
            return MethodValue::Error(repr.unwrap_err());
        }
        res += &unwrap_fast!(repr);
        res += ", ";
    }
    if res.len() > 1 {
        res.pop();
        res.pop();
    }
    res += "}";
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), res))
}

fn dict_str(selfv: Object<'_>) -> MethodType<'_> {
    let mut res = String::from("{");
    let sf = selfv.clone();
    let map = unsafe { &sf.internals.map }.clone();
    for (key, value) in map.into_iter() {
        let repr = RawObject::object_str_safe(key);
        if repr.is_error() {
            return MethodValue::Error(repr.unwrap_err());
        }
        res += &unwrap_fast!(repr);
        res += ": ";
        let repr = RawObject::object_str_safe(value);
        if !repr.is_some() {
            return MethodValue::Error(repr.unwrap_err());
        }
        res += &unwrap_fast!(repr);
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
    //NEGATIVE INDEX IS CONVERTED TO +
    let out = unsafe { &selfv.internals.map }.get(other);

    if out.is_error() {
        return MethodValue::Error(out.unwrap_err());
    }
    MethodValue::Some(unwrap_fast!(out).clone())
}

#[inline]
fn dict_set<'a>(mut selfv: Object<'a>, other: Object<'a>, value: Object<'a>) -> MethodType<'a> {
    //TODO check for hash here!
    let mut map = unsafe { &selfv.internals.map }.clone();
    let res = map.insert(other, value);
    if res.is_error() {
        return MethodValue::Error(res.unwrap_err());
    }

    selfv.internals = ObjectInternals { map };

    MethodValue::Some(none_from!(selfv.vm))
}
fn dict_len(selfv: Object<'_>) -> MethodType<'_> {
    let convert = unsafe { &selfv.internals.map }.len().try_into();

    MethodValue::Some(intobject::int_from(selfv.vm.clone(), unwrap_fast!(convert)))
}

#[allow(unused_unsafe)]
fn dict_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }

    if unsafe { &selfv.internals.map }.len() != unsafe { &other.internals.map }.len() {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }
    for ((key1, value1), (key2, value2)) in std::iter::zip(
        unsafe { &selfv.internals.map }.into_iter(),
        unsafe { &other.internals.map }.into_iter(),
    ) {
        if key1.tp.eq.is_none() {
            let exc = methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'eq' is not defined for key 1 type '{}'",
                    key1.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }
        if value1.tp.eq.is_none() {
            let exc = methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'eq' is not defined for value 1 type '{}'",
                    value1.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        let res = (key1.tp.eq.expect("Method is not defined"))(key1.clone(), key2.clone());
        if res.is_error() {
            return res;
        }
        if !is_type_exact!(
            &unwrap_fast!(res),
            unwrap_fast!(selfv.vm.types.booltp.as_ref()).clone()
        ) {
            let exc = typemismatchexc_from_str(
                selfv.vm.clone(),
                "Method 'eq' did not return 'bool'",
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        if unsafe { unwrap_fast!(res).internals.bool } {
            return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
        }

        let res = (value1.tp.eq.expect("Method is not defined"))(value1.clone(), value2.clone());
        if res.is_error() {
            return res;
        }
        if !is_type_exact!(
            &unwrap_fast!(res),
            unwrap_fast!(selfv.vm.types.booltp.as_ref()).clone()
        ) {
            let exc = typemismatchexc_from_str(
                selfv.vm.clone(),
                "Method 'eq' did not return 'bool'",
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        if unsafe { unwrap_fast!(res).internals.bool } {
            return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
        }
    }
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), true))
}

pub fn init(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("dict"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(dict_new),

        repr: Some(dict_repr),
        str: Some(dict_str),
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

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.dicttp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp.clone());
    finalize_type_dict(tp);
}
