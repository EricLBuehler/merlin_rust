use std::mem::ManuallyDrop;

use super::exceptionobject::valueexc_from_str;
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

pub fn list_from<'a>(vm: Trc<VM<'a>>, raw: Vec<Object<'a>>) -> Object<'a> {
    let mut tp = create_object_from_type(unwrap_fast!(vm.types.listtp.as_ref()).clone(), vm, None);
    tp.internals = ObjectInternals {
        arr: ManuallyDrop::new(raw),
    };
    tp
}

fn list_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn list_repr(selfv: Object<'_>) -> MethodType<'_> {
    let mut res = String::from("[");
    for item in unsafe { &selfv.internals.arr }.iter() {
        let repr = RawObject::object_repr_safe(item.clone());
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
    res += "]";
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), res))
}
fn list_str(selfv: Object<'_>) -> MethodType<'_> {
    let mut res = String::from("[");
    for item in unsafe { &selfv.internals.arr }.iter() {
        let repr = RawObject::object_str_safe(item.clone());
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
    res += "]";
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), res))
}

fn list_get<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
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
    let out = unsafe { &selfv.internals.arr }.get((unsafe { other.internals.int }).unsigned_abs());

    if out.is_none() {
        let exc = valueexc_from_str(
            selfv.vm.clone(),
            &format!(
                "Index out of range: maximum index is '{}', but got '{}'",
                unsafe { &selfv.internals.arr }.len(),
                unsafe { &other.internals.int }.unsigned_abs()
            ),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }
    MethodValue::Some(unwrap_fast!(out).clone())
}
fn list_set<'a>(mut selfv: Object<'a>, other: Object<'a>, value: Object<'a>) -> MethodType<'a> {
    if is_type_exact!(&other, unwrap_fast!(selfv.vm.types.inttp.as_ref()).clone()) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            &format!("Expected 'int' index, got '{}'", other.tp.typename),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    //NEGATIVE INDEX IS CONVERTED TO +
    if unsafe { other.internals.int }.unsigned_abs() >= unsafe { &selfv.internals.arr }.len() {
        let exc = valueexc_from_str(
            selfv.vm.clone(),
            &format!(
                "Index out of range: maximum index is '{}', but got '{}'",
                unsafe { &selfv.internals.arr }.len(),
                unsafe { &other.internals.int }.unsigned_abs()
            ),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    let mut arr = unsafe { &selfv.internals.arr }.clone();
    arr[unsafe { other.internals.int }.unsigned_abs()] = value;

    selfv.internals = ObjectInternals { arr };

    MethodValue::Some(none_from!(selfv.vm.clone()))
}
fn list_len(selfv: Object<'_>) -> MethodType<'_> {
    let convert = unsafe { &selfv.internals.arr }.len().try_into();
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), unwrap_fast!(convert)))
}

#[allow(unused_unsafe)]
fn list_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }

    if unsafe { &selfv.internals.arr }.len() != unsafe { &other.internals.arr }.len() {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }
    for idx in 0..unsafe { &selfv.internals.arr }.len() {
        if unwrap_fast!(&selfv.internals.arr.get(idx)).tp.eq.is_none() {
            let exc = methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                "Method 'eq' is not defined for value",
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }
        let v = unwrap_fast!(&selfv.internals.arr.get(idx));
        let res = (v.tp.eq.expect("Method is not defined"))(
            v.clone(),
            unwrap_fast!(&other.internals.arr.get(idx)).clone(),
        );
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
        typename: String::from("list"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(list_new),
        del: Some(|mut selfv| unsafe { ManuallyDrop::drop(&mut selfv.internals.arr) }),

        repr: Some(list_repr),
        str: Some(list_str),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(list_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: Some(list_get),
        set: Some(list_set),
        len: Some(list_len),

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.listtp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp.clone());
    finalize_type_dict(tp);
}
