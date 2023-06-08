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
    ObjectInternals, RawObject,
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
        selfv.vm.clone(),
        selfv
            .internals
            .get_bool()
            .expect("Expected bool internal value")
            .to_string(),
    ))
}
fn bool_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
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
        selfv.vm.clone(),
        *selfv
            .internals
            .get_bool()
            .expect("Expected bool internal value") as i128,
    ))
}

pub fn generate_cache<'a>(booltp: Object<'a>, tup: *mut (Option<Object<'a>>, Option<Object<'a>>)) {
    unsafe {
        let mut tp = create_object_from_type(booltp.clone());
        (*tp).internals = ObjectInternals::Bool(false);
        let ptr = &(*tup).0 as *const Option<Object> as *mut Option<Object>;
        std::ptr::write(ptr, Some(tp));

        let mut tp = create_object_from_type(booltp.clone());
        (*tp).internals = ObjectInternals::Bool(true);
        let ptr = &(*tup).1 as *const Option<Object> as *mut Option<Object>;
        std::ptr::write(ptr, Some(tp));
    }
}

pub fn init<'a>(vm: Trc<VM<'a>>) {
    let tp: Trc<RawObject<'a>> = Trc::new(RawObject {
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("bool"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
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

    VM::add_type(vm.clone(), &tp.clone().typename, tp.clone());

    finalize_type(tp);
}
