use std::{sync::Arc};
use crate::objects::stringobject::StringObject;

use super::{Object, ObjectTrait, get_type, add_type, MethodValue, ObjectInternals};

#[derive(Clone)]
pub struct NoneType {
    tp: Object,
}

impl ObjectTrait for NoneType {
    fn get_name(self: Arc<Self>) -> String {
        String::from("NoneType")
    }
    fn get_type(self: Arc<Self>) -> Object {
        self.tp.clone()
    }
    fn get_bases(self: Arc<Self>) -> Vec<Object> {
        vec![get_type("type")]
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        MethodValue::Some(StringObject::from("<class 'NoneType'>".to_string()))
    }
}

impl NoneType {
    pub fn init(){
        let tp = Arc::new(NoneType{tp: get_type("type")});
        add_type("NoneType", tp);
    }
}


#[derive(Clone)]
pub struct NoneObject {
    tp: Object,
}

impl NoneObject {
    pub fn from() -> Object {
        Arc::new(NoneObject { tp: get_type("NoneType") })
    }
}

impl ObjectTrait for NoneObject {
    fn get_name(self: Arc<Self>) -> String {
        self.tp.clone().get_name()
    }
    fn get_raw(self: Arc<Self>) -> ObjectInternals {
        ObjectInternals::None
    }
    fn get_type(self: Arc<Self>) -> Object {
        self.tp.clone()
    }
    fn get_bases(self: Arc<Self>) -> Vec<Object> {
        self.tp.clone().get_bases()
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        return MethodValue::Some(StringObject::from("None".to_string()));
    }
}