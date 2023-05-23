use std::collections::HashMap;

// Interpret bytecode
use crate::{objects::{Object, noneobject, utils::object_repr}, compiler::{CompilerInstruction, Bytecode, CompilerRegister}};

pub struct VM {
    types: HashMap<String, Object>,
}

pub struct Interpreter<'a> {
    frames: Vec<Frame>,
    vm: &'a VM,
}

#[derive(Clone)]
struct Frame {
    register1: Object,
    register2: Object,
}

impl VM {
    pub fn new(types: HashMap<String, Object>) -> VM {
        VM { types }
    }

    pub fn execute(&mut self, bytecode: Bytecode) -> Object {
        let mut interpreter = Interpreter::new(self);
        
        return interpreter.run_interpreter(bytecode);
    }
}

impl<'a> Interpreter<'a> {
    pub fn new(vm: &'a VM) -> Interpreter<'a> {
        Interpreter { frames: Vec::new(), vm, }
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
            }
        }

        return noneobject::none_from();
    }
}