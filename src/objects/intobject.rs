use std::{sync::Arc};
use crate::objects::stringobject::StringObject;
use crate::objects::listobject::ListObject;

use super::{Object, ObjectTrait, get_type, add_type, MethodValue};

#[derive(Clone)]
pub struct IntType {
    tp: Object,
}

impl ObjectTrait for IntType {
    fn get_name(self: Arc<Self>) -> String {
        return String::from("int");
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
        return MethodValue::Some(StringObject::from("<class 'int'>".to_string()));
    }
    fn eq(self: Arc<Self>, _other: Object) -> MethodValue<Object, Object> {
        return MethodValue::NotImplemented;
    }
}

impl IntType {
    pub fn init(){
        let tp = Arc::new(IntType{tp: get_type("type")});
        add_type("int", tp);
    }
}


#[derive(Clone)]
pub struct IntObject {
    tp: Object,
    value: i128,
}

impl IntObject {
    pub fn from(value: i128) -> Object {
        return Arc::new(IntObject { tp: get_type("int"), value});
    }
}

impl ObjectTrait for IntObject {
    fn get_name(self: Arc<Self>) -> String {
        let strong = self.tp.clone();
        return strong.get_name();
    }
    fn get_basic_repr(self: Arc<Self>) -> MethodValue<String, Object> {
        return MethodValue::NotImplemented;
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
        return MethodValue::Some(StringObject::from(self.value.to_string()));
    }
    fn eq(self: Arc<Self>, _other: Object) -> MethodValue<Object, Object> {
        return MethodValue::NotImplemented;
    }
}