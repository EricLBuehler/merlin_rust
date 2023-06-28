use std::mem::ManuallyDrop;

use super::{
    boolobject, create_object_from_type, finalize_type, intobject, stringobject, ExcData,
    MethodType, MethodValue, Object, ObjectInternals, RawObject, TypeObject,
};
use crate::is_type_exact;
use crate::unwrap_fast;
use crate::{interpreter::VM, parser::Position};
use trc::Trc;

fn exc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn exc_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        String::from("Exception<>"),
    ))
}
fn exc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn exc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_exc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("Exception"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(exc_new),

        repr: Some(exc_repr),
        str: Some(exc_repr),
        abs: None,
        neg: None,
        hash_fn: Some(exc_hash),

        eq: Some(exc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.exctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}

// =====================

#[allow(dead_code)]
pub fn nameexc_from_obj<'a>(
    vm: Trc<VM<'a>>,
    obj: Object<'a>,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.nameexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );
    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData { obj, start, end }),
    };

    tp
}
pub fn nameexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.nameexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );

    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData {
            obj: stringobject::string_from(vm.clone(), raw.to_string()),
            start,
            end,
        }),
    };
    tp
}

fn nameexc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn nameexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = RawObject::object_str_safe(unsafe { &selfv.internals.exc }.obj.clone());
    if repr.is_error() {
        return MethodValue::Error(repr.unwrap_err());
    }
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("NameExc: \"{}\"", unwrap_fast!(repr)),
    ))
}
fn nameexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(unsafe { &selfv.internals.exc }.obj.clone())
}
fn nameexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn nameexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_nameexc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("NameExc"),
        bases: vec![
            super::ObjectBase::Other(unwrap_fast!(vm.types.exctp.as_ref()).clone()),
            super::ObjectBase::Other(unwrap_fast!(vm.types.objecttp.as_ref()).clone()),
        ],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(nameexc_new),

        repr: Some(nameexc_repr),
        str: Some(nameexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(nameexc_hash),

        eq: Some(nameexc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.nameexctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}

// =====================

#[allow(dead_code)]
pub fn overflowexc_from_obj<'a>(
    vm: Trc<VM<'a>>,
    obj: Object<'a>,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.overflwexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );
    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData { obj, start, end }),
    };

    tp
}
pub fn overflowexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.overflwexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );

    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData {
            obj: stringobject::string_from(vm.clone(), raw.to_string()),
            start,
            end,
        }),
    };
    tp
}

fn overflowexc_new<'a>(
    _selfv: Object<'a>,
    _args: Object<'a>,
    _kwargs: Object<'a>,
) -> MethodType<'a> {
    unimplemented!();
}
fn overflowexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = RawObject::object_str_safe(unsafe { &selfv.internals.exc }.obj.clone());

    if repr.is_error() {
        return MethodValue::Error(repr.unwrap_err());
    }
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("OverflowExc: \"{}\"", unwrap_fast!(repr)),
    ))
}
fn overflowexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(unsafe { &selfv.internals.exc }.obj.clone())
}
fn overflowexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn overflowexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_overflowexc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("OverflowExc"),
        bases: vec![
            super::ObjectBase::Other(unwrap_fast!(vm.types.exctp.as_ref()).clone()),
            super::ObjectBase::Other(unwrap_fast!(vm.types.objecttp.as_ref()).clone()),
        ],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(overflowexc_new),

        repr: Some(overflowexc_repr),
        str: Some(overflowexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(overflowexc_hash),

        eq: Some(overflowexc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.overflwexctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}

// =====================

#[allow(dead_code)]
pub fn methodnotdefinedexc_from_obj<'a>(
    vm: Trc<VM<'a>>,
    obj: Object<'a>,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.mthntfndexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );
    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData { obj, start, end }),
    };

    tp
}
pub fn methodnotdefinedexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.mthntfndexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );

    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData {
            obj: stringobject::string_from(vm.clone(), raw.to_string()),
            start,
            end,
        }),
    };
    tp
}

