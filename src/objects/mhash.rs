use crate::{
    objects::{
        exceptionobject::{methodnotdefinedexc_from_str, typemismatchexc_from_str}, MethodValue,
    },
    parser::Position, is_type_exact,
};

use super::{exceptionobject::keynotfoundexc_from_str, utils::object_str_safe, MethodType, Object};

#[derive(Clone, PartialEq, Eq)]
pub struct HashMap<'a> {
    values: hashbrown::HashMap<i128, Object<'a>>,
    keymap: hashbrown::HashMap<i128, Object<'a>>,
}

impl<'a> HashMap<'a> {
    pub fn new() -> Self {
        return HashMap {
            values: hashbrown::HashMap::new(),
            keymap: hashbrown::HashMap::new(),
        };
    }

    fn hash(key: Object<'a>) -> MethodValue<i128, Object<'a>> {
        if key.hash_fn.is_none() {
            let exc = methodnotdefinedexc_from_str(
                key.vm.clone(),
                "Method 'hash' is not defined",
                Position::default(),
                Position::default(),
            );
            key.vm.interpreters.last().unwrap().raise_exc(exc);
        }
        let res = (key.hash_fn.expect("Hash function not found"))(key.clone());
        if res.is_error() {
            return MethodValue::Error(res.unwrap_err());
        }
        debug_assert!(is_type_exact!(&res.unwrap(), &key.vm.get_type("int")));
        if !is_type_exact!(&res.unwrap(), &key.vm.get_type("int")) {
            let exc = typemismatchexc_from_str(
                key.vm.clone(),
                "Method 'hash' did not return 'int'",
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        MethodValue::Some(
            *res.unwrap()
                .internals
                .get_int()
                .expect("Expected int internal value"),
        )
    }

    pub fn insert(&mut self, key: Object<'a>, value: Object<'a>) -> MethodValue<(), Object<'a>> {
        let keyv = Self::hash(key.clone());
        if keyv.is_error() {
            return MethodValue::Error(keyv.unwrap_err());
        }
        self.values.insert(keyv.unwrap(), value);
        self.keymap.insert(keyv.unwrap(), key);
        MethodValue::Some(())
    }

    pub fn get(&self, key: Object<'a>) -> MethodType<'a> {
        let keyv = Self::hash(key.clone());
        if keyv.is_error() {
            return MethodValue::Error(keyv.unwrap_err());
        }
        let res = self.values.get(&keyv.unwrap());
        if res.is_none() {
            let str = object_str_safe(key.clone());
            if str.is_error() {
                return MethodValue::Error(str.unwrap_err());
            }
            let exc = keynotfoundexc_from_str(
                key.vm.clone(),
                &format!("Key '{}' not found", str.unwrap()),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }
        return MethodValue::Some(res.unwrap().clone());
    }

    pub fn len(&self) -> usize {
        return self.values.len();
    }
}

pub struct HMapIter<'a> {
    keys: Vec<i128>,
    values: hashbrown::HashMap<i128, Object<'a>>,
    keymap: hashbrown::HashMap<i128, Object<'a>>,
    i: usize,
}

impl<'a> Iterator for HMapIter<'a> {
    type Item = (Object<'a>, Object<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys.get(self.i);
        if key.is_none() {
            return None;
        }
        return Some((
            self.keymap.get(key.unwrap()).unwrap().clone(),
            self.values.get(key.unwrap()).unwrap().clone(),
        ));
    }
}

impl<'a> IntoIterator for &HashMap<'a> {
    type Item = (Object<'a>, Object<'a>);
    type IntoIter = HMapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        return HMapIter {
            keys: self.values.keys().map(|x| x.clone()).collect(),
            values: self.values.clone(),
            keymap: self.keymap.clone(),
            i: 0,
        };
    }
}
