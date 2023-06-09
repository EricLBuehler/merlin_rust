use super::{
    boolobject, create_object_from_type, finalize_type, intobject, stringobject, utils, ExcData,
    MethodType, MethodValue, Object, ObjectInternals, TypeObject,
};
use crate::is_type_exact;
use crate::trc::Trc;
use crate::{interpreter::VM, parser::Position};

fn exc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn exc_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        String::from("Exception<>"),
    ))
}
fn exc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn exc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_exc<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("Exception"),
        bases: vec![super::ObjectBase::Other(
            vm.types.objecttp.as_ref().unwrap().clone(),
        )],
        vm: vm.clone(),

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
    });

    vm.types.exctp = Some(tp.clone());

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
    let mut tp = create_object_from_type(vm.types.nameexctp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Exc(ExcData { obj, start, end });

    tp
}
pub fn nameexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.types.nameexctp.as_ref().unwrap().clone());

    tp.internals = ObjectInternals::Exc(ExcData {
        obj: stringobject::string_from(vm.clone(), raw.to_string()),
        start,
        end,
    });
    tp
}

fn nameexc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn nameexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = utils::object_str_safe(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    );
    if !repr.is_some() {
        return MethodValue::NotImplemented;
    }
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("NameExc: \"{}\"", repr.unwrap()),
    ))
}
fn nameexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    )
}
fn nameexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn nameexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_nameexc<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("NameExc"),
        bases: vec![
            super::ObjectBase::Other(vm.types.exctp.as_ref().unwrap().clone()),
            super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone()),
        ],
        vm: vm.clone(),

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
    });

    vm.types.nameexctp = Some(tp.clone());

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
    let mut tp = create_object_from_type(vm.types.overflwexctp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Exc(ExcData { obj, start, end });

    tp
}
pub fn overflowexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.types.overflwexctp.as_ref().unwrap().clone());

    tp.internals = ObjectInternals::Exc(ExcData {
        obj: stringobject::string_from(vm.clone(), raw.to_string()),
        start,
        end,
    });
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
    let repr = utils::object_str_safe(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    );
    if !repr.is_some() {
        return MethodValue::NotImplemented;
    }
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("OverflowExc: \"{}\"", repr.unwrap()),
    ))
}
fn overflowexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    )
}
fn overflowexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn overflowexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_overflowexc<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("OverflowExc"),
        bases: vec![
            super::ObjectBase::Other(vm.types.exctp.as_ref().unwrap().clone()),
            super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone()),
        ],
        vm: vm.clone(),

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
    });

    vm.types.overflwexctp = Some(tp.clone());

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
    let mut tp = create_object_from_type(vm.types.mthntfndexctp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Exc(ExcData { obj, start, end });

    tp
}
pub fn methodnotdefinedexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.types.mthntfndexctp.as_ref().unwrap().clone());

    tp.internals = ObjectInternals::Exc(ExcData {
        obj: stringobject::string_from(vm.clone(), raw.to_string()),
        start,
        end,
    });
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
    let repr = utils::object_str_safe(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    );
    if !repr.is_some() {
        return MethodValue::NotImplemented;
    }
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("MethodNotDefinedExc: \"{}\"", repr.unwrap()),
    ))
}
fn methodnotdefinedexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    )
}
fn methodnotdefinedexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn methodnotdefinedexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_methodnotdefinedexc<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("MethodNotDefinedExc"),
        bases: vec![
            super::ObjectBase::Other(vm.types.exctp.as_ref().unwrap().clone()),
            super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone()),
        ],
        vm: vm.clone(),

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
    });

    vm.types.mthntfndexctp = Some(tp.clone());

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
    let mut tp = create_object_from_type(vm.types.tpmisexctp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Exc(ExcData { obj, start, end });

    tp
}
pub fn typemismatchexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.types.tpmisexctp.as_ref().unwrap().clone());

    tp.internals = ObjectInternals::Exc(ExcData {
        obj: stringobject::string_from(vm.clone(), raw.to_string()),
        start,
        end,
    });
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
    let repr = utils::object_str_safe(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    );
    if !repr.is_some() {
        return MethodValue::NotImplemented;
    }
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("TypeMismatchExc: \"{}\"", repr.unwrap()),
    ))
}
fn typemismatchexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    )
}
fn typemismatchexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn typemismatchexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_typemismatchexc<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("TypeMismatchExc"),
        bases: vec![
            super::ObjectBase::Other(vm.types.exctp.as_ref().unwrap().clone()),
            super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone()),
        ],
        vm: vm.clone(),

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
    });

    vm.types.tpmisexctp = Some(tp.clone());

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
    let mut tp = create_object_from_type(vm.types.keyntfndexctp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Exc(ExcData { obj, start, end });

    tp
}
pub fn keynotfoundexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.types.keyntfndexctp.as_ref().unwrap().clone());

    tp.internals = ObjectInternals::Exc(ExcData {
        obj: stringobject::string_from(vm.clone(), raw.to_string()),
        start,
        end,
    });
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
    let repr = utils::object_str_safe(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    );
    if !repr.is_some() {
        return MethodValue::NotImplemented;
    }
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("KeyNotFoundExc: \"{}\"", repr.unwrap()),
    ))
}
fn keynotfoundexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    )
}
fn keynotfoundexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn keynotfoundexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_keynotfoundexc<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("KeyNotFoundExc"),
        bases: vec![
            super::ObjectBase::Other(vm.types.exctp.as_ref().unwrap().clone()),
            super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone()),
        ],
        vm: vm.clone(),

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
    });

    vm.types.keyntfndexctp = Some(tp.clone());

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
    let mut tp = create_object_from_type(vm.types.valueexctp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Exc(ExcData { obj, start, end });

    tp
}
pub fn valueexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.types.valueexctp.as_ref().unwrap().clone());

    tp.internals = ObjectInternals::Exc(ExcData {
        obj: stringobject::string_from(vm.clone(), raw.to_string()),
        start,
        end,
    });
    tp
}

