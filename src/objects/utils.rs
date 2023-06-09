use super::{stringobject, MethodValue, Object};

pub fn object_repr(object: &Object<'_>) -> String {
    return (object.clone().tp.repr.expect("Method is not defined"))(object.clone())
        .unwrap()
        .internals
        .get_str()
        .expect("Expected str internal value")
        .clone();
}

pub fn object_repr_safe(object: Object<'_>) -> MethodValue<String, Object<'_>> {
    let repr = object.clone().tp.repr;
    if repr.is_none() {
        return MethodValue::Error(stringobject::string_from(
            object.vm.clone(),
            String::from("__repr__ is not implemented."),
        ));
    }

    let reprv = (repr.unwrap())(object.clone());

    if reprv.is_error() {
        return MethodValue::Error(reprv.unwrap_err());
    }

    if reprv.is_not_implemented() {
        return MethodValue::Error(stringobject::string_from(
            object.vm.clone(),
            String::from("__repr__ is not implemented."),
        ));
    }

    if !reprv.unwrap().internals.is_str() {
        return MethodValue::Error(stringobject::string_from(
            object.vm.clone(),
            String::from("__repr__ returned non-string."),
        ));
    }

    return MethodValue::Some(
        reprv
            .unwrap()
            .internals
            .get_str()
            .expect("Expected str internal value")
            .to_owned(),
    );
}

#[allow(dead_code)]
pub fn object_str(object: &Object<'_>) -> String {
    return (object.clone().tp.str.expect("Method is not defined"))(object.clone())
        .unwrap()
        .internals
        .get_str()
        .expect("Expected str internal value")
        .clone();
}

pub fn object_str_safe(object: Object<'_>) -> MethodValue<String, Object<'_>> {
    let str = object.clone().tp.str;
    if str.is_none() {
        return MethodValue::Error(stringobject::string_from(
            object.vm.clone(),
            String::from("__repr__ is not implemented."),
        ));
    }

    let strv = (str.unwrap())(object.clone());

    if strv.is_error() {
        return MethodValue::Error(strv.unwrap_err());
    }

    if strv.is_not_implemented() {
        return MethodValue::Error(stringobject::string_from(
            object.vm.clone(),
            String::from("__repr__ is not implemented."),
        ));
    }

    if !strv.unwrap().internals.is_str() {
        return MethodValue::Error(stringobject::string_from(
            object.vm.clone(),
            String::from("__repr__ returned non-string."),
        ));
    }

    return MethodValue::Some(
        strv.unwrap()
            .internals
            .get_str()
            .expect("Expected str internal value")
            .to_owned(),
    );
}
