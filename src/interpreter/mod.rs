// Interpret bytecode

use std::marker::PhantomData;
use std::{time::Instant};
use colored::Colorize;
use crate::Arc;
use crate::parser::Position;
use crate::{stats, objects::{Object, noneobject, utils::{object_repr, object_repr_safe}, fnobject, listobject, dictobject, exceptionobject, intobject, boolobject}, compiler::{CompilerInstruction, Bytecode, CompilerRegister}, fileinfo::FileInfo, TimeitHolder, none_from};

#[derive(Clone, PartialEq, Eq)]
pub struct Namespaces<'a> {
    locals: Vec<Object<'a>>,
    globals: Option<Object<'a>>,
    _marker: PhantomData<&'a ()>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Arguments<'a> {
    args: Vec<Object<'a>>,
    _marker: PhantomData<&'a ()>,
}

pub const MIN_INT_CACHE: i128 = -5;
pub const MAX_INT_CACHE: i128 = 256;
pub const INT_CACHE_SIZE: i128 = MAX_INT_CACHE-MIN_INT_CACHE;

#[derive(Clone)]
pub struct SingletonCache<'a> {
    pub int_cache: [Option<Object<'a>>; INT_CACHE_SIZE as usize],
    pub bool_cache: (Option<Object<'a>>, Option<Object<'a>>),
    pub none_singleton: Option<Object<'a>>,
    _marker: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct VM<'a> {
    pub types: Arc<hashbrown::HashMap<String, Object<'a>>>,
    pub interpreters: Vec<Arc<Interpreter<'a>>>,
    pub namespaces: Arc<Namespaces<'a>>,
    info: FileInfo<'a>,
    pub cache: SingletonCache<'a>,
}

impl<'a> VM<'a> {
    pub fn get_type(&self, name: &str) -> Object<'a> {
        return self.types.get(name).expect("Type not found").clone();
    }
    pub fn add_type(this: Arc<Self>, name: &str, value: Object<'a>) {
        unsafe {
            let refr = Arc::into_raw(this) as *mut VM<'a>;
            let map_refr = Arc::into_raw((*refr).types.clone()) as *mut hashbrown::HashMap<String, Object>;
            (*map_refr).insert(name.to_string(), value);
            Arc::from_raw(refr);
            Arc::from_raw(map_refr);
        }
    }
}

impl<'a> Eq for VM<'a> {}

impl<'a> PartialEq for VM<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.types == other.types && self.interpreters == other.interpreters && self.namespaces == other.namespaces
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Interpreter<'a> {
    frames: Vec<Frame<'a>>,
    types: Arc<hashbrown::HashMap<String, Object<'a>>>,
    namespaces: Arc<Namespaces<'a>>,
    vm: Arc<VM<'a>>,
}

#[derive(Clone, PartialEq, Eq)]
struct Frame<'a> {
    register1: Object<'a>,
    register2: Object<'a>,
    args: Vec<Arguments<'a>>,
}


macro_rules! read_register {
    ($interp:expr, $register:expr) => {
        match $register {
            CompilerRegister::R1 => {
                $interp.frames.last_mut().expect("No frames").register1.clone()
            }
            CompilerRegister::R2 => {
                $interp.frames.last_mut().expect("No frames").register2.clone()
            }
            CompilerRegister::NA => {
                unimplemented!("Cannot read from NA register");
            }
        }
    };
}

macro_rules! assign_to_register {
    ($interp:expr, $value:expr, $register:expr) => {
        match $register {
            CompilerRegister::R1 => {
                $interp.frames.last_mut().expect("No frames").register1 = $value;
            }
            CompilerRegister::R2 => {
                $interp.frames.last_mut().expect("No frames").register2 = $value;
            }
            CompilerRegister::NA => {
                unimplemented!("Cannot store to NA register");
            }
        }
    };
}

macro_rules! pop_frame {
    ($interp:expr) => {
        {
            unsafe {
                let namespace_refr = Arc::into_raw($interp.namespaces.clone()) as *mut Namespaces<'a>;
                (*namespace_refr).locals.pop();
                Arc::from_raw(namespace_refr);
            }
            $interp.frames.pop();
        }
    };
}

macro_rules! add_frame {
    ($interp:expr) => {
        {
            unsafe {
                let namespace_refr = Arc::into_raw($interp.namespaces.clone()) as *mut Namespaces<'a>;
                let dict = dictobject::dict_from($interp.vm.clone(), hashbrown::HashMap::with_capacity(4));
                (*namespace_refr).locals.push(dict.clone());
                
                if (*namespace_refr).globals.is_none() {
                    (*namespace_refr).globals = Some(dict);
                }
                Arc::from_raw(namespace_refr);
            }
            $interp.frames.push(Frame { register1: none_from!($interp.vm), register2: none_from!($interp.vm), args: Vec::new() })
        }
    };
}

