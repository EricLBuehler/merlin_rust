use std::mem::ManuallyDrop;
use std::ops::Deref;

use crate::{compiler::Bytecode, interpreter::VM, parser::Position, unwrap_fast};
use trc::Trc;

use self::exceptionobject::{
    attrexc_from_str, methodnotdefinedexc_from_str, typemismatchexc_from_str,
};

pub mod mhash;

pub mod intobject;
pub mod objectobject;
pub mod typeobject;
#[macro_use]
pub mod noneobject;
pub mod boolobject;
pub mod classtype;
pub mod codeobject;
pub mod dictobject;
pub mod exceptionobject;
pub mod fnobject;
pub mod listobject;
pub mod methodobject;
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

pub struct RawObject<'a> {
    pub tp: Trc<TypeObject<'a>>,
    pub internals: ObjectInternals<'a>,
    pub dict: Option<Object<'a>>,
    pub vm: Trc<VM<'a>>,
}

#[macro_export]
macro_rules! is_type_exact {
    ($self:expr, $other:expr) => {
        $self.tp.typeid == $other.typeid
    };
}

#[allow(clippy::type_complexity)]
#[derive(Clone, Eq)]
pub struct TypeObject<'a> {
    pub typename: String,
    pub bases: Vec<ObjectBase<'a>>,
    pub typeid: u32,
    pub dict: Option<Object<'a>>,

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

    //attributes
    pub getattr: Option<fn(Object<'a>, Object<'a>) -> MethodType<'a>>, //self, attr
    pub setattr: Option<fn(Object<'a>, Object<'a>, Object<'a>) -> MethodType<'a>>, //self, attr
    pub descrget: Option<fn(Object<'a>, Option<Object<'a>>, Object<'a>) -> MethodType<'a>>, //self (the object), instance (None if the type of instance is not the owner, that is - the owner is the i), owner (the owning type)
    pub descrset: Option<fn(Object<'a>, Object<'a>, Object<'a>) -> MethodType<'a>>, //self, instance
}

impl<'a> Eq for RawObject<'a> {}

impl<'a> PartialEq for RawObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.tp == other.tp
    }
}

impl<'a> PartialEq for TypeObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.typename == other.typename && self.bases == other.bases
    }
}

impl<'a> Drop for RawObject<'a> {
    fn drop(&mut self) {
        unsafe { std::ptr::drop_in_place(&mut self.internals) };
    }
}

impl<'a> RawObject<'a> {
    pub fn object_repr(object: &Object<'_>) -> String {
        unsafe {
            &(object.clone().tp.repr.expect("Method is not defined"))(object.clone())
                .unwrap()
                .internals
                .str
        }
        .to_string()
    }

