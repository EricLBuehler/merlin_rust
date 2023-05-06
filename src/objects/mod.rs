use std::{sync::{Arc, RwLock}, collections::{hash_map::DefaultHasher, HashMap}, hash::{Hash, Hasher}};
use self::{typeobject::TypeType, intobject::IntType, boolobject::BoolType, stringobject::StringType, listobject::ListType};

pub mod utils;

mod typeobject;
mod intobject;
mod boolobject;
mod stringobject;
mod listobject;

type Object = Arc<dyn ObjectTrait + Send + Sync>;

// -> Option<Object> (None means no implementation)

pub trait ObjectTrait {
    fn get_name(self: Arc<Self>) -> String; //self
    fn get_basic_repr(self: Arc<Self>) -> Option<String>; //self
    fn get_type(self: Arc<Self>) -> Object; //self
    fn get_typeid(self: Arc<Self>) -> u64{
        let mut hasher = DefaultHasher::new();
        self.get_name().hash(&mut hasher);
        return hasher.finish();
    }
    fn get_bases(self: Arc<Self>) -> Object; //list, not inherited
    fn new(self: Arc<Self>, args: Object, kwargs: Object) -> Option<Object> where Self: ObjectTrait +  Sized; //cls, args, kwargs
    fn repr(self: Arc<Self>) -> Option<Object>; //self
    fn eq(self: Arc<Self>, _other: Object) -> Option<Object>; //self, other
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
