use crate::Arc;
use crate::{interpreter::VM};

use super::{Object, MethodValue, MethodType, boolobject, stringobject, RawObject, get_typeid, create_object_from_type, finalize_type, intobject};


fn type_new<'a>(selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(create_object_from_type(selfv))
}
fn type_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), format!("<class '{}'>", selfv.typename)))
}
fn type_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), get_typeid(selfv) == get_typeid(other)))
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Type(vm.clone()),
        internals: super::ObjectInternals::No,
        typename: String::from("type"),
        bases: vec![super::ObjectBase::Other(vm.clone().get_type("object"))],
        vm: vm.clone(),

        new: Some(type_new),

        repr: Some(type_repr),
        str: Some(type_repr),
        abs: None,
        neg: None,
        hash_fn: Some(|selfv: Object<'a>| { MethodValue::Some(intobject::int_from(selfv.vm.clone(), -3)) }),

        eq: Some(type_eq),
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