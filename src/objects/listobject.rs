use std::{sync::Arc};
use crate::objects::stringobject;

use super::{RawObject, Object, get_type, add_type, MethodValue, utils};


fn list_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn list_repr(selfv: Object) -> MethodValue<Object, Object> {
    let mut res = String::from("[");
    for item in selfv.internals.get_arr().unwrap() {
        let repr = utils::object_repr_safe(item);
        if !repr.is_some() {
            return MethodValue::NotImplemented;
        }
        res += &repr.unwrap();
        res += ", ";
    }
    if res.len() > 1 {
        res.pop();
        res.pop();
    }
    res += "]";
    MethodValue::Some(stringobject::string_from(res))
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("list"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(list_new),

        repr: Some(list_repr),
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