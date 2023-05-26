use std::{sync::Arc, collections::hash_map::DefaultHasher};
use unicode_segmentation::UnicodeSegmentation;
use std::hash::Hash;
use std::hash::Hasher;

use crate::interpreter::VM;
use crate::objects::{is_instance, boolobject, intobject};

use super::{RawObject, Object,MethodType, MethodValue, ObjectInternals, create_object_from_type, finalize_type};


pub fn string_from(vm: Arc<VM<'_>>, raw: String) -> Object<'_> {
    let mut tp = create_object_from_type(vm.get_type("str"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Str(raw);
    tp
}

fn string_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn string_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(string_from(selfv.vm.clone(), "\"".to_owned()+selfv.internals.get_str().unwrap()+"\""))
}
fn string_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), selfv.internals.get_str().unwrap() == other.internals.get_str().unwrap()))
}

fn string_get<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    is_instance(&other, &selfv.vm.get_type("int"));
    //NEGATIVE INDEX IS CONVERTED TO +
    let out = UnicodeSegmentation::graphemes(selfv.internals.get_str().unwrap().as_str(), true).nth((*other.internals.get_int().unwrap()).unsigned_abs() as usize);
    debug_assert!(out.is_some());
    MethodValue::Some(string_from(selfv.vm.clone(), out.unwrap().to_string()))
}
fn string_len(selfv: Object<'_>) -> MethodType<'_> {
    let convert: Result<i128, _> = selfv.internals.get_str().unwrap().len().try_into();
    debug_assert!(convert.is_ok());
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), convert.unwrap()))
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("str"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(string_new),

        repr: Some(string_repr),
        abs: None,
        neg: None,
        hash_fn: Some(|selfv: Object<'a>| {
            let mut hasher = DefaultHasher::new();
            selfv.internals.get_str().unwrap().hash(&mut hasher);
            
            MethodValue::Some(intobject::int_from(selfv.vm.clone(), hasher.finish() as i128))
        }),

        eq: Some(string_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,
        
        get: Some(string_get),
        set: None,
        len: Some(string_len),
        
        call: None,
    });

    vm.clone().add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}