fn valueexc_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn valueexc_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = utils::object_str_safe(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    );
    if !repr.is_some() {
        return MethodValue::NotImplemented;
    }
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("ValueExc: \"{}\"", repr.unwrap()),
    ))
}
fn valueexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    )
}
fn valueexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn valueexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_valueexc<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("ValueExc"),
        bases: vec![
            super::ObjectBase::Other(vm.types.exctp.as_ref().unwrap().clone()),
            super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone()),
        ],
        vm: vm.clone(),

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
    });

    vm.types.valueexctp = Some(tp.clone());

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
    let mut tp = create_object_from_type(vm.types.divzeroexctp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Exc(ExcData { obj, start, end });

    tp
}
pub fn zerodivexc_from_str<'a>(
    vm: Trc<VM<'a>>,
    raw: &str,
    start: Position,
    end: Position,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.types.divzeroexctp.as_ref().unwrap().clone());

    tp.internals = ObjectInternals::Exc(ExcData {
        obj: stringobject::string_from(vm.clone(), raw.to_string()),
        start,
        end,
    });
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
    let repr = utils::object_str_safe(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    );
    if !repr.is_some() {
        return MethodValue::NotImplemented;
    }
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("DivisionByZeroExc: \"{}\"", repr.unwrap()),
    ))
}
fn zerodivexc_str(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(
        selfv
            .internals
            .get_exc()
            .expect("Expected exc internal value")
            .obj,
    )
}
fn zerodivexc_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn zerodivexc_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn init_zerodivexc<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("DivisionByZeroExc"),
        bases: vec![
            super::ObjectBase::Other(vm.types.exctp.as_ref().unwrap().clone()),
            super::ObjectBase::Other(vm.types.objecttp.as_ref().unwrap().clone()),
        ],
        vm: vm.clone(),

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
    });

    vm.types.divzeroexctp = Some(tp.clone());

    finalize_type(tp);
}
