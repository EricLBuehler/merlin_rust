use super::{
    boolobject, create_object_from_type, finalize_type, intobject, MethodType, MethodValue, Object,
    ObjectInternals, TypeObject,
};
use crate::is_type_exact;
use crate::trc::Trc;
use crate::{interpreter::VM, objects::stringobject};

#[macro_export]
macro_rules! none_from {
    ($vm:expr) => {
        $vm.cache.none_singleton.as_ref().unwrap().clone()
    };
}

fn none_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn none_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        String::from("None"),
    ))
}
fn none_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.tp.vm.clone(), -2))
}
fn none_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
        is_type_exact!(&selfv, other.tp),
    ))
}

pub fn generate_cache<'a>(nonetp: Trc<TypeObject<'a>>, ptr: *mut Option<Object<'a>>) {
    unsafe {
        let mut tp = create_object_from_type(nonetp.clone());
        tp.internals = ObjectInternals::None;
        std::ptr::write(ptr, Some(tp));
    }
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("NoneType"),
        bases: vec![super::ObjectBase::Other(
            vm.types.objecttp.as_ref().unwrap().clone(),
        )],
        vm: vm.clone(),

        new: Some(none_new),

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
    });

    vm.types.nonetp = Some(tp.clone());

    finalize_type(tp);
}