impl<'a> VM<'a> {
    pub fn new(info: FileInfo<'a>) -> VM<'a> {
        let singleton = SingletonCache {
            int_cache: intobject::init_cache(),
            bool_cache: (None, None),
            none_singleton: None,
            _marker: PhantomData,
        };
        VM { types: Arc::new(hashbrown::HashMap::new()),
            interpreters: Vec::new(),
            namespaces: Arc::new(Namespaces { locals: Vec::new(), globals: None, _marker: PhantomData }),
            info,
            cache: singleton }
    }

    pub fn init_cache(this: Arc<Self>) {
        unsafe {
            let refr = Arc::into_raw(this.clone()) as *mut VM;
            let int_cache_arr_ref = &(*refr).cache.int_cache;
            let ptr = int_cache_arr_ref as *const [Option<Object>; INT_CACHE_SIZE as usize] as *mut [Option<Object>; INT_CACHE_SIZE as usize];
            intobject::generate_cache(this.get_type("int"), ptr);
    
            let bool_cache_tup_ref = &(*refr).cache.bool_cache;
            let ptr = bool_cache_tup_ref as *const (Option<Object>, Option<Object>) as *mut (Option<Object>, Option<Object>);
            boolobject::generate_cache(this.get_type("bool"), ptr);
            
            let none_obj_ref = &(*refr).cache.none_singleton;
            let ptr = none_obj_ref as *const Option<Object> as *mut Option<Object>;
            noneobject::generate_cache(this.get_type("NoneType"), ptr);
            
            Arc::from_raw(refr);
        }
    }

    pub fn execute(this: Arc<Self>, bytecode: Arc<Bytecode<'a>>) -> Object<'a> {
        let interpreter = Interpreter::new(this.types.clone(), this.namespaces.clone(), this.clone());
        
        let refr = Arc::into_raw(this.clone()) as *mut VM<'a>;
        unsafe {
            (*refr).interpreters.push(Arc::new(interpreter));
            let interp_refr = Arc::into_raw((*refr).interpreters.last().expect("No interpreters").clone()) as *mut Interpreter<'a>;

            Arc::from_raw(refr);
            Arc::from_raw(interp_refr);
            return (*interp_refr).run_interpreter(bytecode);
        }
    }
    
    pub fn execute_timeit(this: Arc<Self>, bytecode: Arc<Bytecode<'a>>, timeit: &mut TimeitHolder) -> Object<'a> {
        let refr = Arc::into_raw(this.clone()) as *mut VM<'a>;
        
        unsafe {
            let interp_refr = Arc::into_raw((*refr).interpreters.last().expect("No interpreters").clone()) as *mut Interpreter<'a>;
            
            //See bench.rs, this is a verys similar implementation (pub fn iter<T, F>(inner: &mut F) -> stats::Summary)

            let samples = &mut [0f64; 50];

            //Get initial result
            let mut res = (*interp_refr).run_interpreter(bytecode.clone());
            
            for p in &mut *samples {
                let start = Instant::now();
                for _ in 0..5 {
                    res = (*interp_refr).run_interpreter(bytecode.clone());
                }
                let delta = start.elapsed().as_nanos();
                let time: u128;
                if (delta as i128/5 as i128)-(timeit.baseline as i128) < 0{
                    time = 0;
                }
                else {
                    time = delta/5-timeit.baseline;                    
                }
                *p = time as f64;
            }
            Arc::from_raw(refr);
            Arc::from_raw(interp_refr);

            stats::winsorize(samples, 5.0);
            
            let sum: f64 = samples.iter().sum();

            timeit.time = sum/samples.len() as f64;
            
            res
        }
    }

    pub fn execute_vars(this: Arc<Self>, bytecode: Arc<Bytecode<'a>>, vars: Object<'a>) -> Object<'a> {
        let interpreter = Interpreter::new(this.types.clone(), this.namespaces.clone(), this.clone());
        unsafe {
            let refr = Arc::into_raw(this.clone()) as *mut VM<'a>;
            (*refr).interpreters.push(Arc::new(interpreter));
            let interp_refr = Arc::into_raw((*refr).interpreters.last().expect("No interpreters").clone()) as *mut Interpreter<'a>;
            
            let res = (*interp_refr).run_interpreter_vars(bytecode, vars);
            Arc::from_raw(refr);
            Arc::from_raw(interp_refr);
            return res;
        }
    }
    
    pub fn terminate(_: Arc<Self>) -> ! {
        //Clean up child threads here
        std::process::exit(1);
    }
}

impl<'a> Interpreter<'a> {
    pub fn new(types: Arc<hashbrown::HashMap<String, Object<'a>>>, namespaces: Arc<Namespaces<'a>>, vm: Arc<VM<'a>>) -> Interpreter<'a> {
        Interpreter { frames: Vec::new(), types, namespaces, vm }
    }
    

    #[allow(dead_code)]
    fn output_register(&mut self, register: CompilerRegister) {
        match register {
            CompilerRegister::R1 => {
                println!("{}", object_repr(&self.frames.last().expect("No frames").register1));
            }
            CompilerRegister::R2 => {
                println!("{}", object_repr(&self.frames.last().expect("No frames").register2));
            }
            CompilerRegister::NA => {
                unimplemented!("Cannot have NA register");
            }
        }
    }

    fn raise_exc(&mut self, exc_obj: Object<'a>) -> ! {
        let exc = exc_obj.internals.get_exc().expect("Expected exc internal value");
        self.raise_exc_pos(exc_obj, exc.start, exc.end);
    }
    
    fn raise_exc_pos(&mut self, exc_obj: Object<'a>, start: Position, end: Position) -> ! {
        let header: String = match object_repr_safe(&exc_obj) { crate::objects::MethodValue::Some(v) => {v}, _ => { unimplemented!() }};
        let location: String = format!("{}:{}:{}", self.vm.as_ref().info.name, start.line+1, start.startcol+1);
        println!("{}", header.red().bold());
        println!("{}", location.red());
        let lines = Vec::from_iter(self.vm.as_ref().info.data.split(|num| *num as char == '\n'));

        let snippet: String = format!("{}", String::from_utf8(lines.get(start.line).expect("Line index out of range").to_vec()).expect("utf8 conversion failed").blue());
        let mut arrows: String = String::new();
        for idx in 0..snippet.len() {
            if idx>=start.startcol && idx<end.endcol {
                arrows += "^";
            }
            else {
                arrows += " ";
            }
        }
        let linestr = (start.line+1).to_string().blue().bold();
        println!("{} | {}", linestr, snippet);
        println!("{} | {}", " ".repeat(linestr.len()), arrows.green());
        
        //Should this happen??
        VM::terminate(self.vm.clone());
    }

    pub fn run_interpreter_vars(&mut self, bytecode: Arc<Bytecode<'a>>, vars: Object<'a>) -> Object<'a> {
        add_frame!(self);
        unsafe {
            let namespace_refr = Arc::into_raw(self.namespaces.clone()) as *mut Namespaces<'a>;
            (*namespace_refr).locals.pop();
            (*namespace_refr).locals.push(vars);
            Arc::from_raw(namespace_refr);
        }
        self.run_interpreter(bytecode)
    }

    pub fn run_interpreter(&mut self, bytecode: Arc<Bytecode<'a>>) -> Object<'a> {
        if !bytecode.instructions.is_empty() {
            add_frame!(self);
            return self.run_interpreter_raw(bytecode);
        }
        none_from!(self.vm)
    }

    #[inline]
    pub fn run_interpreter_raw(&mut self, bytecode: Arc<Bytecode<'a>>) -> Object<'a> {
        for instruction in &bytecode.instructions {
            match instruction {
                //Constant loading
                CompilerInstruction::LoadConstR1{index, start: _, end: _} => {
                    self.frames.last_mut().expect("No frames").register1 = bytecode.consts.get(*index).expect("Bytecode consts index out of range").clone();
                }
                CompilerInstruction::LoadConstR2{index, start: _, end: _} => {
                    self.frames.last_mut().expect("No frames").register2 = bytecode.consts.get(*index).expect("Bytecode consts index out of range").clone();
                }

                //Binary operations
                CompilerInstruction::BinaryAdd{register: out, start, end} => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.add.is_some());
                    let res = (last.register1.add.expect("Method is not defined"))(last.register1.clone(), last.register2.clone());
                    maybe_handle_exception!(self, res, *start, *end);
                    assign_to_register!(self, res.unwrap(), *out);
                }
                CompilerInstruction::BinarySub{register: out, start, end} => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.sub.is_some());
                    let res = (last.register1.sub.expect("Method is not defined"))(last.register1.clone(), last.register2.clone());
                    maybe_handle_exception!(self, res, *start, *end);
                    assign_to_register!(self, res.unwrap(), *out);
                }
                CompilerInstruction::BinaryMul{register: out, start, end} => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.mul.is_some());
                    let res = (last.register1.mul.expect("Method is not defined"))(last.register1.clone(), last.register2.clone());
                    maybe_handle_exception!(self, res, *start, *end);
                    assign_to_register!(self, res.unwrap(), *out);
                }
                CompilerInstruction::BinaryDiv{register: out, start, end} => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.div.is_some());
                    let res = (last.register1.div.expect("Method is not defined"))(last.register1.clone(), last.register2.clone());
                    maybe_handle_exception!(self, res, *start, *end);
                    assign_to_register!(self, res.unwrap(), *out);
                }

                //Unary operations
                CompilerInstruction::UnaryNeg{register: out, start, end} => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.neg.is_some());
                    let res = (last.register1.neg.expect("Method is not defined"))(last.register1.clone());
                    maybe_handle_exception!(self, res, *start, *end);
                    assign_to_register!(self, res.unwrap(), *out);
                }

                //Variable manipulation
                CompilerInstruction::StoreName{idx, register, start: _, end: _} => {
                    (self.namespaces.locals.last().expect("No locals").set.expect("Method is not defined"))(self.namespaces.locals.last().expect("No locals").clone(), bytecode.names.get(*idx).expect("Bytecode names index out of range").clone(), read_register!(self, *register));
                }
                CompilerInstruction::LoadName{idx, register, start, end} => {
                    let name = bytecode.names.get(*idx).expect("Bytecode names index out of range").clone();
                    for local in self.namespaces.locals.len()-1..0 {
                        let map = self.namespaces.locals.get(local).unwrap().clone();
                        let out = map.internals.get_map().expect("Expected map internal value").get(&name);
                        
                        if let Some(v) = out {
                            if !matches!(register, CompilerRegister::NA) {
                                assign_to_register!(self, v.clone(), *register);
                            }
                        }     
                    }
                    
                    let exc = exceptionobject::nameexc_from_str(self.vm.clone(), &format!("Name '{}' not found  (searched in all locals and also globals)", name.internals.get_str().unwrap()), *start, *end);
                    self.raise_exc(exc);
                }
                CompilerInstruction::StoreGlobal{idx, register, start: _, end: _} => {
                    let globals = self.namespaces.globals.as_ref().unwrap().clone();
                    
                    globals.set.expect("Method is not defined")(globals, bytecode.names.get(*idx).expect("Bytecode names index out of range").clone(), read_register!(self, *register));
                }
                CompilerInstruction::LoadGlobal{idx, register, start, end} => {
                    let globals = self.namespaces.globals.as_ref().unwrap().clone();

                    let name = &bytecode.names.get(*idx).expect("Bytecode names index out of range").clone();

                    let out = globals.internals.get_map().expect("Expected map internal value").get(name);
                    match out {
                        Some(v) => {
                            if !matches!(register, CompilerRegister::NA) {
                                assign_to_register!(self, v.clone(), *register);
                            }
                        }
                        None => {
                            let exc = exceptionobject::nameexc_from_str(self.vm.clone(), &format!("Name '{}' not found (searched in globals)", name.internals.get_str().unwrap()), *start, *end);
                            self.raise_exc(exc);
                        }
                    }
                }

                //Functions, arguments
                CompilerInstruction::MakeFunction{nameidx, argsidx, codeidx, start: _, end: _} => {
                    let code = bytecode.consts.get(*codeidx).expect("Bytecode consts index out of range").clone();
                    let args = bytecode.consts.get(*argsidx).expect("Bytecode consts index out of range").clone();
                    let name = bytecode.names.get(*nameidx).expect("Bytecode names index out of range").clone();
                    let func = fnobject::fn_from(self.vm.clone(), code, args.internals.get_arr().expect("Expected arr internal value").clone(), name.internals.get_str().expect("Expected str internal value").clone());
                    assign_to_register!(self, func, CompilerRegister::R1);
                }
                CompilerInstruction::InitArgs{start: _, end: _} => {
                    self.frames.last_mut().expect("No frames").args.push(Arguments { args: Vec::new(), _marker: PhantomData });
                }
                CompilerInstruction::AddArgument{register, start: _, end: _} => {
                    let reg = read_register!(self, *register);
                    self.frames.last_mut().expect("No frames").args.last_mut().expect("No arguments prepared").args.push(reg);
                }
                CompilerInstruction::Call{callableregister, register, start: _, end: _} => {
                    let callable = read_register!(self, *callableregister);
                    let args = self.frames.last().expect("No frames").args.last().expect("No arguments for call");
                    debug_assert!(callable.call.is_some());
                    let value = (callable.call.expect("Method is not defined"))(callable, listobject::list_from(self.vm.clone(), args.args.clone()));
                    debug_assert!(value.is_some());
                    assign_to_register!(self, value.unwrap(), *register);
                }

                //Control flow
                CompilerInstruction::Return{register, start: _, end: _} => {
                    let res = read_register!(self, *register);
                    pop_frame!(self);
                    return res;
                }
            }
        }

        pop_frame!(self);

        none_from!(self.vm)
    }
}