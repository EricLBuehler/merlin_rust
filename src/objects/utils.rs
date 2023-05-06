use super::Object;

pub fn object_repr(object: &Object) -> String {
    return object.clone().repr().unwrap().get_basic_repr().unwrap();
}