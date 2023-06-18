use std::fmt::Debug;
use std::ops::Deref;

use crate::{compiler::Bytecode, interpreter::VM, parser::Position, unwrap_fast};
use trc::Trc;
pub mod mhash;

pub mod intobject;
pub mod objectobject;
pub mod typeobject;
#[macro_use]
pub mod noneobject;
pub mod boolobject;
pub mod codeobject;
pub mod dictobject;
pub mod exceptionobject;
pub mod fnobject;
pub mod listobject;
pub mod stringobject;

#[derive(Clone, PartialEq, Eq)]
pub enum ObjectBase<'a> {
    Object(Trc<VM<'a>>),
    Other(Trc<TypeObject<'a>>),
}

impl<'a> Deref for ObjectBase<'a> {
    type Target = TypeObject<'a>;

    fn deref(&self) -> &Self::Target {
        match self {
            ObjectBase::Other(v) => v,
            ObjectBase::Object(vm) => unwrap_fast!(vm.types.objecttp.as_ref()),
        }
    }
}

#[derive(Clone)]
pub struct RawObject<'a> {
    pub tp: Trc<TypeObject<'a>>,
    pub internals: ObjectInternals<'a>,
    pub vm: Trc<VM<'a>>,
}

#[derive(Clone, Eq)]
pub struct TypeObject<'a> {
    pub typename: String,
    pub bases: Vec<ObjectBase<'a>>,
    pub typeid: u32,

    //instantiation
    pub new: Option<fn(Object<'a>, Object<'a>, Object<'a>) -> MethodType<'a>>, //self, args, kwargs

    //unary
    pub repr: Option<fn(Object<'a>) -> MethodType<'a>>, //self
    pub str: Option<fn(Object<'a>) -> MethodType<'a>>,  //self
    pub abs: Option<fn(Object<'a>) -> MethodType<'a>>,  //self
    pub neg: Option<fn(Object<'a>) -> MethodType<'a>>,  //self
    pub hash_fn: Option<fn(Object<'a>) -> MethodType<'a>>, //self

    //binary
    pub eq: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, other
    pub add: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, other
    pub sub: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, other
    pub mul: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, other
    pub div: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, other
    pub pow: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, other

    //sequences
    pub get: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, other
    pub set: Option<fn(Object<'a>, Object<'a>, Object<'a>) -> MethodType<'a>>, //self, other, value
    pub len: Option<fn(Object<'a>) -> MethodType<'a>>,             //self

    //interaction
    pub call: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, args
}

impl<'a> Eq for RawObject<'a> {}

impl<'a> PartialEq for RawObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.tp == other.tp && self.internals == other.internals
    }
}

impl<'a> PartialEq for TypeObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.typename == other.typename && self.bases == other.bases
    }
}

impl<'a> Debug for RawObject<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::object_repr(&Trc::new(self.clone())))
    }
}

impl<'a> RawObject<'a> {
    pub fn object_repr(object: &Object<'_>) -> String {
        return (object.clone().tp.repr.expect("Method is not defined"))(object.clone())
            .unwrap()
            .internals
            .get_str()
            .expect("Expected str internal value")
            .clone();
    }

    pub fn object_repr_safe(object: Object<'_>) -> MethodValue<String, Object<'_>> {
        let repr = object.clone().tp.repr;
        if repr.is_none() {
            return MethodValue::Error(stringobject::string_from(
                object.vm.clone(),
                String::from("__repr__ is not implemented."),
            ));
        }

        let reprv = (unwrap_fast!(repr))(object.clone());

        if reprv.is_error() {
            return MethodValue::Error(reprv.unwrap_err());
        }

        if reprv.is_not_implemented() {
            return MethodValue::Error(stringobject::string_from(
                object.vm.clone(),
                String::from("__repr__ is not implemented."),
            ));
        }

        if !unwrap_fast!(reprv).internals.is_str() {
            return MethodValue::Error(stringobject::string_from(
                object.vm.clone(),
                String::from("__repr__ returned non-string."),
            ));
        }

        return MethodValue::Some(
            unwrap_fast!(reprv)
                .internals
                .get_str()
                .expect("Expected str internal value")
                .to_owned(),
        );
    }

    #[allow(dead_code)]
    pub fn object_str(object: &Object<'_>) -> String {
        return (object.clone().tp.str.expect("Method is not defined"))(object.clone())
            .unwrap()
            .internals
            .get_str()
            .expect("Expected str internal value")
            .clone();
    }

    pub fn object_str_safe(object: Object<'_>) -> MethodValue<String, Object<'_>> {
        let str = object.clone().tp.str;
        if str.is_none() {
            return MethodValue::Error(stringobject::string_from(
                object.vm.clone(),
                String::from("__repr__ is not implemented."),
            ));
        }

        let strv = (unwrap_fast!(str))(object.clone());

        if strv.is_error() {
            return MethodValue::Error(strv.unwrap_err());
        }

        if strv.is_not_implemented() {
            return MethodValue::Error(stringobject::string_from(
                object.vm.clone(),
                String::from("__repr__ is not implemented."),
            ));
        }

        if !unwrap_fast!(strv).internals.is_str() {
            return MethodValue::Error(stringobject::string_from(
                object.vm.clone(),
                String::from("__repr__ returned non-string."),
            ));
        }

        return MethodValue::Some(
            unwrap_fast!(strv)
                .internals
                .get_str()
                .expect("Expected str internal value")
                .to_owned(),
        );
    }
}

pub type Object<'a> = Trc<RawObject<'a>>;
pub type MethodType<'a> = MethodValue<Object<'a>, Object<'a>>;

#[derive(Clone, PartialEq, Eq)]
pub struct FnData<'a> {
    code: Object<'a>,
    args: Vec<Object<'a>>,
    name: String,
    indices: Vec<Object<'a>>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ExcData<'a> {
    pub obj: Object<'a>,
    pub start: Position,
    pub end: Position,
}

#[derive(Clone, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ObjectInternals<'a> {
    #[default]
    No,
    Bool(bool),
    Int(isize),
    Str(String),
    Arr(Vec<Object<'a>>),
    Map(mhash::HashMap<'a>),
    Code(Trc<Bytecode<'a>>),
    Fn(FnData<'a>),
    Exc(ExcData<'a>),
    Type(TypeObject<'a>),
}

#[allow(dead_code)]
impl<'a> ObjectInternals<'a> {
    #[inline]
    pub fn is_no(&self) -> bool {
        matches!(self, ObjectInternals::No)
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, ObjectInternals::Bool(_))
    }
    #[inline]
    pub fn get_bool(&self) -> Option<&bool> {
        match self {
            ObjectInternals::Bool(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, ObjectInternals::Int(_))
    }
    #[inline]
    pub fn get_int(&self) -> Option<&isize> {
        match self {
            ObjectInternals::Int(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_str(&self) -> bool {
        matches!(self, ObjectInternals::Str(_))
    }
    #[inline]
    pub fn get_str(&self) -> Option<&String> {
        match self {
            ObjectInternals::Str(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_arr(&self) -> bool {
        matches!(self, ObjectInternals::Arr(_))
    }
    #[inline]
    pub fn get_arr(&self) -> Option<&Vec<Object<'a>>> {
        match self {
            ObjectInternals::Arr(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_map(&self) -> bool {
        matches!(self, ObjectInternals::Map(_))
    }
    #[inline]
    pub fn get_map(&self) -> Option<&mhash::HashMap<'a>> {
        match self {
            ObjectInternals::Map(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_code(&self) -> bool {
        matches!(self, ObjectInternals::Code(_))
    }
    #[inline]
    pub fn get_code(&self) -> Option<&Bytecode<'a>> {
        match self {
            ObjectInternals::Code(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_fn(&self) -> bool {
        matches!(self, ObjectInternals::Fn(_))
    }
    #[inline]
    pub fn get_fn(&self) -> Option<&FnData<'a>> {
        match self {
            ObjectInternals::Fn(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_exc(&self) -> bool {
        matches!(self, ObjectInternals::Exc(_))
    }
    #[inline]
    pub fn get_exc(&self) -> Option<&ExcData<'a>> {
        match self {
            ObjectInternals::Exc(ref v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_type(&self) -> bool {
        matches!(self, ObjectInternals::Type(_))
    }
    #[inline]
    pub fn get_type(&self) -> Option<&TypeObject<'a>> {
        match self {
            ObjectInternals::Type(ref v) => Some(v),
            _ => None,
        }
    }
}

pub enum MethodValue<T, E> {
    Some(T),
    NotImplemented,
    Error(E),
}

#[allow(dead_code)]
impl<T: Clone, E: Clone> MethodValue<T, E> {
    pub unsafe fn unwrap_unchecked(&self) -> T {
        debug_assert!(self.is_some());
        match self {
            MethodValue::Some(v) => v.clone(),
            // SAFETY: the safety contract must be upheld by the caller.
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    pub fn unwrap(&self) -> T {
        match self {
            MethodValue::Some(v) => v.clone(),
            MethodValue::NotImplemented => {
                panic!(
                    "Attempted to unwrap MethodValue with no value (got NotImplemented variant). "
                )
            }
            MethodValue::Error(_) => {
                panic!("Attempted to unwrap MethodValue with no value (got Error variant). ")
            }
        }
    }

    pub fn unwrap_err(&self) -> E {
        match self {
            MethodValue::Some(_) => {
                panic!("Attempted to unwrap MethodValue that is not an error (got Some variant). ")
            }
            MethodValue::NotImplemented => {
                panic!("Attempted to unwrap MethodValue that is not an error (got NotImplemented variant). ")
            }
            MethodValue::Error(v) => v.clone(),
        }
    }

    pub fn is_not_implemented(&self) -> bool {
        matches!(self, MethodValue::NotImplemented)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, MethodValue::Error(_))
    }

    pub fn is_some(&self) -> bool {
        matches!(self, MethodValue::Some(_))
    }
}

#[inline]
fn create_object_from_type<'a>(tp: Trc<TypeObject<'a>>, vm: Trc<VM<'a>>) -> Object<'a> {
    let raw = RawObject {
        vm: vm.clone(),
        tp,
        internals: ObjectInternals::No,
    };
    Trc::new(raw)
}

#[macro_export]
macro_rules! is_type_exact {
    ($self:expr, $other:expr) => {
        $self.tp.typeid == $other.typeid
    };
}

fn inherit_slots<'a>(mut tp: Trc<TypeObject<'a>>, basetp: TypeObject<'a>) {
    tp.new = basetp.new;

    tp.repr = basetp.repr;
    tp.abs = basetp.abs;
    tp.neg = basetp.neg;

    tp.eq = basetp.eq;
    tp.add = basetp.add;
    tp.sub = basetp.sub;
    tp.mul = basetp.mul;
    tp.div = basetp.div;
    tp.pow = basetp.pow;

    tp.get = basetp.get;
    tp.set = basetp.set;
    tp.len = basetp.len;
}

fn finalize_type(tp: Trc<TypeObject<'_>>) {
    let raw = (*tp).clone();
    let cpy = tp.clone();
    for base in cpy.bases.clone() {
        inherit_slots(cpy.clone(), (*base).clone());
    }

    inherit_slots(cpy, raw);
}

pub fn init_types(vm: Trc<VM<'_>>) {
    objectobject::init(vm.clone());
    typeobject::init(vm.clone());
    intobject::init(vm.clone());
    boolobject::init(vm.clone());
    stringobject::init(vm.clone());
    listobject::init(vm.clone());
    noneobject::init(vm.clone());
    dictobject::init(vm.clone());
    codeobject::init(vm.clone());
    fnobject::init(vm.clone());
    exceptionobject::init_exc(vm.clone());
    exceptionobject::init_nameexc(vm.clone());
    exceptionobject::init_overflowexc(vm.clone());
    exceptionobject::init_methodnotdefinedexc(vm.clone());
    exceptionobject::init_typemismatchexc(vm.clone());
    exceptionobject::init_keynotfoundexc(vm.clone());
    exceptionobject::init_valueexc(vm.clone());
    exceptionobject::init_zerodivexc(vm.clone());
}

macro_rules! maybe_handle_exception {
    ($self:ident, $res:ident, $bytecode:expr, $i:expr) => {
        if $res.is_error() {
            let pos = $bytecode
                .positions
                .get($i)
                .expect("Instruction out of range");
            let exc = $res.unwrap_err();
            $self.raise_exc_pos(exc, pos.0, pos.1);
        }
    };
}

macro_rules! maybe_handle_exception_pos {
    ($self:ident, $res:ident, $start:expr, $end:expr) => {
        if $res.is_error() {
            let exc = $res.unwrap_err();
            $self.raise_exc_pos(exc, $start, $end);
        }
    };
}
