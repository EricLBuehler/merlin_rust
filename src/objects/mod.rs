use std::{sync::{Arc, RwLock}, collections::{hash_map::DefaultHasher, HashMap}, hash::{Hash, Hasher}};


pub mod utils;

pub mod objectobject;
pub mod typeobject;
pub mod intobject;
pub mod boolobject;
pub mod stringobject;
pub mod listobject;
pub mod noneobject;

#[derive(Clone)]
pub enum ObjectType {
    Type,
    Other(Object)
}

#[allow(dead_code)]
impl ObjectType {
    pub fn get_value(&self) -> Object {
        match self {
            ObjectType::Other(v) => {
                return v.clone();
            }
            _ => {
                let tp = get_type("type");
                return tp;
            }
        }
    }
}

#[derive(Clone)]
pub enum ObjectBase {
    Object,
    Other(Object)
}

impl ObjectBase {
    pub fn get_value(&self) -> Object {
        match self {
            ObjectBase::Other(v) => {
                return v.clone();
            }
            _ => {
                let tp = get_type("object");
                return tp;
            }
        }
    }
}

#[derive(Clone)]
pub struct RawObject {
    pub tp: ObjectType,
    pub internals: ObjectInternals,
    pub typename: String,
    pub bases: Vec<ObjectBase>,

    pub new: Option<fn(Object, Object, Object) -> MethodValue<Object, Object>>, //self, args, kwargs
    
    pub repr: Option<fn(Object,) -> MethodValue<Object, Object>>, //self
    pub abs: Option<fn(Object) -> MethodValue<Object, Object>>, //self, other
    pub neg: Option<fn(Object) -> MethodValue<Object, Object>>, //self, other

    pub eq: Option<fn(Object, Object) -> MethodValue<Object, Object>>, //self, other
    pub add: Option<fn(Object, Object) -> MethodValue<Object, Object>>, //self, other
    pub sub: Option<fn(Object, Object) -> MethodValue<Object, Object>>, //self, other
    pub mul: Option<fn(Object, Object) -> MethodValue<Object, Object>>, //self, other
    pub div: Option<fn(Object, Object) -> MethodValue<Object, Object>>, //self, other
    pub pow: Option<fn(Object, Object) -> MethodValue<Object, Object>>, //self, other
}
pub type Object = Arc<RawObject>;

#[derive(Clone, Default)]
#[allow(dead_code)]
pub enum ObjectInternals {
    #[default]
    No,
    Bool(bool),
    Int(i128),
    Str(String),
    Arr(Vec<Object>),
    None,
}

#[allow(dead_code)]
impl ObjectInternals {
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
    pub fn get_arr(&self) -> Option<&Vec<Object>> {
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

lazy_static! {
    pub static ref TYPES: RwLock<HashMap<String, Object>> = RwLock::new(HashMap::new());
}

//helper functions
pub fn get_type(key: &str) -> Object {
    TYPES.read().unwrap().get(key).unwrap().clone()
}
fn add_type(key: &str, obj: Object) {
    TYPES.write().unwrap().insert(key.to_string(), obj);
}

fn create_object_from_type(tp: Object) -> Object {
    let mut tp = tp.clone();
    let alt = tp.clone();
    
    let mut refr = Arc::make_mut(&mut tp);
    refr.tp = ObjectType::Other(alt);
    tp
}

fn get_typeid(selfv: Object) -> u64 {
    let mut hasher = DefaultHasher::new();
    selfv.typename.hash(&mut hasher);
    hasher.finish()
}

fn is_instance(selfv: &Object, other: &Object) -> bool {
    return get_typeid(selfv.clone()) == get_typeid(other.clone());
}

fn inherit_slots(tp: &mut RawObject, basetp: Object) {
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
}

fn finalize_type(tp: Object) {
    let mut cpy = tp.clone();
    let refr = Arc::make_mut(&mut cpy);

    for base in refr.bases.clone() {
        match base {
            ObjectBase::Other(basetp) => {
                inherit_slots(refr, basetp);
            }
            ObjectBase::Object => {
                inherit_slots(refr, get_type("object"));
            }
        }
    }

    inherit_slots(refr, tp);
}

pub fn init_types() -> HashMap<String, Object> {
    objectobject::init();
    typeobject::init();
    intobject::init();
    boolobject::init();
    stringobject::init();
    listobject::init();
    noneobject::init();

    let mut types = HashMap::new();
    for key in TYPES.read().unwrap().keys() {
        let typ = get_type(key);
        types.insert(key.clone(), typ);
    }

    types
}
