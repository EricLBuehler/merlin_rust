use std::{sync::Arc};
use crate::objects::listobject::ListObject;

use super::{Object, ObjectTrait, get_type, add_type, MethodValue};

#[derive(Clone)]
pub struct StringType {
    tp: Object,
}

impl ObjectTrait for StringType {
    fn get_name(self: Arc<Self>) -> String {
        return String::from("str");
    }
    fn get_basic_repr(self: Arc<Self>) -> MethodValue<String, Object> {
        return MethodValue::NotImplemented;
    }
    fn get_type(self: Arc<Self>) -> Object {
        return self.tp.clone();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        return ListObject::from(vec![get_type("type")]);
    }
    fn new(self: Arc<Self>, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
        return MethodValue::NotImplemented;
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        return MethodValue::Some(StringObject::from("<class 'str'>".to_string()));
    }
    fn eq(self: Arc<Self>, _other: Object) -> MethodValue<Object, Object> {
        return MethodValue::NotImplemented;
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
        let strong = self.tp.clone();
        return strong.get_name();
    }
    fn get_basic_repr(self: Arc<Self>) -> MethodValue<String, Object> {
        return MethodValue::Some(self.value.clone());
    }
    fn get_type(self: Arc<Self>) -> Object {
        return self.tp.clone();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        let strong = self.tp.clone();
        return strong.get_bases();
    }
    fn new(self: Arc<Self>, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> {
        return MethodValue::NotImplemented;
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        return MethodValue::Some(self.clone())
    }
    fn eq(self: Arc<Self>, _other: Object) -> MethodValue<Object, Object> {
        return MethodValue::NotImplemented;
    }
}