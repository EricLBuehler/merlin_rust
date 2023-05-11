use super::{Object, MethodValue};

pub fn object_repr(object: &Object) -> String {
    return object.clone().repr().unwrap().get_basic_repr().unwrap();
}

pub fn object_repr_safe(object: &Object) -> MethodValue<String, String> {
    let repr = object.clone().repr();
    if repr.is_not_implemented() {
        return MethodValue::Error(String::from("__repr__ is not implemented."));
    }
    
    let reprv = repr.unwrap().get_basic_repr();
    
    if reprv.is_not_implemented() {
        return MethodValue::Error(String::from("__repr__ returned non-string."));
    }

    return MethodValue::Some(reprv.unwrap());
}