use std::{sync::Arc};
use crate::objects::stringobject::StringObject;

use super::{Object, ObjectTrait, get_type, add_type};

#[derive(Clone)]
pub struct ListType {
    tp: Object,
}

impl ObjectTrait for ListType {
    fn get_name(self: Arc<Self>) -> String {
        return String::from("list");
    }
    fn get_basic_repr(self: Arc<Self>) -> Option<String> {
        return None;
    }
    fn get_type(self: Arc<Self>) -> Object {
        return self.tp.clone();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        return ListObject::from(vec![get_type("type"), get_type("object")]);
    }
    fn new(self: Arc<Self>, _args: Object, _kwargs: Object) -> Option<Object> {
        return None;
    }
    fn repr(self: Arc<Self>) -> Option<Object> {
        return Some(StringObject::from("<class 'str'>".to_string()));
    }
    fn eq(self: Arc<Self>, _other: Object) -> Option<Object> {
        return None;
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

impl ObjectTrait for Arc<ListObject> {
    fn get_name(self: Arc<Self>) -> String {
        let strong = self.tp.clone();
        return strong.get_name();
    }
    fn get_basic_repr(self: Arc<Self>) -> Option<String> {
        return None;
    }
    fn get_type(self: Arc<Self>) -> Object {
        return self.tp.clone();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        let strong = self.tp.clone();
        return strong.get_bases();
    }
    fn new(self: Arc<Self>, _args: Object, _kwargs: Object) -> Option<Object> {
        unimplemented!();
    }
    fn repr(self: Arc<Self>) -> Option<Object> {
        return Some(self.clone())
    }
    fn eq(self: Arc<Self>, _other: Object) -> Option<Object> {
        return None;
    }
}

impl ObjectTrait for ListObject {
    fn get_name(self: Arc<Self>) -> String {
        unimplemented!();
    }
    fn get_basic_repr(self: Arc<Self>) -> Option<String> {
        unimplemented!();
    }
    fn get_type(self: Arc<Self>) -> Object {
        unimplemented!();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        unimplemented!();
    }
    fn new(self: Arc<Self>, _args: Object, _kwargs: Object) -> Option<Object> {
        unimplemented!();
    }
    fn repr(self: Arc<Self>) -> Option<Object> {
        unimplemented!();
    }
    fn eq(self: Arc<Self>, _other: Object) -> Option<Object> {
        unimplemented!();
    }
}