use super::exceptionobject::typemismatchexc_from_str;
use super::{create_object_from_type, finalize_type, MethodType, MethodValue, Object, RawObject};
use crate::is_type_exact;
use crate::parser::Position;
use crate::trc::Trc;
use crate::{
    compiler::Bytecode,
    interpreter::VM,
    objects::{boolobject, stringobject, ObjectInternals},
};

pub fn code_from<'a>(vm: Trc<VM<'a>>, bytecode: Trc<Bytecode<'a>>) -> Object<'a> {
    let mut tp: Trc<RawObject> = create_object_from_type(vm.get_type("code"));
    tp.internals = ObjectInternals::Code(bytecode);
    tp
}

fn code_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn code_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.vm.clone(),
        format!("<code object @ 0x{:x}>", Trc::as_ptr(&selfv) as usize),
    ))
}
fn code_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, &other) {
        let exc = typemismatchexc_from_str(
            selfv.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.vm.clone(),
        selfv
            .internals
            .get_code()
            .expect("Expected Bytecode internal value")
            == other
                .internals
                .get_code()
                .expect("Expected Bytecode internal value"),
    ))
}

pub fn init<'a>(vm: Trc<VM<'a>>) {
    let tp: Trc<RawObject<'a>> = Trc::new(RawObject {
        tp: super::ObjectType::Other(vm.get_type("type")),
        internals: super::ObjectInternals::No,
        typename: String::from("code"),
        bases: vec![super::ObjectBase::Other(vm.get_type("object"))],
        vm: vm.clone(),

        new: Some(code_new),

        repr: Some(code_repr),
        str: Some(code_repr),
        abs: None,
        neg: None,
        hash_fn: None,
        eq: Some(code_eq),
        add: None,
        sub: None,
        mul: None,
        div: None,
        pow: None,

        get: None,
        set: None,
        len: None,

        call: None,
    });

    VM::add_type(vm.clone(), &tp.clone().typename, tp.clone());

    finalize_type(tp);
}
