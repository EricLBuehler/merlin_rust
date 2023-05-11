use std::{sync::{Arc, RwLock}, collections::{hash_map::DefaultHasher, HashMap}, hash::{Hash, Hasher}};
use self::{typeobject::TypeType, intobject::IntType, boolobject::BoolType, stringobject::StringType, listobject::ListType};

pub mod utils;

mod typeobject;
mod intobject;
mod boolobject;
mod stringobject;
mod listobject;

type Object = Arc<dyn ObjectTrait + Send + Sync>;

// -> MethodValue<Object> (None means no implementation)

pub enum MethodValue<T>{
    Some(T),
    NotImplemented,
    Error,
}

impl<T: Clone> MethodValue<T> {
    pub fn unwrap(&self) -> T{
        match self {
            MethodValue::Some(v) => {
                return v.clone();
            }
            MethodValue::NotImplemented => {
                panic!("Attempted to unwrap MethodValue with no value (got NotImplemented variant). ")
            }
            MethodValue::Error => {
                panic!("Attempted to unwrap MethodValue with no value (got Error variant). ")
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
        if let MethodValue::Error = self {
            return true;
        }
        return false;
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
    fn get_basic_repr(self: Arc<Self>) -> MethodValue<String>; //self
    fn get_type(self: Arc<Self>) -> Object; //self
    fn get_typeid(self: Arc<Self>) -> u64{
        let mut hasher = DefaultHasher::new();
        self.get_name().hash(&mut hasher);
        return hasher.finish();
    }
    fn get_bases(self: Arc<Self>) -> Object; //list, not inherited
    fn new(self: Arc<Self>, args: Object, kwargs: Object) -> MethodValue<Object>; //cls, args, kwargs
    fn repr(self: Arc<Self>) -> MethodValue<Object>; //self
    fn eq(self: Arc<Self>, _other: Object) -> MethodValue<Object>; //self, other
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

pub fn init_types() {
    TypeType::init();
    IntType::init();
    BoolType::init();
    StringType::init();
    ListType::init();
}
