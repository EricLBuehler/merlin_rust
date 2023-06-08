use std::fmt::Debug;

use crate::objects::utils::object_repr;
use crate::trc::Trc;
use crate::{compiler::Bytecode, interpreter::VM, parser::Position};
pub mod mhash;

pub mod utils;

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

#[derive(Clone, PartialEq, Eq, Default)]
pub enum ObjectType<'a> {
    #[default]
    No,
    Type(Trc<VM<'a>>),
    Other(Object<'a>),
}

#[allow(dead_code)]
impl<'a> ObjectType<'a> {
    pub fn get_value(&self) -> Object<'a> {
        match self {
            ObjectType::Other(v) => v.clone(),
            ObjectType::Type(vm) => {
                let tp = vm.get_type("type");
                tp
            }
            _ => {
                unimplemented!();
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ObjectBase<'a> {
    Object(Trc<VM<'a>>),
    Other(Object<'a>),
}

#[allow(dead_code)]
impl<'a> ObjectBase<'a> {
    pub fn get_value(&self) -> Object<'a> {
        match self {
            ObjectBase::Other(v) => v.clone(),
            ObjectBase::Object(vm) => {
                let tp = vm.get_type("object");
                tp
            }
        }
    }
}

#[derive(Clone)]
pub struct RawObject<'a> {
    pub tp: ObjectType<'a>,
    pub internals: ObjectInternals<'a>,
    pub typename: String,
    pub bases: Vec<ObjectBase<'a>>,
    pub vm: Trc<VM<'a>>,

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
        self.tp == other.tp
            && self.typename == other.typename
            && self.internals == other.internals
            && self.bases == other.bases
    }
}

impl<'a> Debug for RawObject<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", object_repr(&Trc::new(self.clone())))
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
    Int(i128),
    Str(String),
    Arr(Vec<Object<'a>>),
    Map(mhash::HashMap<'a>),
    Code(Trc<Bytecode<'a>>),
    Fn(FnData<'a>),
    Exc(ExcData<'a>),
    None,
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
            ObjectInternals::Bool(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, ObjectInternals::Int(_))
    }
    #[inline]
    pub fn get_int(&self) -> Option<&i128> {
        match self {
            ObjectInternals::Int(v) => Some(v),
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
            ObjectInternals::Str(v) => Some(v),
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
            ObjectInternals::Arr(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, ObjectInternals::None)
    }
    #[inline]
    pub fn get_none(&self) -> Option<()> {
        match self {
            ObjectInternals::None => Some(()),
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
            ObjectInternals::Map(v) => Some(v),
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
            ObjectInternals::Code(v) => Some(v),
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
    pub fn get_exc(&self) -> Option<ExcData<'a>> {
        match self {
            ObjectInternals::Exc(v) => Some(v.clone()),
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
fn create_object_from_type(tp: Object<'_>) -> Object<'_> {
    let mut obj = (*tp).clone();
    obj.tp = ObjectType::Other(tp);
    Trc::new(obj)
}

#[macro_export]
macro_rules! is_type_exact {
    ($self:expr, $other:expr) => {
        $self.typename == $other.typename
    };
}

fn inherit_slots<'a>(mut tp: Object<'a>, basetp: Object<'a>) {
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

fn finalize_type(tp: Object<'_>) {
    let raw = (*tp).clone();
    let cpy = tp.clone();
    for base in cpy.bases.clone() {
        match base {
            ObjectBase::Other(basetp) => {
                inherit_slots(cpy.clone(), basetp);
            }
            ObjectBase::Object(_) => {
                let x = tp.vm.get_type("object");
                inherit_slots(cpy.clone(), x);
            }
        }
    }
    
    inherit_slots(cpy, Trc::new(raw));
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
