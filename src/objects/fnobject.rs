use super::exceptionobject::valueexc_from_str;
use super::{create_object_from_type, finalize_type, MethodType, MethodValue, Object, TypeObject};

use crate::is_type_exact;
use crate::objects::exceptionobject::typemismatchexc_from_str;
use crate::parser::Position;
use crate::unwrap_fast;
use crate::{
    interpreter::VM,
    objects::{boolobject, stringobject, ObjectInternals},
};
use itertools::izip;
use trc::Trc;

pub fn fn_from<'a>(
    vm: Trc<VM<'a>>,
    code: Object<'a>,
    args: Vec<Object<'a>>,
    indices: Vec<Object<'a>>,
    name: String,
) -> Object<'a> {
    let mut tp = create_object_from_type(unwrap_fast!(vm.types.fntp.as_ref()).clone(), vm);
    tp.internals = ObjectInternals::Fn(super::FnData {
        code,
        args,
        name,
        indices,
    });
    tp
}

fn fn_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn fn_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!(
            "<fn '{}' @ 0x{:x}>",
            selfv
                .internals
                .get_fn()
                .expect("Expected Fn internal value")
                .name,
            Trc::as_ptr(&selfv) as usize
        ),
    ))
}
fn fn_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        return MethodValue::Some(boolobject::bool_from(selfv.vm.clone(), false));
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        selfv
            .internals
            .get_fn()
            .expect("Expected Fn internal value")
            == other
                .internals
                .get_fn()
                .expect("Expected Fn internal value"),
    ))
}

fn fn_call<'a>(selfv: Object<'a>, args: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&args, unwrap_fast!(selfv.vm.types.listtp.as_ref()).clone()) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Expected args to be a 'list'",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    if args
        .internals
        .get_arr()
        .expect("Expected arr internal value")
        .len()
        != selfv
            .internals
            .get_fn()
            .expect("Expected Fn internal value")
            .args
            .len()
    {
        let exc = valueexc_from_str(
            selfv.vm.clone(),
            &format!(
                "Expected {} arguments, got {}",
                args.internals
                    .get_arr()
                    .expect("Expected arr internal value")
                    .len(),
                selfv
                    .internals
                    .get_fn()
                    .expect("Expected Fn internal value")
                    .args
                    .len()
            ),
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }
    let mut map = hashbrown::HashMap::new();
    for (value, index) in izip!(
        args.internals
            .get_arr()
            .expect("Expected arr internal value"),
        &selfv
            .internals
            .get_fn()
            .expect("Expected Fn internal value")
            .indices,
    ) {
        map.insert(
            index
                .internals
                .get_int()
                .expect("Expected int internal value"),
            value.clone(),
        );
    }

    let code = selfv
        .internals
        .get_fn()
        .expect("Expected Fn internal value")
        .code
        .internals
        .get_code()
        .expect("Expected Bytecode internal value");
    MethodValue::Some(VM::execute_vars(
        selfv.vm.clone(),
        Trc::new(code.clone()),
        map,
    ))
}

pub fn init(mut vm: Trc<VM<'_>>) {
    let tp = Trc::new(TypeObject {
        typename: String::from("fn"),
        bases: vec![super::ObjectBase::Other(
            unwrap_fast!(vm.types.objecttp.as_ref()).clone(),
        )],
        typeid: vm.types.n_types,

        new: Some(fn_new),

        repr: Some(fn_repr),
        str: Some(fn_repr),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(fn_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: Some(fn_call),
    });

    vm.types.fntp = Some(tp.clone());
    vm.types.n_types += 1;

    finalize_type(tp);
}
