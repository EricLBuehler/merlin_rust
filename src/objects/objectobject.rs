use std::{sync::Arc};

use super::{Object, add_type, MethodValue, boolobject, stringobject, RawObject, create_object_from_type, finalize_type};


fn object_new(selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(create_object_from_type(selfv))
}
fn object_repr(_selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(stringobject::string_from("object".to_string()))
}
fn object_eq(selfv: Object, other: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(boolobject::bool_from(Arc::ptr_eq(&selfv, &other)))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Type,
        internals: super::ObjectInternals::No,
        typename: String::from("object"),
        bases: vec![super::ObjectBase::Object],

        new: Some(object_new),

        repr: Some(object_repr),
        abs: None,
        neg: None,

        eq: Some(object_eq),
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