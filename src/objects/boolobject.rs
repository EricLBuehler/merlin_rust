use crate::is_type_exact;
use crate::objects::exceptionobject::typemismatchexc_from_str;
use crate::parser::Position;
use crate::trc::Trc;
use crate::{
    interpreter::VM,
    objects::{boolobject, stringobject},
};

use super::{
    create_object_from_type, finalize_type, intobject, MethodType, MethodValue, Object,
    ObjectInternals, RawObject, TypeObject,
};

pub fn bool_from(vm: Trc<VM<'_>>, raw: bool) -> Object<'_> {
    match raw {
        false => vm.cache.bool_cache.0.as_ref().unwrap().clone(),
        true => vm.cache.bool_cache.1.as_ref().unwrap().clone(),
    }
}

fn bool_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn bool_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        selfv
            .internals
            .get_bool()
            .expect("Expected bool internal value")
            .to_string(),
    ))
}
fn bool_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.tp.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        selfv
            .internals
            .get_bool()
            .expect("Expected bool internal value")
            == other
                .internals
                .get_bool()
                .expect("Expected bool internal value"),
    ))
}
fn bool_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(
        selfv.tp.vm.clone(),
        *selfv
            .internals
            .get_bool()
            .expect("Expected bool internal value") as i128,
    ))
}

pub fn generate_cache<'a>(booltp: Trc<TypeObject<'a>>, tup: *mut (Option<Object<'a>>, Option<Object<'a>>)) {
    unsafe {
        let mut tp = create_object_from_type(booltp.clone());
        tp.internals = ObjectInternals::Bool(false);
        let ptr = &(*tup).0 as *const Option<Object> as *mut Option<Object>;
        std::ptr::write(ptr, Some(tp));

        let mut tp = create_object_from_type(booltp.clone());
        tp.internals = ObjectInternals::Bool(true);
        let ptr = &(*tup).1 as *const Option<Object> as *mut Option<Object>;
        std::ptr::write(ptr, Some(tp));
    }
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("bool"),
        bases: vec![super::ObjectBase::Other(
            vm.types.objecttp.as_ref().unwrap().clone(),
        )],
        vm: vm.clone(),

        new: Some(bool_new),

        repr: Some(bool_repr),
        str: Some(bool_repr),
        abs: None,
        neg: None,
        hash_fn: Some(bool_hash),

        eq: Some(bool_eq),
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

    vm.types.booltp = Some(tp.clone());

    finalize_type(tp);
}
