use super::{
    boolobject, create_object_from_type, finalize_type, finalize_type_dict, intobject, MethodType,
    MethodValue, Object, ObjectInternals, TypeObject,
};
use crate::{interpreter::VM, objects::stringobject};
use crate::{is_type_exact, unwrap_fast};
use trc::Trc;

#[macro_export]
macro_rules! none_from {
    ($vm:expr) => {
        unwrap_fast!($vm.cache.none_singleton.as_ref()).clone()
    };
}

fn none_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn none_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        String::from("None"),
    ))
}
fn none_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), -2))
}
fn none_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn generate_cache<'a>(
    vm: Trc<VM<'a>>,
    nonetp: Trc<TypeObject<'a>>,
    ptr: *mut Option<Object<'a>>,
) {
    unsafe {
        let mut tp = create_object_from_type(nonetp.clone(), vm, None);
        tp.internals = ObjectInternals { none: () };
        std::ptr::write(ptr, Some(tp));
    }
}

pub fn init(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("NoneType"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: None,

        new: Some(none_new),
        del: Some(|_| {}),

        repr: Some(none_repr),
        str: Some(none_repr),
        abs: None,
        neg: None,
        hash_fn: Some(none_hash),

        eq: Some(none_eq),
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

    vm.types.nonetp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp.clone());
    finalize_type_dict(tp);
}
