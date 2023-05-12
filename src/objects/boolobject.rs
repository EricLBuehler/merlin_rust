use std::{sync::Arc};
use crate::objects::stringobject::StringObject;
use crate::objects::listobject::ListObject;

use super::{Object, ObjectTrait, get_type, add_type, MethodValue, ObjectInternals};

#[derive(Clone)]
pub struct BoolType {
    tp: Object,
}

impl ObjectTrait for BoolType {
    fn get_name(self: Arc<Self>) -> String {
        return String::from("bool");
    }
    fn get_type(self: Arc<Self>) -> Object {
        return self.tp.clone();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        return ListObject::from(vec![get_type("type")]);
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        return MethodValue::Some(StringObject::from("<class 'bool'>".to_string()));
    }
}

impl BoolType {
    pub fn init(){
        let tp = Arc::new(BoolType{tp: get_type("type")});
        add_type("bool", tp);
    }
}


#[derive(Clone)]
pub struct BoolObject {
    tp: Object,
    value: bool,
}

impl BoolObject {
    pub fn from(value: bool) -> Object {
        return Arc::new(BoolObject { tp: get_type("bool"), value});
    }
}

impl ObjectTrait for BoolObject {
    fn get_name(self: Arc<Self>) -> String {
        let strong = self.tp.clone();
        return strong.get_name();
    }
    fn get_raw(self: Arc<Self>) -> MethodValue<ObjectInternals, Object> {
        return MethodValue::Some(ObjectInternals::Bool(self.value));
    }
    fn get_type(self: Arc<Self>) -> Object {
        return self.tp.clone();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        let strong = self.tp.clone();
        return strong.get_bases();
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        return MethodValue::Some(StringObject::from(self.value.to_string()));
    }
}