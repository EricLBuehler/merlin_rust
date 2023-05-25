use std::{sync::Arc};
use crate::{objects::{stringobject, ObjectInternals, boolobject}, compiler::Bytecode};

use super::{RawObject, Object, get_type, add_type, MethodValue, finalize_type, is_instance, create_object_from_type};


pub fn code_from(bytecode: Bytecode) -> Object {
    let mut tp = create_object_from_type(get_type("code"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Code(bytecode);
    tp
}

fn code_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn code_repr(selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(stringobject::string_from(format!("<code object @ 0x{:x}>", Arc::as_ptr(&selfv) as i128)))
}
fn code_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    debug_assert!(is_instance(&selfv, &other));
    debug_assert!(selfv.internals.get_arr().unwrap().len() == other.internals.get_arr().unwrap().len());
    for idx in 0..selfv.internals.get_arr().unwrap().len() {
        debug_assert!(selfv.internals.get_arr().unwrap().get(idx).unwrap().eq.is_some());
        let v = selfv.internals.get_arr().unwrap().get(idx).unwrap();
        let res = (v.eq.unwrap())(v.clone(), other.internals.get_arr().unwrap().get(idx).unwrap().clone());
        debug_assert!(res.is_some());
        debug_assert!(is_instance(&res.unwrap(), &get_type("bool")));
        if *res.unwrap().internals.get_bool().unwrap() {
            return MethodValue::Some(boolobject::bool_from(false));
        }
    }
    MethodValue::Some(boolobject::bool_from(true))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("code"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(code_new),

        repr: Some(code_repr),
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
    });

    add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}