fn methodnotdefinedexc_new<'a>(
    _selfv: Object<'a>,
    _args: Object<'a>,
    _kwargs: Object<'a>,
) -> MethodType<'a> {
    unimplemented!();
}
fn methodnotdefinedexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = RawObject::object_str_safe(unsafe { &selfv.internals.exc }.obj.clone());

    if repr.is_error() {
        return MethodValue::Error(repr.unwrap_err());
    }
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("MethodNotDefinedExc: \"{}\"", unwrap_fast!(repr)),
    ))
}
fn methodnotdefinedexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(unsafe { &selfv.internals.exc }.obj.clone())
}
fn methodnotdefinedexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn methodnotdefinedexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_methodnotdefinedexc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("MethodNotDefinedExc"),
        bases: vec![
            super::ObjectBase::Other(unwrap_fast!(vm.types.exctp.as_ref()).clone()),
            super::ObjectBase::Other(unwrap_fast!(vm.types.objecttp.as_ref()).clone()),
        ],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(methodnotdefinedexc_new),

        repr: Some(methodnotdefinedexc_repr),
        str: Some(methodnotdefinedexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(methodnotdefinedexc_hash),

        eq: Some(methodnotdefinedexc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.mthntfndexctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}

// =====================

#[allow(dead_code)]
pub fn typemismatchexc_from_obj<'a>(
    vm: Trc<VM<'a>>,
    obj: Object<'a>,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.tpmisexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );
    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData { obj, start, end }),
    };

    tp
}
pub fn typemismatchexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.tpmisexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );

    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData {
            obj: stringobject::string_from(vm.clone(), raw.to_string()),
            start,
            end,
        }),
    };
    tp
}

fn typemismatchexc_new<'a>(
    _selfv: Object<'a>,
    _args: Object<'a>,
    _kwargs: Object<'a>,
) -> MethodType<'a> {
    unimplemented!();
}
fn typemismatchexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = RawObject::object_str_safe(unsafe { &selfv.internals.exc }.obj.clone());

    if repr.is_error() {
        return MethodValue::Error(repr.unwrap_err());
    }
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("TypeMismatchExc: \"{}\"", unwrap_fast!(repr)),
    ))
}
fn typemismatchexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(unsafe { &selfv.internals.exc }.obj.clone())
}
fn typemismatchexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn typemismatchexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_typemismatchexc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("TypeMismatchExc"),
        bases: vec![
            super::ObjectBase::Other(unwrap_fast!(vm.types.exctp.as_ref()).clone()),
            super::ObjectBase::Other(unwrap_fast!(vm.types.objecttp.as_ref()).clone()),
        ],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(typemismatchexc_new),

        repr: Some(typemismatchexc_repr),
        str: Some(typemismatchexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(typemismatchexc_hash),

        eq: Some(typemismatchexc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.tpmisexctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}

// =====================

#[allow(dead_code)]
pub fn keynotfoundexc_from_obj<'a>(
    vm: Trc<VM<'a>>,
    obj: Object<'a>,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.keyntfndexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );
    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData { obj, start, end }),
    };

    tp
}
pub fn keynotfoundexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.keyntfndexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );

    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData {
            obj: stringobject::string_from(vm.clone(), raw.to_string()),
            start,
            end,
        }),
    };
    tp
}

fn keynotfoundexc_new<'a>(
    _selfv: Object<'a>,
    _args: Object<'a>,
    _kwargs: Object<'a>,
) -> MethodType<'a> {
    unimplemented!();
}
fn keynotfoundexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = RawObject::object_str_safe(unsafe { &selfv.internals.exc }.obj.clone());

    if repr.is_error() {
        return MethodValue::Error(repr.unwrap_err());
    }
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("KeyNotFoundExc: \"{}\"", unwrap_fast!(repr)),
    ))
}
fn keynotfoundexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(unsafe { &selfv.internals.exc }.obj.clone())
}
fn keynotfoundexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn keynotfoundexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_keynotfoundexc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("KeyNotFoundExc"),
        bases: vec![
            super::ObjectBase::Other(unwrap_fast!(vm.types.exctp.as_ref()).clone()),
            super::ObjectBase::Other(unwrap_fast!(vm.types.objecttp.as_ref()).clone()),
        ],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(keynotfoundexc_new),

        repr: Some(keynotfoundexc_repr),
        str: Some(keynotfoundexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(keynotfoundexc_hash),

        eq: Some(keynotfoundexc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.keyntfndexctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}

// =====================

#[allow(dead_code)]
pub fn valueexc_from_obj<'a>(
    vm: Trc<VM<'a>>,
    obj: Object<'a>,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.valueexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );
    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData { obj, start, end }),
    };

    tp
}
pub fn valueexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.valueexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );

    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData {
            obj: stringobject::string_from(vm.clone(), raw.to_string()),
            start,
            end,
        }),
    };
    tp
}

