use std::{sync::Arc};
use crate::objects::stringobject::StringObject;

use super::{Object, ObjectTrait, get_type, add_type, MethodValue, boolobject::BoolObject};


#[derive(Clone)]
pub struct TypeType {
}

impl ObjectTrait for TypeType {
    fn get_name(self: Arc<Self>) -> String {
        String::from("type")
    }
    fn get_type(self: Arc<Self>) -> Object {
        self
    }
    fn get_bases(self: Arc<Self>) -> Vec<Object> {
        vec![get_type("types")]
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        MethodValue::Some(StringObject::from("<class 'type'>".to_string()))
    }
    fn eq(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> {
        MethodValue::Some(BoolObject::from(self.get_typeid() == other.get_typeid()))
    }
}

impl TypeType {
    pub fn init(){
        let tp = Arc::new(TypeType{});
        add_type("type", tp);
    }
}
