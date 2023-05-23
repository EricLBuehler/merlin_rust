use std::{sync::Arc};

use super::{Object, ObjectTrait, get_type, add_type, MethodValue, ObjectInternals};

#[derive(Clone)]
pub struct StringType {
    tp: Object,
}

impl ObjectTrait for StringType {
    fn get_name(self: Arc<Self>) -> String {
        String::from("str")
    }
    fn get_type(self: Arc<Self>) -> Object {
        self.tp.clone()
    }
    fn get_bases(self: Arc<Self>) -> Vec<Object> {
        vec![get_type("type")]
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        MethodValue::Some(StringObject::from("<class 'str'>".to_string()))
    }
}

impl StringType {
    pub fn init(){
        let tp = Arc::new(StringType{tp: get_type("type")});
        add_type("str", tp);
    }
}


#[derive(Clone)]
pub struct StringObject {
    tp: Object,
    value: String,
}

impl StringObject {
    pub fn from(value: String) -> Object {
        return Arc::new(StringObject { tp: get_type("str"), value});
    }
}

impl ObjectTrait for StringObject {
    fn get_name(self: Arc<Self>) -> String {
        self.tp.clone().get_name()
    }
    fn get_raw(self: Arc<Self>) -> ObjectInternals {
        ObjectInternals::Str(self.value.clone())
    }
    fn get_type(self: Arc<Self>) -> Object {
        self.tp.clone()
    }
    fn get_bases(self: Arc<Self>) -> Vec<Object> {
        self.tp.clone().get_bases()
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        MethodValue::Some(self)
    }
}