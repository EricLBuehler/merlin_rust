use crate::Arc;
use crate::{interpreter::VM};

use super::{Object, MethodValue, MethodType, boolobject, stringobject, RawObject, create_object_from_type, finalize_type, intobject};


fn object_new<'a>(selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(create_object_from_type(selfv))
}
fn object_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), "object".to_string()))
}
fn object_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), Arc::ptr_eq(&selfv, &other)))
}
fn object_hash(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), -1))
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Type(vm.clone()),
        internals: super::ObjectInternals::No,
        typename: String::from("object"),
        bases: vec![super::ObjectBase::Object(vm.clone())],
        vm: vm.clone(),

        new: Some(object_new),

        repr: Some(object_repr),
        str: Some(object_repr),
        abs: None,
        neg: None,
        hash_fn: Some(object_hash),

        eq: Some(object_eq),
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