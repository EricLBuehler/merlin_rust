use super::{create_object_from_type, finalize_type, MethodType, MethodValue, Object, RawObject};
use crate::Arc;
use crate::{
    interpreter::VM,
    objects::{boolobject, dictobject, is_instance, stringobject, ObjectInternals},
};

pub fn fn_from<'a>(
    vm: Arc<VM<'a>>,
    code: Object<'a>,
    args: Vec<Object<'a>>,
    name: String,
) -> Object<'a> {
    let mut tp = create_object_from_type(vm.get_type("fn"));
    let refr = Arc::make_mut(&mut tp);
    refr.internals = ObjectInternals::Fn(super::FnData { code, args, name });
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
            Arc::as_ptr(&selfv) as i128
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
    debug_assert!(is_instance(&args, &selfv.vm.clone().get_type("list")));

    debug_assert!(
        args.internals
            .get_arr()
            .expect("Expected arr internal value")
            .len()
            == selfv
                .internals
                .get_fn()
                .expect("Expected Fn internal value")
                .args
                .len()
    );
    let mut map = hashbrown::HashMap::new();
    for (name, value) in std::iter::zip(
        args.internals
            .get_arr()
            .expect("Expected arr internal value"),
        &selfv
            .internals
            .get_fn()
            .expect("Expected Fn internal value")
            .args,
    ) {
        map.insert(name.clone(), value.clone());
    }
    let vars = dictobject::dict_from(selfv.vm.clone(), map);
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
        Arc::new(code.clone()),
        vars,
    ))
}

pub fn init<'a>(vm: Arc<VM<'a>>) {
    let tp: Arc<RawObject<'a>> = Arc::new(RawObject {
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
