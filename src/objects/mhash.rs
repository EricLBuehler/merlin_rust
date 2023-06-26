use crate::{
    is_type_exact,
    objects::{
        exceptionobject::{methodnotdefinedexc_from_str, typemismatchexc_from_str},
        MethodValue,
    },
    parser::Position,
    unwrap_fast,
};

use super::{exceptionobject::keynotfoundexc_from_str, MethodType, Object, RawObject};

#[derive(Clone, PartialEq, Eq)]
pub struct HashMap<'a> {
    values: hashbrown::HashMap<isize, (Object<'a>, Object<'a>)>,
}

impl<'a> Default for HashMap<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> HashMap<'a> {
    pub fn new() -> Self {
        HashMap {
            values: hashbrown::HashMap::new(),
        }
    }

    #[allow(unused_unsafe)]
    #[inline]
    fn hash(key: Object<'a>) -> MethodValue<isize, Object<'a>> {
        if key.tp.hash_fn.is_none() {
            let exc = methodnotdefinedexc_from_str(
                key.vm.clone(),
                &format!(
                    "Method 'hash' is not defined for '{}' type",
                    key.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            unwrap_fast!(key.vm.interpreters.last()).raise_exc(exc);
        }
        let res = (key.tp.hash_fn.expect("Hash function not found"))(key.clone());
        if res.is_error() {
            return MethodValue::Error(res.unwrap_err());
        }

        if !is_type_exact!(
            &unwrap_fast!(res),
            unwrap_fast!(key.vm.types.inttp.as_ref()).clone()
        ) {
            let exc = typemismatchexc_from_str(
                key.vm.clone(),
                "Method 'hash' did not return 'int'",
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        MethodValue::Some(unsafe { unwrap_fast!(res).internals.int })
    }

    #[inline]
    pub fn insert(&mut self, key: Object<'a>, value: Object<'a>) -> MethodValue<(), Object<'a>> {
        let keyv = Self::hash(key.clone());
        if keyv.is_error() {
            return MethodValue::Error(keyv.unwrap_err());
        }
        self.values.insert(unwrap_fast!(keyv), (key, value));
        MethodValue::Some(())
    }

    pub fn get(&self, key: Object<'a>) -> MethodType<'a> {
        let keyv = Self::hash(key.clone());
        if keyv.is_error() {
            return MethodValue::Error(keyv.unwrap_err());
        }
        let res = self.values.get(&unwrap_fast!(keyv));
        if res.is_none() {
            let str = RawObject::object_str_safe(key.clone());
            if str.is_error() {
                return MethodValue::Error(str.unwrap_err());
            }
            let exc = keynotfoundexc_from_str(
                key.vm.clone(),
                &format!("Key '{}' not found", unwrap_fast!(str)),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }
        MethodValue::Some(unwrap_fast!(res).1.clone())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

pub struct HMapIter<'a> {
    keys: Vec<isize>,
    values: hashbrown::HashMap<isize, (Object<'a>, Object<'a>)>,
    i: usize,
}

impl<'a> Iterator for HMapIter<'a> {
    type Item = (Object<'a>, Object<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys.get(self.i)?;
        let get = unwrap_fast!(self.values.get(key));
        self.i += 1;
        Some((get.0.clone(), get.1.clone()))
    }
}

impl<'a> IntoIterator for &HashMap<'a> {
    type Item = (Object<'a>, Object<'a>);
    type IntoIter = HMapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        return HMapIter {
            keys: self.values.keys().copied().collect(),
            values: self.values.clone(),
            i: 0,
        };
    }
}
