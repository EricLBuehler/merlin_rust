use super::{Object, MethodValue};

pub fn object_repr(object: &Object<'_>) -> String {
    return (object.clone().repr.expect("Method is not defined"))(object.clone()).unwrap().internals.get_str().expect("Expected str internal value").clone();
}

pub fn object_repr_safe(object: &Object<'_>) -> MethodValue<String, String> {
    let repr = object.clone().repr;
    if repr.is_none() {
        return MethodValue::Error(String::from("__repr__ is not implemented."));
    }
    
    let reprv = (repr.unwrap())(object.clone());

    debug_assert!(!reprv.is_error());

    if reprv.is_not_implemented() {
        return MethodValue::Error(String::from("__repr__ is not implemented."));
    }

    if !reprv.unwrap().internals.is_str() {
        return MethodValue::Error(String::from("__repr__ returned non-string."));
    }


    return MethodValue::Some(reprv.unwrap().internals.get_str().expect("Expected str internal value").to_owned());
}

pub fn object_str(object: &Object<'_>) -> String {
    return (object.clone().str.expect("Method is not defined"))(object.clone()).unwrap().internals.get_str().expect("Expected str internal value").clone();
}

pub fn object_str_safe(object: &Object<'_>) -> MethodValue<String, String> {
    let str = object.clone().str;
    if str.is_none() {
        return MethodValue::Error(String::from("__repr__ is not implemented."));
    }
    
    let strv = (str.unwrap())(object.clone());

    debug_assert!(!strv.is_error());

    if strv.is_not_implemented() {
        return MethodValue::Error(String::from("__repr__ is not implemented."));
    }

    if !strv.unwrap().internals.is_str() {
        return MethodValue::Error(String::from("__repr__ returned non-string."));
    }


    return MethodValue::Some(strv.unwrap().internals.get_str().expect("Expected str internal value").to_owned());
}