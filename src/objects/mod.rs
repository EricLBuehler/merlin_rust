use std::{sync::{Arc, RwLock}, collections::{hash_map::DefaultHasher, HashMap}, hash::{Hash, Hasher}};
use self::{typeobject::TypeType, intobject::IntType, boolobject::BoolType, stringobject::StringType, listobject::ListType};

pub mod utils;

pub mod typeobject;
pub mod intobject;
pub mod boolobject;
pub mod stringobject;
pub mod listobject;

type Object = Arc<dyn ObjectTrait + Send + Sync>;

#[derive(Clone)]
pub enum ObjectInternals {
    No,
    Bool(bool),
    Int(i128),
    Str(String),
    Arr(Vec<Object>),
}

impl ObjectInternals {
    pub fn is_no(&self) -> bool {
        match self {
            ObjectInternals::No => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            ObjectInternals::Bool(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
    pub fn get_bool(&self) -> Option<&bool> {
        match self {
            ObjectInternals::Bool(v) => {
                return Some(v);
            }
            _ => {
                return None;
            }
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            ObjectInternals::Int(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
    pub fn get_int(&self) -> Option<&i128> {
        match self {
            ObjectInternals::Int(v) => {
                return Some(v);
            }
            _ => {
                return None;
            }
        }
    }

    pub fn is_str(&self) -> bool {
        match self {
            ObjectInternals::Str(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
    pub fn get_str(&self) -> Option<&String> {
        match self {
            ObjectInternals::Str(v) => {
                return Some(v);
            }
            _ => {
                return None;
            }
        }
    }

    pub fn is_arr(&self) -> bool {
        match self {
            ObjectInternals::Arr(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
    pub fn get_arr(&self) -> Option<&Vec<Object>> {
        match self {
            ObjectInternals::Arr(v) => {
                return Some(v);
            }
            _ => {
                return None;
            }
        }
    }
}

pub enum MethodValue<T, E>{
    Some(T),
    NotImplemented,
    Error(E),
}

impl<T: Clone, E: Clone> MethodValue<T, E> {
    pub fn unwrap(&self) -> T{
        match self {
            MethodValue::Some(v) => {
                return v.clone();
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
                return v.clone()
            }
        }
    }

    pub fn is_not_implemented(&self) -> bool {
        if let MethodValue::NotImplemented = self {
            return true;
        }
        return false;
    }

    pub fn is_error(&self) -> bool {
        match self {
            MethodValue::Error(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            MethodValue::Some(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
}

pub trait ObjectTrait {
    fn get_name(self: Arc<Self>) -> String; //self
    fn get_raw(self: Arc<Self>) -> MethodValue<ObjectInternals, Object> { //self
        return MethodValue::NotImplemented;
    }
    fn get_type(self: Arc<Self>) -> Object; //self
    fn get_typeid(self: Arc<Self>) -> u64 { //self
        let mut hasher = DefaultHasher::new();
        self.get_name().hash(&mut hasher);
        return hasher.finish();
    }
    fn get_bases(self: Arc<Self>) -> Object; //list, not inherited
    fn new(self: Arc<Self>, _args: Object, _kwargs: Object) -> MethodValue<Object, Object> { //cls, args, kwargs
        return MethodValue::NotImplemented;
    }
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> { //self
        return MethodValue::NotImplemented;
    }
    fn eq(self: Arc<Self>, _other: Object) -> MethodValue<Object, Object> { //self, other
        return MethodValue::NotImplemented;
    }
}

lazy_static! {
    pub static ref TYPES: RwLock<HashMap<String, Object>> = RwLock::new(HashMap::new());
}

pub fn get_type(key: &str) -> Object {
    return TYPES.read().unwrap().get(key).unwrap().clone();
}
fn add_type(key: &str, obj: Object) {
    TYPES.write().unwrap().insert(key.to_string(), obj);
}

pub fn init_types() -> HashMap<String, Object> {
    TypeType::init();
    IntType::init();
    BoolType::init();
    StringType::init();
    ListType::init();

    let mut types = HashMap::new();
    for key in TYPES.read().unwrap().keys() {
        let typ = get_type(key);
        types.insert(key.clone(), typ);
    }

    return types;
}
