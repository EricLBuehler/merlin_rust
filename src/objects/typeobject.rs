use std::{sync::Arc};

use super::{Object, add_type, MethodValue, boolobject, stringobject, RawObject, get_type, get_typeid, create_object_from_type, finalize_type, intobject};


fn type_new(selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(create_object_from_type(selfv))
}
fn type_repr(selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(stringobject::string_from(format!("<class '{}'>", selfv.typename)))
}
fn type_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(boolobject::bool_from(get_typeid(selfv) == get_typeid(other)))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Type,
        internals: super::ObjectInternals::No,
        typename: String::from("type"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(type_new),

        repr: Some(type_repr),
        abs: None,
        neg: None,
        hash_fn: Some(|_: Object| { MethodValue::Some(intobject::int_from(-3)) }),

        eq: Some(type_eq),
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