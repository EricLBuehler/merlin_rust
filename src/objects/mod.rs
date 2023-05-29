use std::{sync::{Arc}, collections::{hash_map::DefaultHasher}, hash::{Hash, Hasher}};

use crate::{compiler::Bytecode, interpreter::VM, parser::Position};
use ahash::AHashMap;


pub mod utils;

pub mod objectobject;
pub mod typeobject;
pub mod intobject;
pub mod boolobject;
pub mod stringobject;
pub mod listobject;
pub mod noneobject;
pub mod dictobject;
pub mod codeobject;
pub mod fnobject;
pub mod exceptionobject;

#[derive(Clone, PartialEq, Eq, Default)]
pub enum ObjectType<'a> {
    #[default]
    No,
    Type(Arc<VM<'a>>),
    Other(Object<'a>)
}

#[allow(dead_code)]
impl<'a> ObjectType<'a> {
    pub fn get_value(&self) -> Object<'a> {
        match self {
            ObjectType::Other(v) => {
                v.clone()
            }
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
    Object(Arc<VM<'a>>),
    Other(Object<'a>)
}

#[allow(dead_code)]
impl<'a> ObjectBase<'a> {
    pub fn get_value(&self) -> Object<'a> {
        match self {
            ObjectBase::Other(v) => {
                v.clone()
            }
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
    pub vm: Arc<VM<'a>>,

    //instantiation
    pub new: Option<fn(Object<'a>, Object<'a>, Object<'a>) -> MethodType<'a>>, //self, args, kwargs
    
    //unary
    pub repr: Option<fn(Object<'a>) -> MethodType<'a>>, //self
    pub str: Option<fn(Object<'a>) -> MethodType<'a>>, //self
    pub abs: Option<fn(Object<'a>) -> MethodType<'a>>, //self
    pub neg: Option<fn(Object<'a>) -> MethodType<'a>>, //self
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
    pub len: Option<fn(Object<'a>) -> MethodType<'a>>, //self

    //interaction
    pub call: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, args
}

impl<'a> Eq for RawObject<'a> {}

impl<'a> PartialEq for RawObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.tp == other.tp &&
                self.typename == other.typename &&
                self.internals == other.internals &&
                self.bases == other.bases
    }
}

impl<'a> Hash for RawObject<'a> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        debug_assert!(self.hash_fn.is_some());
        let res = (self.hash_fn.expect("Hash function not found"))(Arc::new(self.clone()));
        debug_assert!(res.is_some());
        debug_assert!(is_instance(&res.unwrap(), &self.vm.get_type("int")));
        ////println!("{} {}", self.internals.get_str().unwrap(), *res.unwrap().internals.get_int().expect("Expected int internal value"));
        state.write_i128(*res.unwrap().internals.get_int().expect("Expected int internal value"));
    }
}

pub type Object<'a> = Arc<RawObject<'a>>;
pub type MethodType<'a> = MethodValue<Object<'a>, Object<'a>>;

#[derive(Clone, PartialEq, Eq)]
pub struct FnData<'a> {
    code: Object<'a>,
    args: Vec<Object<'a>>,
    name: String,
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
    Map(AHashMap<Object<'a>, Object<'a>>),
    Code(Arc<Bytecode<'a>>),
    Fn(FnData<'a>),
    Exc(ExcData<'a>),
    None,
}

#[allow(dead_code)]
impl<'a> ObjectInternals<'a> {
    pub fn is_no(&self) -> bool {
        matches!(self, ObjectInternals::No)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, ObjectInternals::Bool(_))
    }
    pub fn get_bool(&self) -> Option<&bool> {
        match self {
            ObjectInternals::Bool(v) => {
                Some(v)
            }
            _ => {
                None
            }
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, ObjectInternals::Int(_))
    }
    pub fn get_int(&self) -> Option<&i128> {
        match self {
            ObjectInternals::Int(v) => {
                Some(v)
            }
            _ => {
                None
            }
        }
    }

    pub fn is_str(&self) -> bool {
        matches!(self, ObjectInternals::Str(_))
    }
    pub fn get_str(&self) -> Option<&String> {
        match self {
            ObjectInternals::Str(v) => {
                Some(v)
            }
            _ => {
                None
            }
        }
    }

    pub fn is_arr(&self) -> bool {
        matches!(self, ObjectInternals::Arr(_))
    }
    pub fn get_arr(&self) -> Option<&Vec<Object<'a>>> {
        match self {
            ObjectInternals::Arr(v) => {
                Some(v)
            }
            _ => {
                None
            }
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, ObjectInternals::None)
    }
    pub fn get_none(&self) -> Option<()> {
        match self {
            ObjectInternals::None => {
                Some(())
            }
            _ => {
                None
            }
        }
    }

    pub fn is_map(&self) -> bool {
        matches!(self, ObjectInternals::Map(_))
    }
    pub fn get_map(&self) -> Option<&AHashMap<Object<'a>, Object<'a>>> {
        match self {
            ObjectInternals::Map(v) => {
                Some(v)
            }
            _ => {
                None
            }
        }
    }

    pub fn is_code(&self) -> bool {
        matches!(self, ObjectInternals::Code(_))
    }
    pub fn get_code(&self) -> Option<&Bytecode<'a>> {
        match self {
            ObjectInternals::Code(v) => {
                Some(v)
            }
            _ => {
                None
            }
        }
    }

    pub fn is_fn(&self) -> bool {
        matches!(self, ObjectInternals::Fn(_))
    }
    pub fn get_fn(&self) -> Option<&FnData<'a>> {
        match self {
            ObjectInternals::Fn(v) => {
                Some(v)
            }
            _ => {
                None
            }
        }
    }

    pub fn is_exc(&self) -> bool {
        matches!(self, ObjectInternals::Exc(_))
    }
    pub fn get_exc(&self) -> Option<ExcData<'a>> {
        match self {
            ObjectInternals::Exc(v) => {
                Some(v.clone())
            }
            _ => {
                None
            }
        }
    }
}

pub enum MethodValue<T, E>{
    Some(T),
    NotImplemented,
    Error(E),
}

#[allow(dead_code)]
impl<T: Clone, E: Clone> MethodValue<T, E> {
    pub fn unwrap(&self) -> T{
        match self {
            MethodValue::Some(v) => {
                v.clone()
            }
            MethodValue::NotImplemented => {
                panic!("Attempted to unwrap MethodValue with no value (got NotImplemented variant). ")
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
            MethodValue::Error(v) => {
                v.clone()
            }
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

#[inline(always)]
fn create_object_from_type(tp: Object<'_>) -> Object<'_> {
    let mut tp = tp.clone();
    let alt = tp.clone();
    
    let mut refr = Arc::make_mut(&mut tp);
    refr.tp = ObjectType::Other(alt);
    tp
}

#[inline(always)]
fn get_typeid(selfv: Object<'_>) -> u64 {
    let mut hasher = DefaultHasher::new();
    selfv.typename.hash(&mut hasher);
    hasher.finish()
}

#[inline(always)]
fn is_instance<'a>(selfv: &Object<'a>, other: &Object<'a>) -> bool {
    get_typeid(selfv.clone()) == get_typeid(other.clone())
}

fn inherit_slots<'a>(tp: &mut RawObject<'a>, basetp: Object<'a>) {
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
    let mut cpy = tp.clone();
    let refr = Arc::make_mut(&mut cpy);

    for base in refr.bases.clone() {
        match base {
            ObjectBase::Other(basetp) => {
                inherit_slots(refr, basetp);
            }
            ObjectBase::Object(_) => {
                inherit_slots(refr, tp.vm.get_type("object"));
            }
        }
    }

    inherit_slots(refr, tp);
}

pub fn init_types(vm: Arc<VM<'_>>) {
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
}
