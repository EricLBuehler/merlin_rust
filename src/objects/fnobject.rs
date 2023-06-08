use super::exceptionobject::valueexc_from_str;
use super::{create_object_from_type, finalize_type, MethodType, MethodValue, Object, RawObject};

use crate::is_type_exact;
use crate::objects::exceptionobject::typemismatchexc_from_str;
use crate::parser::Position;
use crate::trc::Trc;
use crate::{
    interpreter::VM,
    objects::{boolobject, stringobject, ObjectInternals},
};
use itertools::izip;

pub fn fn_from<'a>(
    vm: Trc<VM<'a>>,
    code: Object<'a>,
    args: Vec<Object<'a>>,
    indices: Vec<Object<'a>>,
    name: String,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.get_type("fn"));
    (*tp).internals = ObjectInternals::Fn(super::FnData {
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
            Trc::as_ptr(&selfv) as i128
        ),
    ))
}
fn fn_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
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
    if !is_type_exact!(&args, &selfv.vm.clone().get_type("list")) {
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

pub fn init<'a>(vm: Trc<VM<'a>>) {
    let tp: Trc<RawObject<'a>> = Trc::new(RawObject {
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("fn"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

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

    VM::add_type(vm.clone(), &tp.clone().typename, tp.clone());

    finalize_type(tp);
}
