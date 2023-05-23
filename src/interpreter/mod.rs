// Interpret bytecode
use crate::{objects::{Object, noneobject, utils::object_repr}, compiler::{CompilerInstruction, Bytecode, CompilerRegister}};

pub struct Interpreter {
    frames: Vec<Frame>,
}

#[derive(Clone)]
struct Frame {
    register1: Object,
    register2: Object,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { frames: Vec::new() }
    }

    fn add_frame(&mut self) {
        self.frames.push(Frame { register1: noneobject::NoneObject::from(), register2: noneobject::NoneObject::from() })
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
                    let res = last.register1.clone().add(last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinarySub(out, _start, _end) => {
                    let last = self.frames.last().unwrap();
                    let res = last.register1.clone().sub(last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinaryMul(out, _start, _end) => {
                    let last = self.frames.last().unwrap();
                    let res = last.register1.clone().mul(last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
                CompilerInstruction::BinaryDiv(out, _start, _end) => {
                    let last = self.frames.last().unwrap();
                    let res = last.register1.clone().div(last.register2.clone());
                    debug_assert!(res.is_some());
                    self.assign_to_register(res.unwrap(), out);

                    if cfg!(debug_assertions) { self.output_register(out) };
                }
            }
        }

        return noneobject::NoneObject::from();
    }
}