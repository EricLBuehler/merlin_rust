use super::{Object, MethodValue};

pub fn object_repr(object: &Object) -> String {
    return object.clone().repr().unwrap().get_raw().unwrap().get_str().unwrap().to_owned();
}

pub fn object_repr_safe(object: &Object) -> MethodValue<String, String> {
    let repr = object.clone().repr();
    if repr.is_not_implemented() {
        return MethodValue::Error(String::from("__repr__ is not implemented."));
    }
    
    let reprv = repr.unwrap().get_raw();
    
    if reprv.is_not_implemented() {
        return MethodValue::Error(String::from("__repr__ returned non-string."));
    }    
    if !reprv.unwrap().is_str() {
        return MethodValue::Error(String::from("__repr__ returned non-string."));
    }


    return MethodValue::Some(reprv.unwrap().get_str().unwrap().to_owned());
}