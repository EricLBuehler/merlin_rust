use std::{sync::Arc};
use crate::objects::stringobject;

use super::{RawObject, Object, get_type, add_type, MethodValue, create_object_from_type};

pub fn none_from() -> Object {
    create_object_from_type(get_type("NoneType"))
}

fn none_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn none_repr(_selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(stringobject::string_from(String::from("None")))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("NoneType"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(none_new),

        repr: Some(none_repr),
        abs: None,
        neg: None,

        eq: None,
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,
    });

    add_type(&tp.clone().typename, tp);
}