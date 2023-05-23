use std::{sync::{Arc, RwLock}, collections::{hash_map::DefaultHasher, HashMap}, hash::{Hash, Hasher}};
use self::{typeobject::TypeType, intobject::IntType, boolobject::BoolType, stringobject::StringType, listobject::ListType, noneobject::NoneType};

pub mod utils;

pub mod typeobject;
pub mod intobject;
pub mod boolobject;
pub mod stringobject;
pub mod listobject;
pub mod noneobject;

pub type Object = Arc<dyn ObjectTrait + Send + Sync>;

#[derive(Clone)]
pub enum ObjectInternals {
    No,
    Bool(bool),
    Int(i128),
    Str(String),
    Arr(Vec<Object>),
    None,
}

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


pub trait ObjectTrait {
    fn get_name(self: Arc<Self>) -> String; //self
    fn get_raw(self: Arc<Self>) -> ObjectInternals { //self
        ObjectInternals::No
    }
    fn get_type(self: Arc<Self>) -> Object; //self
    fn get_typeid(self: Arc<Self>) -> u64 { //self
        let mut hasher = DefaultHasher::new();
        self.get_name().hash(&mut hasher);
        hasher.finish()
    }
    fn get_bases(self: Arc<Self>) -> Vec<Object>; //list, not inherited


    //true if type does not have internals and only uses its dict
    fn is_dict_inherit(self: Arc<Self>) -> bool {
        false
    }

    //instantiation
    fn new(self: Arc<Self>, args: Object, kwargs: Object) -> MethodValue<Object, Object> { //cls, args, kwargs
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.new(args.clone(), kwargs.clone());
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }
        
        MethodValue::NotImplemented
    }

    //unary
    fn repr(self: Arc<Self>) -> MethodValue<Object, Object> { //self
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.repr();
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
    }
    fn abs(self: Arc<Self>) -> MethodValue<Object, Object> { //self
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.abs();
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
    }
    fn neg(self: Arc<Self>) -> MethodValue<Object, Object> { //self
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.neg();
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
    }

    //binary
    fn eq(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> { //self, other
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.eq(other.clone());
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
    }
    fn add(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> { //self, other
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.add(other.clone());
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
    }
    fn sub(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> { //self, other
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.sub(other.clone());
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
    }
    fn mul(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> { //self, other
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.mul(other.clone());
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
    }
    fn div(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> { //self, other
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.div(other.clone());
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
    }
    fn pow(self: Arc<Self>, other: Object) -> MethodValue<Object, Object> { //self, other
        if self.clone().is_dict_inherit() {
            for base in self.get_bases() {
                let res = base.pow(other.clone());
                if res.is_some() {
                    return res;
                }
                debug_assert!(res.is_not_implemented());
            }
        }

        MethodValue::NotImplemented
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
    NoneType::init();

    let mut types = HashMap::new();
    for key in TYPES.read().unwrap().keys() {
        let typ = get_type(key);
        types.insert(key.clone(), typ);
    }

    types
}
