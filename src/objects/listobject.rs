use crate::{objects::{stringobject, ObjectInternals, boolobject}, interpreter::VM};
use super::{RawObject, Object,MethodType, MethodValue, utils, finalize_type, is_instance, intobject, create_object_from_type};
use crate::Arc;

pub fn list_from<'a>(vm: Arc<VM<'a>>, raw: Vec<Object<'a>>) -> Object<'a> {
    let mut tp = create_object_from_type(vm.get_type("list"));
    let refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Arr(raw);
    tp
}

fn list_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn list_repr(selfv: Object<'_>) -> MethodType<'_> {
    let mut res = String::from("[");
    for item in selfv.internals.get_arr().expect("Expected arr internal value") {
        let repr = utils::object_repr_safe(item);
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
    res += "]";
    MethodValue::Some(stringobject::string_from(selfv.vm.clone(), res))
}

fn list_get<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&other, &selfv.vm.get_type("int")));
    //NEGATIVE INDEX IS CONVERTED TO +
    let out = selfv.internals.get_arr().expect("Expected arr internal value").get((*other.internals.get_int().expect("Expected int internal value")).unsigned_abs() as usize);
    debug_assert!(out.is_some());
    MethodValue::Some(out.unwrap().clone())
}
fn list_set<'a>(selfv: Object<'a>, other: Object<'a>, value: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&other, &selfv.vm.get_type("int")));
    //NEGATIVE INDEX IS CONVERTED TO +
    debug_assert!(((*other.internals.get_int().expect("Expected int internal value")).unsigned_abs() as usize) < selfv.internals.get_arr().expect("Expected arr internal value").len());
    let mut arr = selfv.internals.get_arr().expect("Expected arr internal value").clone();
    arr[(*other.internals.get_int().expect("Expected int internal value")).unsigned_abs() as usize] = value;
    
    unsafe {
        let refr = Arc::into_raw(selfv.clone()) as *mut RawObject<'a>;
        (*refr).internals = ObjectInternals::Arr(arr.to_vec());
        Arc::from_raw(refr);
    }

    MethodValue::Some(none_from!(selfv.vm.clone()))
}
fn list_len(selfv: Object<'_>) -> MethodType<'_> {
    let convert: Result<i128, _> = selfv.internals.get_arr().expect("Expected arr internal value").len().try_into();
    debug_assert!(convert.is_ok());
    MethodValue::Some(intobject::int_from(selfv.vm.clone(), convert.unwrap()))
}

fn list_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    debug_assert!(is_instance(&selfv, &other));
    debug_assert!(selfv.internals.get_arr().expect("Expected arr internal value").len() == other.internals.get_arr().expect("Expected arr internal value").len());
    for idx in 0..selfv.internals.get_arr().expect("Expected arr internal value").len() {
        debug_assert!(selfv.internals.get_arr().expect("Expected arr internal value").get(idx).unwrap().eq.is_some());
        let v = selfv.internals.get_arr().expect("Expected arr internal value").get(idx).unwrap();
        let res = (v.eq.expect("Method is not defined"))(v.clone(), other.internals.get_arr().expect("Expected arr internal value").get(idx).unwrap().clone());
        debug_assert!(res.is_some());
        debug_assert!(is_instance(&res.unwrap(), &selfv.vm.get_type("bool")));
        if *res.unwrap().internals.get_bool().expect("Expected bool internal value") {
            return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
        }
    }
    MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), true))
}

pub fn init<'a>(vm: Arc<VM<'a>>){
    let tp: Arc<RawObject<'a>> = Arc::new( RawObject{
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("list"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(list_new),

        repr: Some(list_repr),
        str: Some(list_repr),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(list_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,
        
        get: Some(list_get),
        set: Some(list_set),
        len: Some(list_len),
        
        call: None,
    });

    VM::add_type(vm.clone(), &tp.clone().typename, tp.clone());

    finalize_type(tp);
}