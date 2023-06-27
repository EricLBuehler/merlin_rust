#![allow(unused_unsafe)]
use std::mem::ManuallyDrop;

use trc::Trc;

use crate::{interpreter::VM, parser::Position, unwrap_fast};

use super::{
    create_object_from_typeobject, exceptionobject::methodnotdefinedexc_from_str, finalize_type,
    listobject, stringobject, MethodType, MethodValue, Object, ObjectInternals, TypeObject,
};

//unary
fn class_repr(selfv: Object<'_>) -> MethodType<'_> {
    let repr = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "repr".to_string()),
    );
    if repr.is_some() {
        let call_fn = unwrap_fast!(repr).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(repr).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(repr), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'repr' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_str(selfv: Object<'_>) -> MethodType<'_> {
    let str = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "str".to_string()),
    );
    if str.is_some() {
        let call_fn = unwrap_fast!(str).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(str).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(str), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'str' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_abs(selfv: Object<'_>) -> MethodType<'_> {
    let abs = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "abs".to_string()),
    );
    if abs.is_some() {
        let call_fn = unwrap_fast!(abs).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(abs).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(abs), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'abs' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_neg(selfv: Object<'_>) -> MethodType<'_> {
    let neg = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "neg".to_string()),
    );
    if neg.is_some() {
        let call_fn = unwrap_fast!(neg).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(neg).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(neg), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'neg' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_hash(selfv: Object<'_>) -> MethodType<'_> {
    let hash = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "hash".to_string()),
    );
    if hash.is_some() {
        let call_fn = unwrap_fast!(hash).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(hash).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(hash), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'hash' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

//binary
fn class_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    let eq = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "eq".to_string()),
    );
    if eq.is_some() {
        let call_fn = unwrap_fast!(eq).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(eq).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv, other]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(eq), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'hash' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_add<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    let add = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "add".to_string()),
    );
    if add.is_some() {
        let call_fn = unwrap_fast!(add).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(add).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv, other]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(add), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'add' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_sub<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    let sub = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "sub".to_string()),
    );
    if sub.is_some() {
        let call_fn = unwrap_fast!(sub).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(sub).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv, other]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(sub), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'sub' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_mul<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    let mul = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "mul".to_string()),
    );
    if mul.is_some() {
        let call_fn = unwrap_fast!(mul).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(mul).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv, other]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(mul), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'mul' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_div<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    let div = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "div".to_string()),
    );
    if div.is_some() {
        let call_fn = unwrap_fast!(div).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(div).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv, other]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(div), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'div' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_pow<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    let pow = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "pow".to_string()),
    );
    if pow.is_some() {
        let call_fn = unwrap_fast!(pow).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(pow).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv, other]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(pow), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'pow' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

//sequences
fn class_get<'a>(selfv: Object<'a>, key: Object<'a>) -> MethodType<'a> {
    let get = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "get".to_string()),
    );
    if get.is_some() {
        let call_fn = unwrap_fast!(get).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(get).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv, key]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(get), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'get' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_set<'a>(selfv: Object<'a>, key: Object<'a>, value: Object<'a>) -> MethodType<'a> {
    let set = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "set".to_string()),
    );
    if set.is_some() {
        let call_fn = unwrap_fast!(set).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(set).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv, key, value]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(set), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'set' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

fn class_len(selfv: Object<'_>) -> MethodType<'_> {
    let len = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "len".to_string()),
    );
    if len.is_some() {
        let call_fn = unwrap_fast!(len).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(len).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let args = listobject::list_from(selfv.vm.clone(), vec![selfv]);
        return (unwrap_fast!(call_fn))(unwrap_fast!(len), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'len' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

//interaction
fn class_call<'a>(selfv: Object<'a>, args: Object<'a>) -> MethodType<'a> {
    let call = unsafe { &unwrap_fast!(selfv.tp.dict.as_ref()).internals.map }.get(
        stringobject::string_from(selfv.vm.clone(), "call".to_string()),
    );
    if call.is_some() {
        let call_fn = unwrap_fast!(call).tp.call;
        if call_fn.is_none() {
            return MethodValue::Error(methodnotdefinedexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Method 'call' is not defined for '{}' type",
                    unwrap_fast!(call).tp.typename
                ),
                Position::default(),
                Position::default(),
            ));
        }
        let mut selfv_vec = vec![selfv.clone()];
        selfv_vec.extend(unsafe { &args.internals.arr }.iter().map(|arg| arg.clone()));
        let args = listobject::list_from(selfv.vm.clone(), selfv_vec);
        return (unwrap_fast!(call_fn))(unwrap_fast!(call), args);
    }
    MethodValue::Error(methodnotdefinedexc_from_str(
        selfv.vm.clone(),
        &format!(
            "Method 'len' is not defined for '{}' type",
            selfv.tp.typename
        ),
        Position::default(),
        Position::default(),
    ))
}

pub fn create_class<'a>(mut vm: Trc<VM<'a>>, name: String, dict: Object<'a>) -> Object<'a> {
    let tp = Trc::new(TypeObject {
        typename: name,
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,
        dict: Some(dict),

        new: None,
        del: Some(|_| {}),

        repr: Some(class_repr),
        str: Some(class_str),
        abs: Some(class_abs),
        neg: Some(class_neg),
        hash_fn: Some(class_hash),

        eq: Some(class_eq),
        add: Some(class_add),
        sub: Some(class_sub),
        mul: Some(class_mul),
        div: Some(class_div),
        pow: Some(class_pow),

        get: Some(class_get),
        set: Some(class_set),
        len: Some(class_len),

        call: Some(class_call),
    });

    vm.types.n_types += 1;

    finalize_type(tp.clone());

    let mut typobj = create_object_from_typeobject(vm.types.typetp.clone().unwrap(), vm);
    typobj.internals = ObjectInternals {
        typ: ManuallyDrop::new((*tp).clone()),
    };
    typobj
}