fn valueexc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn valueexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = RawObject::object_str_safe(unsafe { &selfv.internals.exc }.obj.clone());

    if repr.is_error() {
        return MethodValue::Error(repr.unwrap_err());
    }
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("ValueExc: \"{}\"", unwrap_fast!(repr)),
    ))
}
fn valueexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(unsafe { &selfv.internals.exc }.obj.clone())
}
fn valueexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn valueexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_valueexc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("ValueExc"),
        bases: vec![
            super::ObjectBase::Other(unwrap_fast!(vm.types.exctp.as_ref()).clone()),
            super::ObjectBase::Other(unwrap_fast!(vm.types.objecttp.as_ref()).clone()),
        ],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(valueexc_new),

        repr: Some(valueexc_repr),
        str: Some(valueexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(valueexc_hash),

        eq: Some(valueexc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.valueexctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}

// =====================

#[allow(dead_code)]
pub fn zerodivexc_from_obj<'a>(
    vm: Trc<VM<'a>>,
    obj: Object<'a>,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.divzeroexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );
    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData { obj, start, end }),
    };

    tp
}
pub fn zerodivexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.divzeroexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );

    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData {
            obj: stringobject::string_from(vm.clone(), raw.to_string()),
            start,
            end,
        }),
    };
    tp
}

fn zerodivexc_new<'a>(
    _selfv: Object<'a>,
    _args: Object<'a>,
    _kwargs: Object<'a>,
) -> MethodType<'a> {
    unimplemented!();
}
fn zerodivexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = RawObject::object_str_safe(unsafe { &selfv.internals.exc }.obj.clone());

    if repr.is_error() {
        return MethodValue::Error(repr.unwrap_err());
    }
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("DivisionByZeroExc: \"{}\"", unwrap_fast!(repr)),
    ))
}
fn zerodivexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(unsafe { &selfv.internals.exc }.obj.clone())
}
fn zerodivexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn zerodivexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_zerodivexc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("DivisionByZeroExc"),
        bases: vec![
            super::ObjectBase::Other(unwrap_fast!(vm.types.exctp.as_ref()).clone()),
            super::ObjectBase::Other(unwrap_fast!(vm.types.objecttp.as_ref()).clone()),
        ],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(zerodivexc_new),

        repr: Some(zerodivexc_repr),
        str: Some(zerodivexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(zerodivexc_hash),

        eq: Some(zerodivexc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.divzeroexctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}

// =====================

#[allow(dead_code)]
pub fn attrexc_from_obj<'a>(
    vm: Trc<VM<'a>>,
    obj: Object<'a>,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.attrexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );
    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData { obj, start, end }),
    };

    tp
}
pub fn attrexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(
        unwrap_fast!(vm.types.attrexctp.as_ref()).clone(),
        vm.clone(),
        None,
    );

    tp.internals = ObjectInternals {
        exc: ManuallyDrop::new(ExcData {
            obj: stringobject::string_from(vm.clone(), raw.to_string()),
            start,
            end,
        }),
    };
    tp
}

fn attrexc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn attrexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = RawObject::object_str_safe(unsafe { &selfv.internals.exc }.obj.clone());

    if repr.is_error() {
        return MethodValue::Error(repr.unwrap_err());
    }
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("AttributeExc: \"{}\"", unwrap_fast!(repr)),
    ))
}
fn attrexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(unsafe { &selfv.internals.exc }.obj.clone())
}
fn attrexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.vm.clone(),
        (-(selfv.tp.typeid as i32) - 10) as isize,
    ))
}
fn attrexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_attrexc(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("AttributeExc"),
        bases: vec![
            super::ObjectBase::Other(unwrap_fast!(vm.types.exctp.as_ref()).clone()),
            super::ObjectBase::Other(unwrap_fast!(vm.types.objecttp.as_ref()).clone()),
        ],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(attrexc_new),

        repr: Some(attrexc_repr),
        str: Some(attrexc_str),
        abs: None,
        neg: None,
        hash_fn: Some(attrexc_hash),

        eq: Some(attrexc_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,

        getattr: None,
        setattr: None,
        descrget: None,
        descrset: None,
    });

    vm.types.attrexctp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}
