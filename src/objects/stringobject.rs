use std::{sync::Arc};

use super::{RawObject, Object, get_type, add_type, MethodValue, ObjectInternals, create_object_from_type, finalize_type};


pub fn string_from(raw: String) -> Object {
    let mut tp = create_object_from_type(get_type("str"));
    let mut refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Str(raw);
    tp
}

fn string_new(_selfv: Object, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
    unimplemented!();
}
fn string_repr(selfv: Object) -> MethodValue<Object, Object> {
    MethodValue::Some(selfv)
}

pub fn init(){
    let tp: Arc<RawObject> = Arc::new( RawObject{
        tp: super::ObjectType::Other(get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("str"),
        bases: vec![super::ObjectBase::Other(get_type("object"))],

        new: Some(string_new),

        repr: Some(string_repr),
        abs: None,
        neg: None,

        eq: None,
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,
    });

    add_type(&tp.clone().typename, tp.clone());

    finalize_type(tp);
}