    #[allow(unused_unsafe)]
    pub fn object_repr_safe(object: Object<'_>) -> MethodValue<String, Object<'_>> {
        let repr = object.clone().tp.repr;
        if repr.is_none() {
            let exc = methodnotdefinedexc_from_str(
                object.vm.clone(),
                &format!(
                    "Method 'repr' is not defined for '{}' type",
                    object.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        let reprv = (unwrap_fast!(repr))(object.clone());

        if reprv.is_error() {
            return MethodValue::Error(reprv.unwrap_err());
        }

        if !is_type_exact!(
            &unwrap_fast!(reprv),
            unwrap_fast!(object.vm.types.strtp.as_ref()).clone()
        ) {
            let exc = typemismatchexc_from_str(
                object.vm.clone(),
                &format!(
                    "Method 'repr' of '{}' type returned non-string",
                    object.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        MethodValue::Some(unsafe { &unwrap_fast!(reprv).internals.str }.to_string())
    }

    #[allow(dead_code)]
    pub fn object_str(object: &Object<'_>) -> String {
        unsafe {
            &(object.clone().tp.str.expect("Method is not defined"))(object.clone())
                .unwrap()
                .internals
                .str
        }
        .to_string()
    }

    #[allow(unused_unsafe)]
    pub fn object_str_safe(object: Object<'_>) -> MethodValue<String, Object<'_>> {
        let str = object.clone().tp.str;
        if str.is_none() {
            let exc = methodnotdefinedexc_from_str(
                object.vm.clone(),
                &format!(
                    "Method 'str' is not defined for '{}' type",
                    object.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        let strv = (unwrap_fast!(str))(object.clone());

        if strv.is_error() {
            return MethodValue::Error(strv.unwrap_err());
        }

        if !is_type_exact!(
            &unwrap_fast!(strv),
            unwrap_fast!(object.vm.types.strtp.as_ref()).clone()
        ) {
            let exc = typemismatchexc_from_str(
                object.vm.clone(),
                &format!(
                    "Method 'str' of '{}' type returned non-string",
                    object.tp.typename
                ),
                Position::default(),
                Position::default(),
            );
            return MethodValue::Error(exc);
        }

        MethodValue::Some(unsafe { &unwrap_fast!(strv).internals.str }.to_string())
    }

    #[inline]
    fn generic_getattr(selfv: Object<'a>, attr: Object<'a>) -> MethodType<'a> {
        if selfv.dict.is_none() {
            let repr = RawObject::object_str_safe(attr);
            if repr.is_error() {
                return MethodValue::Error(repr.unwrap_err());
            }
            return MethodValue::Error(attrexc_from_str(
                selfv.vm.clone(),
                &format!(
                    "Object of type '{}' has no attribute '{}'",
                    selfv.tp.typename,
                    repr.unwrap(),
                ),
                Position::default(),
                Position::default(),
            ));
        } else {
            let res = selfv.dict.as_ref().unwrap().clone().tp.get.unwrap()(
                selfv.dict.as_ref().unwrap().clone(),
                attr.clone(),
            );
            if res.is_error()
                && is_type_exact!(
                    res.unwrap_err(),
                    selfv.vm.types.keyntfndexctp.as_ref().unwrap()
                )
            {
                let repr = RawObject::object_str_safe(attr);
                if repr.is_error() {
                    return MethodValue::Error(repr.unwrap_err());
                }
                return MethodValue::Error(attrexc_from_str(
                    selfv.vm.clone(),
                    &format!(
                        "Object of type '{}' has no attribute '{}'",
                        selfv.tp.typename,
                        repr.unwrap(),
                    ),
                    Position::default(),
                    Position::default(),
                ));
            }

            if unwrap_fast!(res).tp.descrget.is_some() {
                if is_type_exact!(selfv, unwrap_fast!(selfv.vm.types.typetp.as_ref()))
                    && Trc::ptr_eq(
                        selfv.dict.as_ref().unwrap(),
                        unsafe { &selfv.internals.typ }.dict.as_ref().unwrap(),
                    )
                {
                    return unwrap_fast!(res).tp.descrget.unwrap()(
                        unwrap_fast!(res).clone(),
                        None,
                        create_object_from_typeobject(selfv.vm.clone(), selfv.tp.clone()),
                    );
                }
                return unwrap_fast!(res).tp.descrget.unwrap()(
                    unwrap_fast!(res).clone(),
                    Some(selfv.clone()),
                    create_object_from_typeobject(selfv.vm.clone(), selfv.tp.clone()),
                );
            }

            res
        }
    }
}

pub type Object<'a> = Trc<RawObject<'a>>;
pub type MethodType<'a> = MethodValue<Object<'a>, Object<'a>>;

#[derive(Clone, PartialEq, Eq)]
pub struct FnData<'a> {
    code: Object<'a>,
    args: Vec<Object<'a>>,
    name: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct FnWrapper<'a> {
    fun: Object<'a>,
    instance: Object<'a>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ExcData<'a> {
    pub obj: Object<'a>,
    pub start: Position,
    pub end: Position,
}

pub union ObjectInternals<'a> {
    pub none: (),
    pub bool: bool,
    pub int: isize,
    pub str: ManuallyDrop<String>,
    pub arr: ManuallyDrop<Vec<Object<'a>>>,
    pub map: ManuallyDrop<mhash::HashMap<'a>>,
    pub code: ManuallyDrop<Trc<Bytecode<'a>>>,
    pub fun: ManuallyDrop<FnData<'a>>,
    pub exc: ManuallyDrop<ExcData<'a>>,
    pub typ: ManuallyDrop<TypeObject<'a>>,
    pub fn_wrapper: ManuallyDrop<FnWrapper<'a>>,
}

pub enum MethodValue<T, E> {
    Some(T),
    Error(E),
}

#[allow(dead_code)]
impl<T: Clone, E: Clone> MethodValue<T, E> {
    #[inline]
    pub unsafe fn unwrap_unchecked(&self) -> T {
        debug_assert!(self.is_some());
        match self {
            MethodValue::Some(v) => v.clone(),
            // SAFETY: the safety contract must be upheld by the caller.
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn unwrap(&self) -> T {
        match self {
            MethodValue::Some(v) => v.clone(),
            MethodValue::Error(_) => {
                panic!("Attempted to unwrap MethodValue with no value (got Error variant). ")
            }
        }
    }

    #[inline]
    pub fn unwrap_err(&self) -> E {
        match self {
            MethodValue::Some(_) => {
                panic!("Attempted to unwrap MethodValue that is not an error (got Some variant). ")
            }
            MethodValue::Error(v) => v.clone(),
        }
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self, MethodValue::Error(_))
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        matches!(self, MethodValue::Some(_))
    }
}

#[inline]
fn create_object_from_type<'a>(
    tp: Trc<TypeObject<'a>>,
    vm: Trc<VM<'a>>,
    dict: Option<Object<'a>>,
) -> Object<'a> {
    let raw = RawObject {
        vm: vm.clone(),
        tp,
        dict,
        internals: ObjectInternals { none: () },
    };
    Trc::new(raw)
}

#[inline]
fn create_object_from_typeobject<'a>(vm: Trc<VM<'a>>, tp: Trc<TypeObject<'a>>) -> Object<'a> {
    let raw = RawObject {
        vm: vm.clone(),
        tp: unwrap_fast!(vm.types.typetp.as_ref()).clone(),
        dict: tp.dict.clone(),
        internals: ObjectInternals {
            typ: ManuallyDrop::new((*tp).clone()),
        },
    };
    Trc::new(raw)
}

fn inherit_slots<'a>(mut tp: Trc<TypeObject<'a>>, basetp: TypeObject<'a>) {
    tp.new = if basetp.new.is_some() {
        basetp.new
    } else {
        tp.new
    };

    tp.repr = if basetp.repr.is_some() {
        basetp.repr
    } else {
        tp.repr
    };
    tp.abs = if basetp.abs.is_some() {
        basetp.abs
    } else {
        tp.abs
    };
    tp.neg = if basetp.neg.is_some() {
        basetp.neg
    } else {
        tp.neg
    };

    tp.eq = if basetp.eq.is_some() {
        basetp.eq
    } else {
        tp.eq
    };
    tp.add = if basetp.add.is_some() {
        basetp.add
    } else {
        tp.add
    };
    tp.sub = if basetp.sub.is_some() {
        basetp.sub
    } else {
        tp.sub
    };
    tp.mul = if basetp.mul.is_some() {
        basetp.mul
    } else {
        tp.mul
    };
    tp.div = if basetp.div.is_some() {
        basetp.div
    } else {
        tp.div
    };
    tp.pow = if basetp.pow.is_some() {
        basetp.pow
    } else {
        tp.pow
    };

    tp.get = if basetp.get.is_some() {
        basetp.get
    } else {
        tp.get
    };
    tp.set = if basetp.set.is_some() {
        basetp.set
    } else {
        tp.set
    };
    tp.len = if basetp.len.is_some() {
        basetp.len
    } else {
        tp.len
    };

    tp.call = if basetp.call.is_some() {
        basetp.call
    } else {
        tp.call
    };

    tp.getattr = if basetp.getattr.is_some() {
        basetp.getattr
    } else {
        tp.getattr
    };
    tp.setattr = if basetp.setattr.is_some() {
        basetp.setattr
    } else {
        tp.setattr
    };
    tp.descrget = if basetp.descrget.is_some() {
        basetp.descrget
    } else {
        tp.descrget
    };
    tp.descrset = if basetp.descrset.is_some() {
        basetp.descrset
    } else {
        tp.descrset
    };
}

fn finalize_type_dict(_tp: Trc<TypeObject<'_>>) {
    //TODO!
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
    exceptionobject::init_attrexc(vm.clone());
    methodobject::init(vm.clone());
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
