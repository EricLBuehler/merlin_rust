use std::{sync::Arc};
use crate::objects::stringobject::StringObject;

use super::{Object, ObjectTrait, get_type, add_type, MethodValue, utils, ObjectInternals};

#[derive(Clone)]
pub struct ListType {
    tp: Object,
}

impl ObjectTrait for ListType {
    fn get_name(self: Arc<Self>) -> String {
        String::from("list")
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
    pub fn from(value: Vec<Object>) -> Object {
        Arc::new(ListObject { tp: get_type("str"), value })
    }
}

impl ObjectTrait for ListObject {
    fn get_name(self: Arc<Self>) -> String {
        self.tp.clone().get_name()
    }
    fn get_raw(self: Arc<Self>) -> ObjectInternals {
        ObjectInternals::Arr(self.value.clone())
    }
    fn get_type(self: Arc<Self>) -> Object {
        self.tp.clone()
    }
    fn get_bases(self: Arc<Self>) -> Vec<Object> {
        self.tp.clone().get_bases()
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        let mut res = String::from("[");
        for item in &self.value {
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
        MethodValue::Some(StringObject::from(res))
    }
}