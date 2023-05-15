use std::{sync::Arc};
use crate::objects::stringobject::StringObject;
use crate::objects::listobject::ListObject;

use super::{Object, ObjectTrait, get_type, add_type, MethodValue, ObjectInternals};

#[derive(Clone)]
pub struct IntType {
    tp: Object,
}

impl ObjectTrait for IntType {
    fn get_name(self: Arc<Self>) -> String {
        return String::from("int");
    }
    fn get_type(self: Arc<Self>) -> Object {
        return self.tp.clone();
    }
    fn get_bases(self: Arc<Self>) -> Object {
        return ListObject::from(vec![get_type("type")]);
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> {
        return MethodValue::Some(StringObject::from("<class 'int'>".to_string()));
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
    pub fn from_str(value: String) -> MethodValue<Object, Object> {
        let convert = value.parse::<i128>();
        debug_assert!(!convert.is_err());
        return MethodValue::Some(Arc::new(IntObject { tp: get_type("int"), value: convert.unwrap()}));
    }
}

impl ObjectTrait for IntObject {
    fn get_name(self: Arc<Self>) -> String {
        let strong = self.tp.clone();
        return strong.get_name();
    }
    fn get_raw(self: Arc<Self>) -> ObjectInternals {
        return ObjectInternals::Int(self.value);
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
    fn abs(self: Arc<Self>) -> MethodValue<Object, Object> {
        let res = self.value.checked_abs();
        debug_assert!(res.is_some());

        return MethodValue::Some(Self::from(res.unwrap()))
    }
    fn neg(self: Arc<Self>) -> MethodValue<Object, Object> {
        let res = self.value.checked_neg();
        debug_assert!(res.is_some());

        return MethodValue::Some(Self::from(res.unwrap()))
    }
    fn add(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> {
        debug_assert!(self.clone().get_typeid() == other.clone().get_typeid());

        let otherv = other.get_raw().get_int().unwrap().clone();

        let res = self.value.checked_add(otherv);
        debug_assert!(res.is_some());

        return MethodValue::Some(Self::from(res.unwrap()))
    }
    fn sub(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> {
        debug_assert!(self.clone().get_typeid() == other.clone().get_typeid());

        let otherv = other.get_raw().get_int().unwrap().clone();

        let res = self.value.checked_sub(otherv);
        debug_assert!(res.is_some());

        return MethodValue::Some(Self::from(res.unwrap()))
    }
    fn mul(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> {
        debug_assert!(self.clone().get_typeid() == other.clone().get_typeid());

        let otherv = other.get_raw().get_int().unwrap().clone();

        let res = self.value.checked_mul(otherv);
        debug_assert!(res.is_some());

        return MethodValue::Some(Self::from(res.unwrap()))
    }
    fn div(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> {
        debug_assert!(self.clone().get_typeid() == other.clone().get_typeid());

        let otherv = other.get_raw().get_int().unwrap().clone();
        debug_assert!(otherv != 0);

        let res = self.value.checked_div(otherv);
        debug_assert!(res.is_some());

        return MethodValue::Some(Self::from(res.unwrap()))
    }
    fn pow(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> {
        debug_assert!(self.clone().get_typeid() == other.clone().get_typeid());

        let otherv = other.get_raw().get_int().unwrap().clone();

        if otherv >=0 {
            debug_assert!(otherv < std::u32::MAX as i128);

            let res = self.value.checked_pow(otherv as u32);
            debug_assert!(res.is_some());
    
            return MethodValue::Some(Self::from(res.unwrap()))
        }

        debug_assert!(otherv.abs() < std::u32::MAX as i128);

        let res = self.value.checked_pow(otherv.abs() as u32);
        debug_assert!(res.is_some());

        return MethodValue::Some(Self::from(res.unwrap()))
    }
}