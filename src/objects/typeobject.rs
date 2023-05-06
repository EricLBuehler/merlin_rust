use std::{sync::Arc};
use crate::objects::stringobject::StringObject;
use crate::objects::listobject::ListObject;

use super::{Object, ObjectTrait, get_type, add_type, boolobject::BoolObject};


#[derive(Clone)]
pub struct TypeType {
}

impl ObjectTrait for TypeType {
    fn get_name(self: Arc<Self>) -> String {
        return String::from("type");
    }
    fn get_basic_repr(self: Arc<Self>) -> Option<String> {
        return None;
    }
    fn get_type(self: Arc<Self>) -> Object {
        return self.clone();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        return ListObject::from(vec![get_type("types"), get_type("object")]);
    }
    fn new(self: Arc<Self>, _args: Object, _kwargs: Object) -> Option<Object> {
        return None;
    }
    fn repr(self: Arc<Self>) -> Option<Object> {
        return Some(StringObject::from("<class 'type'>".to_string()));
    }
    fn eq(self: Arc<Self>, other: Object) -> Option<Object> {
        return Some(BoolObject::from(self.get_typeid() == other.get_typeid()));
    }
}

impl TypeType {
    pub fn init(){
        let tp = Arc::new(TypeType{});
        add_type("type", tp);
    }
}
