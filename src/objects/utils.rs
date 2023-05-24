use super::{Object, MethodValue};

pub fn object_repr(object: &Object) -> String {
    return (object.clone().repr.unwrap())(object.clone()).unwrap().internals.get_str().unwrap().clone();
}

pub fn object_repr_safe(object: &Object) -> MethodValue<String, String> {
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


    return MethodValue::Some(reprv.unwrap().internals.get_str().unwrap().to_owned());
}