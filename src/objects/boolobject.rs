use std::{sync::Arc};
use crate::objects::stringobject;

use super::{RawObject, Object, get_type, add_type, MethodValue, ObjectInternals, create_object_from_type};


pub fn bool_from(raw: bool) -> Object {
    let mut tp = create_object_from_type(get_type("bool"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Bool(raw);
    tp
}

fn bool_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn bool_repr(selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(stringobject::string_from(selfv.internals.get_bool().unwrap().to_string()))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("bool"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(bool_new),

        repr: Some(bool_repr),
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