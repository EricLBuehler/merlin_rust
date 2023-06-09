use super::exceptionobject::typemismatchexc_from_str;
use super::{create_object_from_type, finalize_type, MethodType, MethodValue, Object, RawObject, TypeObject};
use crate::is_type_exact;
use crate::parser::Position;
use crate::trc::Trc;
use crate::{
    compiler::Bytecode,
    interpreter::VM,
    objects::{boolobject, stringobject, ObjectInternals},
};

pub fn code_from<'a>(vm: Trc<VM<'a>>, bytecode: Trc<Bytecode<'a>>) -> Object<'a> {
    let mut tp: Trc<RawObject> = create_object_from_type(vm.types.codetp.as_ref().unwrap().clone());
    tp.internals = ObjectInternals::Code(bytecode);
    tp
}

fn code_new<'a>(_selfv: Object<'a>, _args: Object<'a>, _kwargs: Object<'a>) -> MethodType<'a> {
    unimplemented!();
}
fn code_repr(selfv: Object<'_>) -> MethodType<'_> {
    MethodValue::Some(stringobject::string_from(
        selfv.tp.vm.clone(),
        format!("<code object @ 0x{:x}>", Trc::as_ptr(&selfv) as usize),
    ))
}
fn code_eq<'a>(selfv: Object<'a>, other: Object<'a>) -> MethodType<'a> {
    if !is_type_exact!(&selfv, other.tp) {
        let exc = typemismatchexc_from_str(
            selfv.tp.vm.clone(),
            "Types do not match",
            Position::default(),
            Position::default(),
        );
        return MethodValue::Error(exc);
    }

    MethodValue::Some(boolobject::bool_from(
        selfv.tp.vm.clone(),
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

pub fn init<'a>(mut vm: Trc<VM<'a>>) {
    let tp: Trc<TypeObject<'a>> = Trc::new(TypeObject {
        typename: String::from("code"),
        bases: vec![super::ObjectBase::Other(
            vm.types.objecttp.as_ref().unwrap().clone(),
        )],
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

    vm.types.codetp = Some(tp.clone());

    finalize_type(tp);
}
