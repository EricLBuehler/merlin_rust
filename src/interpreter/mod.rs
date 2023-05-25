use std::{collections::HashMap, env::args};

// Interpret bytecode
use crate::{objects::{Object, noneobject, utils::object_repr, dictobject, fnobject}, compiler::{CompilerInstruction, Bytecode, CompilerRegister}};

pub struct Namespaces {
    locals: Object,
}

pub struct VM<'a> {
    types: HashMap<String, Object>,
    interpreters: Vec<Interpreter<'a>>,
    namespaces: Namespaces,
}

pub struct Interpreter<'a> {
    frames: Vec<Frame>,
    types: &'a HashMap<String, Object>,
    namespaces: &'a Namespaces,
}

#[derive(Clone)]
struct Frame {
    register1: Object,
    register2: Object,
}

impl<'a> VM<'a> {
    pub fn new(types: HashMap<String, Object>) -> VM<'a> {
        VM { types, interpreters: Vec::new(), namespaces: Namespaces { locals: dictobject::dict_from(HashMap::new()) } }
    }

    pub fn execute(&'a mut self, bytecode: Bytecode) -> Object {
        let interpreter = Interpreter::new(&self.types, &self.namespaces);
        self.interpreters.push(interpreter);
        return self.interpreters.last_mut().unwrap().run_interpreter(bytecode);
    }
}

impl<'a> Interpreter<'a> {
    pub fn new(types: &'a HashMap<String, Object>, namespaces: &'a Namespaces) -> Interpreter<'a> {
        Interpreter { frames: Vec::new(), types, namespaces }
    }

    fn add_frame(&mut self) {
        self.frames.push(Frame { register1: noneobject::none_from(), register2: noneobject::none_from() })
    }

    fn assign_to_register(&mut self, value: Object, register: CompilerRegister) {
        match register {
            CompilerRegister::R1 => {
                self.frames.last_mut().unwrap().register1 = value.clone();
            }
            CompilerRegister::R2 => {
                self.frames.last_mut().unwrap().register2 = value.clone();
            }
            CompilerRegister::NA => {
                unimplemented!("Cannot have NA register");
            }
        }
    }

    fn output_register(&mut self, register: CompilerRegister) {
        match register {
            CompilerRegister::R1 => {
                println!("{}", object_repr(&self.frames.last().unwrap().register1));
            }
            CompilerRegister::R2 => {
                println!("{}", object_repr(&self.frames.last().unwrap().register2));
            }
            CompilerRegister::NA => {
                unimplemented!("Cannot have NA register");
            }
        }
    }

    pub fn run_interpreter(&mut self, bytecode: Bytecode) -> Object {
        self.add_frame();

        for instruction in bytecode.instructions {
            match instruction {
                CompilerInstruction::LoadConstR1(idx, _start, _end) => {
                    self.frames.last_mut().unwrap().register1 = bytecode.consts.get(idx).unwrap().clone();
                }
                CompilerInstruction::LoadConstR2(idx, _start, _end) => {
                    self.frames.last_mut().unwrap().register2 = bytecode.consts.get(idx).unwrap().clone();
                }
                CompilerInstruction::BinaryAdd(out, _start, _end) => {
                    let last = self.frames.last().unwrap();
                    debug_assert!(last.register1.clone().add.is_some());
                    let res = (last.register1.clone().add.unwrap())(last.register1.clone(), last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinarySub(out, _start, _end) => {
                    let last = self.frames.last().unwrap();
                    debug_assert!(last.register1.clone().sub.is_some());
                    let res = (last.register1.clone().sub.unwrap())(last.register1.clone(), last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinaryMul(out, _start, _end) => {
                    let last = self.frames.last().unwrap();
                    debug_assert!(last.register1.clone().mul.is_some());
                    let res = (last.register1.clone().mul.unwrap())(last.register1.clone(), last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinaryDiv(out, _start, _end) => {
                    let last = self.frames.last().unwrap();
                    debug_assert!(last.register1.clone().div.is_some());
                    let res = (last.register1.clone().div.unwrap())(last.register1.clone(), last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::StoreName(idx, register, _start, _end) => {
                    (self.namespaces.locals.set.unwrap())(self.namespaces.locals.clone(), bytecode.names.get(idx).unwrap().clone(), self.frames.last().unwrap().register1.clone());
                    if !matches!(register, CompilerRegister::NA) {
                        self.assign_to_register(noneobject::none_from(), register);
                    }
                }
                CompilerInstruction::LoadName(idx, register, _start, _end) => {
                    let map = self.namespaces.locals.clone();
                    let out = map.internals.get_map().unwrap().get(&bytecode.names.get(idx).unwrap().clone());
                    debug_assert!(out.is_some());
                    if !matches!(register, CompilerRegister::NA) {
                        self.assign_to_register(out.unwrap().clone(), register);
                    }
                }
                CompilerInstruction::MakeFunction(nameidx, argsidx, codeidx, _start, _end) => {
                    let code = bytecode.consts.get(codeidx).unwrap().clone();
                    let args = bytecode.consts.get(argsidx).unwrap().clone();
                    let name = bytecode.names.get(nameidx).unwrap().clone();
                    let func = fnobject::fn_from(code, args.internals.get_arr().unwrap().clone(), name.internals.get_str().unwrap().clone());
                    self.assign_to_register(func, CompilerRegister::R1);
                }
            }
        }

        return noneobject::none_from();
    }
}