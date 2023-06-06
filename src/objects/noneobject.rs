use super::{
    boolobject, create_object_from_type, finalize_type, intobject, MethodType,
    MethodValue, Object, ObjectInternals, RawObject,
};
use crate::{Arc, is_type_exact};
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
        is_type_exact!(&selfv, &other),
    ))
}

pub fn generate_cache<'a>(nonetp: Object<'a>, ptr: *mut Option<Object<'a>>) {
    unsafe {
        let mut tp = create_object_from_type(nonetp.clone());
        let refr = Arc::make_mut(&mut tp);
        refr.internals = ObjectInternals::None;
        std::ptr::write(ptr, Some(tp));
    }
}

pub fn init<'a>(vm: Arc<VM<'a>>) {
    let tp: Arc<RawObject<'a>> = Arc::new(RawObject {
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("NoneType"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
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

    VM::add_type(vm.clone(), &tp.clone().typename, tp.clone());

    finalize_type(tp);
}
