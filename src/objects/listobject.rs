use std::{sync::Arc};
use crate::objects::stringobject::StringObject;

use super::{Object, ObjectTrait, get_type, add_type, MethodValue, utils};

#[derive(Clone)]
pub struct ListType {
    tp: Object,
}

impl ObjectTrait for ListType {
    fn get_name(self: Arc<Self>) -> String {
        return String::from("list");
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

impl ListType {
    pub fn init(){
        let tp = Arc::new(ListType{tp: get_type("type")});
        add_type("list", tp);
    }
}


#[derive(Clone)]
pub struct ListObject {
    tp: Object,
    value: Vec<Object>,
}

impl ListObject {
    pub fn from(value: Vec<Object>) -> Arc<Self> {
        return Arc::new(ListObject { tp: get_type("str"), value });
    }
}

impl ObjectTrait for ListObject {
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
        let mut res = String::from("[");
        for item in &self.value {
            let repr = utils::object_repr_safe(item);
            if repr.is_error() {
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
        return MethodValue::Some(StringObject::from(res));
    }
    fn eq(self: Arc<Self>, _other: Object) -> MethodValue<Object, Object> {
        return MethodValue::NotImplemented;
    }
}