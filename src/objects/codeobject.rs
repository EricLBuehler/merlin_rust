use std::rc::Rc;
use crate::{objects::{stringobject, ObjectInternals, boolobject}, compiler::Bytecode, interpreter::VM};

use super::{RawObject, Object,MethodType, MethodValue, finalize_type, is_instance, create_object_from_type};


pub fn code_from<'a>(vm: Rc<VM<'a>>, bytecode: Rc<Bytecode<'a>>) -> Object<'a> {
    let mut tp: std::rc::Rc<RawObject> = create_object_from_type(vm.get_type("code"));
    let mut refr = Rc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Code(bytecode);
    tp
}

fn code_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn code_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), format!("<code object @ 0x{:x}>", Rc::as_ptr(&selfv) as i128)))
}
fn code_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), selfv.internals.get_code().expect("Expected Bytecode internal value") == other.internals.get_code().expect("Expected Bytecode internal value")))
}

pub fn init<'a>(vm: Rc<VM<'a>>){
    let tp: Rc<RawObject<'a>> = Rc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("code"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(code_new),

        repr: Some(code_repr),
        str: Some(code_repr),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(code_eq),
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

    vm.clone().add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}