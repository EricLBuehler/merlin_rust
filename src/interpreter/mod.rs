use std::{collections::HashMap, sync::Arc};

// Interpret bytecode
use crate::{objects::{Object, noneobject, utils::object_repr, fnobject, listobject, dictobject}, compiler::{CompilerInstruction, Bytecode, CompilerRegister}};

#[derive(PartialEq, Eq)]
pub struct Namespaces<'a> {
    locals: Vec<Object<'a>>,
    globals: *mut Object<'a>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Arguments<'a> {
    args: Vec<Object<'a>>,
}


pub struct VM<'a> {
    pub types: Arc<HashMap<String, Object<'a>>>,
    interpreters: Vec<Arc<Interpreter<'a>>>,
    namespaces: Arc<Namespaces<'a>>,
}

impl<'a> VM<'a> {
    pub fn get_type(&self, name: &str) -> Object<'a> {
        return self.types.get(name).expect("Type not found").clone();
    }
    pub fn add_type(self: Arc<Self>, name: &str, value: Object<'a>) {
        unsafe {
            let refr = Arc::into_raw(self) as *mut VM<'a>;
            let map_refr = Arc::into_raw((*refr).types.clone()) as *mut HashMap<String, Object>;
            (*map_refr).insert(name.to_string(), value);
        }
    }
}

impl<'a> Eq for VM<'a> {}

impl<'a> PartialEq for VM<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.types == other.types && self.interpreters == other.interpreters && self.namespaces == other.namespaces
    }
}

#[derive(PartialEq, Eq)]
pub struct Interpreter<'a> {
    frames: Vec<Frame<'a>>,
    types: Arc<HashMap<String, Object<'a>>>,
    namespaces: Arc<Namespaces<'a>>,
    vm: Arc<VM<'a>>,
}

#[derive(Clone, PartialEq, Eq)]
struct Frame<'a> {
    register1: Object<'a>,
    register2: Object<'a>,
    args: Vec<Arguments<'a>>,
}

impl<'a> VM<'a> {
    pub fn new() -> VM<'a> {
        VM { types: Arc::new(HashMap::new()), interpreters: Vec::new(), namespaces: Arc::new(Namespaces { locals: Vec::new(), globals: std::ptr::null_mut()  }) }
    }

    pub fn execute(self: Arc<Self>, bytecode: Arc<Bytecode<'a>>) -> Object<'a> {
        let interpreter = Interpreter::new(self.types.clone(), self.namespaces.clone(), self.clone());
        unsafe {
            let refr = Arc::into_raw(self.clone()) as *mut VM<'a>;
            (*refr).interpreters.push(Arc::new(interpreter));
            let interp_refr = Arc::into_raw((*refr).interpreters.last().expect("No interpreters").clone()) as *mut Interpreter<'a>;
            return (*interp_refr).run_interpreter(bytecode);
        }
    }

    pub fn execute_vars(self: Arc<Self>, bytecode: Arc<Bytecode<'a>>, vars: Object<'a>) -> Object<'a> {
        let interpreter = Interpreter::new(self.types.clone(), self.namespaces.clone(), self.clone());
        unsafe {
            let refr = Arc::into_raw(self.clone()) as *mut VM<'a>;
            (*refr).interpreters.push(Arc::new(interpreter));
            let interp_refr = Arc::into_raw((*refr).interpreters.last().expect("No interpreters").clone()) as *mut Interpreter<'a>;
            return (*interp_refr).run_interpreter_vars(bytecode, vars);
        }
    }
}

impl<'a> Interpreter<'a> {
    pub fn new(types: Arc<HashMap<String, Object<'a>>>, namespaces: Arc<Namespaces<'a>>, vm: Arc<VM<'a>>) -> Interpreter<'a> {
        Interpreter { frames: Vec::new(), types, namespaces, vm }
    }

    fn add_frame(&mut self) {
        unsafe {
            let namespace_refr = Arc::into_raw(self.namespaces.clone()) as *mut Namespaces<'a>;
            let dict = dictobject::dict_from(self.vm.clone(), HashMap::new());
            (*namespace_refr).locals.push(dict.clone());
            if (*namespace_refr).globals == std::ptr::null_mut() {
                (*namespace_refr).globals = Arc::into_raw(dict) as *mut Object;
            }
        }
        self.frames.push(Frame { register1: noneobject::none_from(self.vm.clone()), register2: noneobject::none_from(self.vm.clone()), args: Vec::new() })
    }

    fn assign_to_register(&mut self, value: Object<'a>, register: CompilerRegister) {
        match register {
            CompilerRegister::R1 => {
                self.frames.last_mut().expect("No frames").register1 = value.clone();
            }
            CompilerRegister::R2 => {
                self.frames.last_mut().expect("No frames").register2 = value.clone();
            }
            CompilerRegister::NA => {
                unimplemented!("Cannot store to NA register");
            }
        }
    }

    fn read_register(&mut self, register: CompilerRegister) -> Object<'a> {
        match register {
            CompilerRegister::R1 => {
                return self.frames.last_mut().expect("No frames").register1.clone();
            }
            CompilerRegister::R2 => {
                return self.frames.last_mut().expect("No frames").register2.clone();
            }
            CompilerRegister::NA => {
                unimplemented!("Cannot read from NA register");
            }
        }
    }

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
    
