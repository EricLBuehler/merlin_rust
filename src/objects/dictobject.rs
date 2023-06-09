use super::mhash::HashMap;
use super::{
    create_object_from_type, finalize_type, intobject, utils, MethodType, MethodValue, Object,
    TypeObject,
};

use crate::is_type_exact;
use crate::objects::exceptionobject::{methodnotdefinedexc_from_str, typemismatchexc_from_str};
use crate::parser::Position;
use crate::trc::Trc;
use crate::{
    interpreter::VM,
    objects::{boolobject, stringobject, ObjectInternals},
};

#[allow(dead_code)]
pub fn dict_from<'a>(vm: Trc<VM<'a>>, raw: HashMap<'a>) -> Object<'a> {
    let mut tp = create_object_from_type(vm.types.dicttp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Map(raw);
    tp
}

fn dict_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn dict_repr(selfv: Object<'_>) -> MethodType<'_> {
    let mut res = String::from("{");
    let sf = selfv.clone();
    let map = sf
        .internals
        .get_map()
        .expect("Expected map internal value")
        .clone();
    for (key, value) in map.into_iter() {
        let repr = utils::object_repr_safe(key);
        if !repr.is_some() {
            return MethodValue::NotImplemented;
        }
        res += &repr.unwrap();
        res += ": ";
        let repr = utils::object_repr_safe(value);
        if !repr.is_some() {
            return MethodValue::NotImplemented;
        }
        res += &repr.unwrap();
        res += ", ";
    }
    if res.len() > 1 {
        res.pop();
        res.pop();
    }
    res += "}";
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), res))
}

fn dict_get<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    //NEGATIVE INDEX IS CONVERTED TO +
    let out = selfv
        .internals
        .get_map()
        .expect("Expected map internal value")
        .get(other);

    if out.is_error() {
        return MethodValue::Error(out.unwrap_err());
    }
    MethodValue::Some(out.unwrap().clone())
}

#[inline]
fn dict_set<'a>(mut selfv: Object<'a>, other: Object<'a>, value: Object<'a>) -> MethodType<'a> {
    //TODO check for hash here!
    let mut map = selfv
        .internals
        .get_map()
        .expect("Expected map internal value")
        .clone();
    let res = map.insert(other, value);
    if res.is_error() {
        return MethodValue::Error(res.unwrap_err());
    }

    selfv.internals = ObjectInternals::Map(map);

    MethodValue::Some(none_from!(selfv.vm))
}
fn dict_len(selfv: Object<'_>) -> MethodType<'_> {
    let convert: Result<i128, _> = selfv
        .internals
        .get_map()
        .expect("Expected map internal value")
        .len()
        .try_into();

    MethodValue::Some(intobject::int_from(selfv.vm.clone(), convert.unwrap()))
}

fn dict_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    if selfv
        .internals
        .get_map()
        .expect("Expected map internal value")
        .len()
        != other
            .internals
            .get_map()
            .expect("Expected map internal value")
            .len()
    {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }
    for ((key1, value1), (key2, value2)) in std::iter::zip(
        selfv
            .internals
            .get_map()
            .expect("Expected map internal value")
            .clone()
            .into_iter(),
        other
            .internals
            .get_map()
            .expect("Expected map internal value")
            .clone()
            .into_iter(),
    ) {
        if key1.tp.eq.is_none() {
            let exc = methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'eq' is not defined for key 1 type '{}'",
                    key1.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }
        if value1.tp.eq.is_none() {
            let exc = methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'eq' is not defined for value 1 type '{}'",
                    value1.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        let res = (key1.tp.eq.expect("Method is not defined"))(key1.clone(), key2.clone());
        if res.is_error() {
            return res;
        }
        if !is_type_exact!(
            &res.unwrap(),
            selfv.vm.types.booltp.as_ref().unwrap().clone()
        ) {
            let exc = typemismatchexc_from_str(
                selfv.vm.clone(),
                "Method 'eq' did not return 'bool'",
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        if *res
            .unwrap()
            .internals
            .get_bool()
            .expect("Expected bool internal value")
        {
            return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
        }

        let res = (value1.tp.eq.expect("Method is not defined"))(value1.clone(), value2.clone());
        if res.is_error() {
            return res;
        }
        if !is_type_exact!(
            &res.unwrap(),
            selfv.vm.types.booltp.as_ref().unwrap().clone()
        ) {
            let exc = typemismatchexc_from_str(
                selfv.vm.clone(),
                "Method 'eq' did not return 'bool'",
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        if *res
            .unwrap()
            .internals
            .get_bool()
            .expect("Expected bool internal value")
        {
            return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
        }
    }
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), true))
}

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("dict"),
        bases: vec![super::ObjectBase::Other(
            vm.types.objecttp.as_ref().unwrap().clone(),
        )],
        vm: vm.clone(),

        new: Some(dict_new),

        repr: Some(dict_repr),
        str: Some(dict_repr),
        abs: None,
        neg: None,
        hash_fn: None,

        eq: Some(dict_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: Some(dict_get),
        set: Some(dict_set),
        len: Some(dict_len),

        call: None,
    });

    vm.types.dicttp = Some(tp.clone());

    finalize_type(tp);
}
