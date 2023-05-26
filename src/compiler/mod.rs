//Generate bytecode from AST

use std::{sync::Arc, marker::PhantomData};

use crate::{objects::{Object, intobject, stringobject, listobject, codeobject}, parser::{self, nodes::{NodeType, BinaryOpType}, Position}, errors::{raise_error, ErrorType}, fileinfo::FileInfo, interpreter::VM};

pub struct Compiler<'a> {
    instructions: Vec<CompilerInstruction>,
    consts: Vec<Object<'a>>,
    names: Vec<Object<'a>>,
    info: &'a FileInfo<'a>,
    vm: Arc<VM<'a>>,
}

//first Position is start, second is end
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerInstruction {
    LoadConstR1(usize, Position, Position), //load const from consts[index] into R1
    LoadConstR2(usize, Position, Position), //load const from consts[index] into R2
    BinaryAdd(CompilerRegister, Position, Position), //Sum R1 (right), and R2 (left). Result in specified register
    BinarySub(CompilerRegister, Position, Position), //Subtract R2 (left) from R1 (right). Result in specified register
    BinaryMul(CompilerRegister, Position, Position), //Multiply R1 (right), and R2 (left). Result in specified register
    BinaryDiv(CompilerRegister, Position, Position), //Divide R1 (right) by R2 (left). Result in specified register
    StoreName(usize, CompilerRegister, Position, Position), //store R1 to names[index], loads None to specified register
    LoadName(usize, CompilerRegister, Position, Position), //load names[index] from locals to specified register
    MakeFunction(usize, usize, usize, Position, Position), //build function with name as names[index1], args as consts[index2], code as consts[index3] to R1
    InitArgs(Position, Position), //Initialize argument collector
    AddArgument(CompilerRegister, Position, Position), //Add argument from specified register to latest argument collector
    Call(CompilerRegister, CompilerRegister, Position, Position), //Call callable in specified register 1, result in specified register 2
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerRegister {
    R1,
    R2,
    NA,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Bytecode<'a> {
    pub instructions: Vec<CompilerInstruction>,
    pub consts: Vec<Object<'a>>,
    pub names: Vec<Object<'a>>,
    _marker: PhantomData<&'a ()>,
}

type Node = parser::nodes::Node;

impl<'a> Compiler<'a> {
    pub fn new(info: &'a FileInfo<'a>, vm: Arc<VM<'a>>) -> Compiler<'a> {
        Compiler{instructions: Vec::new(), consts: Vec::new(), names: Vec::new(), info, vm}
    }

    pub fn generate_bytecode(&mut self, ast: &Vec<Node>) -> Arc<Bytecode<'a>> {
        for head_node in ast {
            self.compile_statement(head_node);
        }
        Arc::new( Bytecode {instructions: self.instructions.clone(), consts: self.consts.clone(), names: self.names.clone(), _marker: PhantomData} )
    }

    fn compile_statement(&mut self, expr: &Node) {
        match expr.tp {
            NodeType::Decimal => {
                self.compile_expr(&expr, CompilerRegister::NA);
            }
            NodeType::Binary => {
                self.compile_expr(&expr, CompilerRegister::R1);
            }
            NodeType::Identifier => {
                self.compile_expr(&expr, CompilerRegister::R1);
            }
            NodeType::StoreNode => {
                self.compile_expr(&expr, CompilerRegister::NA);
            }
            NodeType::Function => {
                self.names.push(stringobject::string_from(self.vm.clone(), expr.data.get_data().raw.get("name").unwrap().clone()));
                let nameidx = self.names.len() - 1;

                let mut args = Vec::new();
                for arg in expr.data.get_data().args.unwrap() {
                    args.push(stringobject::string_from(self.vm.clone(), arg));
                }
                self.consts.push(listobject::list_from(self.vm.clone(), args));
                let argsidx = self.consts.len() - 1;

                let mut compiler = Compiler::new(self.info, self.vm.clone());
                let bytecode = compiler.generate_bytecode(expr.data.get_data().nodearr.unwrap());
                self.consts.push(codeobject::code_from(self.vm.clone(), bytecode));
                let codeidx = self.consts.len() - 1;

                self.instructions.push(CompilerInstruction::MakeFunction(nameidx, argsidx, codeidx, expr.start, expr.end));
                self.instructions.push(CompilerInstruction::StoreName(self.names.len()-1, CompilerRegister::NA, expr.start, expr.end));
            }
            NodeType::Call => {
                self.compile_expr(&expr, CompilerRegister::R1);
            }
        }
    }

    fn compile_expr(&mut self, expr: &Node, register: CompilerRegister) {
        match expr.tp {
            NodeType::Decimal => {
                let int = intobject::int_from_str(self.vm.clone(), expr.data.get_data().raw.get("value").unwrap().to_string());
                
                debug_assert!(int.is_some());
                
                self.consts.push(int.unwrap());
                match register {
                    CompilerRegister::R1 => {
                        self.instructions.push(CompilerInstruction::LoadConstR1(self.consts.len()-1, expr.start, expr.end));
                    }
                    CompilerRegister::R2 => {
                        self.instructions.push(CompilerInstruction::LoadConstR2(self.consts.len()-1, expr.start, expr.end));
                    }
                    CompilerRegister::NA => {
                        
                    }
                }
            }
            NodeType::Binary => {
                self.compile_expr(expr.data.get_data().nodes.get("left").unwrap(), CompilerRegister::R1);
                self.compile_expr(expr.data.get_data().nodes.get("right").unwrap(), CompilerRegister::R2);

                match register {
                    CompilerRegister::NA => {}
                    _ => {
                        match expr.data.get_data().op.unwrap() {
                            BinaryOpType::Add => {
                                self.instructions.push(CompilerInstruction::BinaryAdd(register, expr.start, expr.end));
                            }
                            BinaryOpType::Sub => {
                                self.instructions.push(CompilerInstruction::BinarySub(register, expr.start, expr.end));
                            }
                            BinaryOpType::Mul => {
                                self.instructions.push(CompilerInstruction::BinaryMul(register, expr.start, expr.end));
                            }
                            BinaryOpType::Div => {
                                self.instructions.push(CompilerInstruction::BinaryDiv(register, expr.start, expr.end));
                            }
                        }
                    }
                }
            }
            NodeType::StoreNode => {
                self.compile_expr(expr.data.get_data().nodes.get("expr").unwrap(), CompilerRegister::R1);
                self.names.push(stringobject::string_from(self.vm.clone(), expr.data.get_data().raw.get("name").unwrap().clone()));
                self.instructions.push(CompilerInstruction::StoreName(self.names.len()-1, register, expr.start, expr.end));
            }
            NodeType::Identifier => {
                self.names.push(stringobject::string_from(self.vm.clone(), expr.data.get_data().raw.get("name").unwrap().clone()));
                self.instructions.push(CompilerInstruction::LoadName(self.names.len()-1, register, expr.start, expr.end));
            }
            NodeType::Function => {
                raise_error("Function definition is not an expression", ErrorType::FunctionNotExpression, &expr.start, self.info);
            }
            NodeType::Call => {
                self.instructions.push(CompilerInstruction::InitArgs(expr.start, expr.end));
                let mut args = Vec::new();
                for arg in expr.data.get_data().nodearr.unwrap() {
                    args.push(self.compile_expr(arg, register));
                    self.instructions.push(CompilerInstruction::AddArgument(register, expr.start, expr.end));
                }
                self.names.push(stringobject::string_from(self.vm.clone(), expr.data.get_data().raw.get("name").unwrap().clone()));
                let nameidx = self.names.len() - 1;
                self.instructions.push(CompilerInstruction::LoadName(nameidx, register, expr.start, expr.end));
                self.instructions.push(CompilerInstruction::Call(register, register, expr.start, expr.end));
            }
        }
    }
}