    pub fn run_interpreter_vars(&mut self, bytecode: Arc<Bytecode<'a>>, vars: Object<'a>) -> Object<'a> {
        self.add_frame();
        unsafe {
            let namespace_refr = Arc::into_raw(self.namespaces.clone()) as *mut Namespaces<'a>;
            (*namespace_refr).locals.pop();
            (*namespace_refr).locals.push(vars);
        }
        self.run_interpreter(bytecode)
    }

    pub fn run_interpreter(&mut self, bytecode: Arc<Bytecode<'a>>) -> Object<'a> {
        self.add_frame();
        self.run_interpreter_raw(bytecode)
    }

    pub fn run_interpreter_raw(&mut self, bytecode: Arc<Bytecode<'a>>) -> Object<'a> {
        for instruction in bytecode.instructions.clone() {
            match instruction {
                CompilerInstruction::LoadConstR1(idx, _start, _end) => {
                    self.frames.last_mut().expect("No frames").register1 = bytecode.consts.get(idx).expect("Bytecode consts index out of range").clone();
                }
                CompilerInstruction::LoadConstR2(idx, _start, _end) => {
                    self.frames.last_mut().expect("No frames").register2 = bytecode.consts.get(idx).expect("Bytecode consts index out of range").clone();
                }
                CompilerInstruction::BinaryAdd(out, _start, _end) => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.clone().add.is_some());
                    let res = (last.register1.clone().add.expect("Method is not defined"))(last.register1.clone(), last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinarySub(out, _start, _end) => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.clone().sub.is_some());
                    let res = (last.register1.clone().sub.expect("Method is not defined"))(last.register1.clone(), last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinaryMul(out, _start, _end) => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.clone().mul.is_some());
                    let res = (last.register1.clone().mul.expect("Method is not defined"))(last.register1.clone(), last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinaryDiv(out, _start, _end) => {
                    let last = self.frames.last().expect("No frames");
                    debug_assert!(last.register1.clone().div.is_some());
                    let res = (last.register1.clone().div.expect("Method is not defined"))(last.register1.clone(), last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::StoreName(idx, register, _start, _end) => {
                    (self.namespaces.locals.last().expect("No locals").set.expect("Method is not defined"))(self.namespaces.locals.last().expect("No locals").clone(), bytecode.names.get(idx).expect("Bytecode names index out of range").clone(), self.read_register(register));
                    if !matches!(register, CompilerRegister::NA) {
                        self.assign_to_register(noneobject::none_from(self.vm.clone()), register);
                    }
                }
                CompilerInstruction::LoadName(idx, register, _start, _end) => {
                    let map = self.namespaces.locals.last().expect("No locals").clone();
                    let out = map.internals.get_map().expect("Expected map internal value").get(&bytecode.names.get(idx).expect("Bytecode names index out of range").clone());
                    debug_assert!(out.is_some());
                    if !matches!(register, CompilerRegister::NA) {
                        self.assign_to_register(out.unwrap().clone(), register);
                    }
                }
                CompilerInstruction::MakeFunction(nameidx, argsidx, codeidx, _start, _end) => {
                    let code = bytecode.consts.get(codeidx).expect("Bytecode consts index out of range").clone();
                    let args = bytecode.consts.get(argsidx).expect("Bytecode consts index out of range").clone();
                    let name = bytecode.names.get(nameidx).expect("Bytecode names index out of range").clone();
                    let func = fnobject::fn_from(self.vm.clone(), code, args.internals.get_arr().expect("Expected arr internal value").clone(), name.internals.get_str().expect("Expected str internal value").clone());
                    self.assign_to_register(func, CompilerRegister::R1);
                }
                CompilerInstruction::InitArgs(_start, _end) => {
                    self.frames.last_mut().expect("No frames").args.push(Arguments { args: Vec::new() });
                }
                CompilerInstruction::AddArgument(register, _start, _end) => {
                    let reg = self.read_register(register);
                    self.frames.last_mut().expect("No frames").args.last_mut().expect("No arguments prepared").args.push(reg);
                }
                CompilerInstruction::Call(callableregister, register, _start, _end) => {
                    let callable = self.read_register(callableregister);
                    let args = self.frames.last().expect("No frames").args.last().expect("No arguments for call");
                    debug_assert!(callable.call.is_some());
                    let value = (callable.call.expect("Method is not defined"))(callable, listobject::list_from(self.vm.clone(), args.args.clone()));
                    debug_assert!(value.is_some());
                    self.assign_to_register(value.unwrap(), register);
                }
                CompilerInstruction::Return(register, _start, _end) => {
                    let res = self.read_register(register);
                    self.frames.pop();
                    return res;
                }
                CompilerInstruction::StoreGlobal(idx, register, _start, _end) => {
                    let globals = unsafe { (*self.namespaces.globals).clone() };

                    globals.set.expect("Method is not defined")(self.namespaces.locals.last().expect("No locals").clone(), bytecode.names.get(idx).expect("Bytecode names index out of range").clone(), self.read_register(register));

                    if !matches!(register, CompilerRegister::NA) {
                        self.assign_to_register(noneobject::none_from(self.vm.clone()), register);
                    }
                }
                CompilerInstruction::LoadGlobal(idx, register, _start, _end) => {
                    let globals = unsafe { (*self.namespaces.globals).clone() };

                    let out = globals.internals.get_map().expect("Expected map internal value").get(&bytecode.names.get(idx).expect("Bytecode names index out of range").clone());
                    debug_assert!(out.is_some());
                    if !matches!(register, CompilerRegister::NA) {
                        self.assign_to_register(out.unwrap().clone(), register);
                    }
                }
            }
        }

        self.frames.pop();

        noneobject::none_from(self.vm.clone())
    }
}