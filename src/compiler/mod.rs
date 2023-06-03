//Generate bytecode from AST

use std::{marker::PhantomData};
use crate::Arc;
use crate::objects::utils::object_repr_safe;
use colored::Colorize;
use crate::{objects::{Object, intobject, stringobject, listobject, codeobject}, parser::{self, nodes::{NodeType, OpType}, Position}, errors::{raise_error, ErrorType}, fileinfo::FileInfo, interpreter::VM};

pub struct Compiler<'a> {
    instructions: Vec<CompilerInstruction>,
    consts: Vec<Object<'a>>,
    names: Vec<Object<'a>>,
    info: &'a FileInfo<'a>,
    vm: Arc<VM<'a>>,
    scope: CompilerScope,
    positions: Vec<(Position, Position)>,
}

pub enum CompilerScope {
    Local,
    Global,
}

//first Position is start, second is end
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerInstruction {
    LoadConstR1{index: usize}, //load const from consts[index] into R1
    LoadConstR2{index: usize}, //load const from consts[index] into R2
    BinaryAdd{register: CompilerRegister}, //Sum R1 (right), and R2 (left). Result in specified register
    BinarySub{register: CompilerRegister}, //Subtract R2 (left) from R1 (right). Result in specified register
    BinaryMul{register: CompilerRegister}, //Multiply R1 (right), and R2 (left). Result in specified register
    BinaryDiv{register: CompilerRegister}, //Divide R1 (right) by R2 (left). Result in specified register
    StoreName{idx: usize, register: CompilerRegister}, //store R1 to names[index], loads None to specified register
    LoadName{idx: usize, register: CompilerRegister}, //load names[index] from locals to specified register
    MakeFunction{nameidx: usize, argsidx: usize, codeidx: usize}, //build function with name as names[index1], args as consts[index2], code as consts[index3] to R1
    InitArgs{start: Position, end: Position}, //Initialize argument collector
    AddArgument{register: CompilerRegister}, //Add argument from specified register to latest argument collector
    Call{callableregister: CompilerRegister, register: CompilerRegister}, //Call callable in specified register 1, result in specified register 2
    Return{register: CompilerRegister}, //Return from specified register
    StoreGlobal{idx: usize, register: CompilerRegister}, //store R1 to names[index], loads None to specified register
    LoadGlobal{idx: usize, register: CompilerRegister}, //load names[index] from locals to specified register
    UnaryNeg{register: CompilerRegister}, //Unary negation of R1. Result in specified register
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
    pub positions: Vec<(Position, Position)>,
    _marker: PhantomData<&'a ()>,
}

type Node = parser::nodes::Node;

impl<'a> Compiler<'a> {
    pub fn new(info: &'a FileInfo<'a>, vm: Arc<VM<'a>>, scope: CompilerScope) -> Compiler<'a> {
        Compiler{instructions: Vec::new(), consts: Vec::new(), names: Vec::new(), info, vm, scope, positions: Vec::new()}
    }

    pub fn generate_bytecode(&mut self, ast: &Vec<Node>) -> Arc<Bytecode<'a>> {
        for head_node in ast {
            self.compile_statement(head_node);
        }
        Arc::new( Bytecode {instructions: self.instructions.clone(), consts: self.consts.clone(), names: self.names.clone(), positions: self.positions.clone(), _marker: PhantomData} )
    }

    fn compile_statement(&mut self, expr: &Node) {
        match expr.tp {
            NodeType::Decimal => {
                self.compile_expr(expr, CompilerRegister::NA);
            }
            NodeType::Binary => {
                self.compile_expr(expr, CompilerRegister::R1);
            }
            NodeType::Identifier => {
                self.compile_expr(expr, CompilerRegister::R1);
            }
            NodeType::StoreNode => {
                self.compile_expr(expr, CompilerRegister::R1);
            }
            NodeType::Function => {
                self.names.push(stringobject::string_from(self.vm.clone(), expr.data.get_data().raw.get("name").expect("Node.raw.name not found").clone()));
                let nameidx = self.names.len() - 1;

                let mut args = Vec::new();
                for arg in expr.data.get_data().args.expect("Node.args is not present") {
                    args.push(stringobject::string_from(self.vm.clone(), arg));
                }
                self.consts.push(listobject::list_from(self.vm.clone(), args));
                let argsidx = self.consts.len() - 1;

                let mut compiler = Compiler::new(self.info, self.vm.clone(), CompilerScope::Local);
                let bytecode = compiler.generate_bytecode(expr.data.get_data().nodearr.expect("Node.nodearr is not present"));
                self.consts.push(codeobject::code_from(self.vm.clone(), bytecode));
                let codeidx = self.consts.len() - 1;

                self.instructions.push(CompilerInstruction::MakeFunction{nameidx, argsidx, codeidx});
                self.positions.push((expr.start, expr.end));
                self.instructions.push(CompilerInstruction::StoreName{idx: self.names.len()-1, register: CompilerRegister::R1});
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Call => {
                self.compile_expr(expr, CompilerRegister::R1);
            }
            NodeType::Return => {
                self.compile_expr(expr, CompilerRegister::R1);
            }
            NodeType::Unary => {
                self.compile_expr(expr, CompilerRegister::R1);
            }
        }
    }
    
    fn raise_exc_pos(&mut self, exc_obj: Object<'a>, start: Position, end: Position) -> ! {
        let header: String = match object_repr_safe(&exc_obj) { crate::objects::MethodValue::Some(v) => {v}, _ => { unimplemented!() }};
        let location: String = format!("{}:{}:{}", self.info.name, start.line+1, start.startcol+1);
        println!("{}", header.red().bold());
        println!("{}", location.red());
        let lines = Vec::from_iter(self.info.data.split(|num| *num as char == '\n'));

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

    fn compile_expr(&mut self, expr: &Node, register: CompilerRegister) {
        match expr.tp {
            NodeType::Decimal => {
                let int = intobject::int_from_str(self.vm.clone(), expr.data.get_data().raw.get("value").expect("Node.raw.value not found").to_string());
                
                maybe_handle_exception!(self, int, expr.start, expr.end);
                
                self.consts.push(int.unwrap());
                match register {
                    CompilerRegister::R1 => {
                        self.instructions.push(CompilerInstruction::LoadConstR1{index: self.consts.len()-1});
                        self.positions.push((expr.start, expr.end));
                    }
                    CompilerRegister::R2 => {
                        self.instructions.push(CompilerInstruction::LoadConstR2{index: self.consts.len()-1});
                        self.positions.push((expr.start, expr.end));
                    }
                    CompilerRegister::NA => {
                        
                    }
                }
            }
            NodeType::Binary => {
                self.compile_expr(expr.data.get_data().nodes.get("left").expect("Node.nodes.left not found"), CompilerRegister::R1);
                self.compile_expr(expr.data.get_data().nodes.get("right").expect("Node.nodes.right not found"), CompilerRegister::R2);

                match register {
                    CompilerRegister::NA => {}
                    _ => {
                        match expr.data.get_data().op.expect("Node.op is not present") {
                            OpType::Add => {
                                self.instructions.push(CompilerInstruction::BinaryAdd{register});
                                self.positions.push((expr.start, expr.end));
                            }
                            OpType::Sub => {
                                self.instructions.push(CompilerInstruction::BinarySub{register});
                                self.positions.push((expr.start, expr.end));
                            }
                            OpType::Mul => {
                                self.instructions.push(CompilerInstruction::BinaryMul{register});
                                self.positions.push((expr.start, expr.end));
                            }
                            OpType::Div => {
                                self.instructions.push(CompilerInstruction::BinaryDiv{register});
                                self.positions.push((expr.start, expr.end));
                            }
                            _ => {
                                unimplemented!();
                            }
                        }
                    }
                }
            }
            NodeType::StoreNode => {
                self.compile_expr(expr.data.get_data().nodes.get("expr").expect("Node.nodes.expr not found"), register);
                self.names.push(stringobject::string_from(self.vm.clone(), expr.data.get_data().raw.get("name").expect("Node.raw.name not found").clone()));
                match self.scope {
                    CompilerScope::Local => {
                        self.instructions.push(CompilerInstruction::StoreName{idx: self.names.len()-1, register});
                        self.positions.push((expr.start, expr.end));
                    }
                    CompilerScope::Global => {
                        self.instructions.push(CompilerInstruction::StoreGlobal{idx: self.names.len()-1, register});
                        self.positions.push((expr.start, expr.end));
                    }
                }
            }
            NodeType::Identifier => {
                self.names.push(stringobject::string_from(self.vm.clone(), expr.data.get_data().raw.get("name").expect("Node.raw.name not found").clone()));
                match self.scope {
                    CompilerScope::Local => {
                        self.instructions.push(CompilerInstruction::LoadName{idx: self.names.len()-1, register});
                        self.positions.push((expr.start, expr.end));
                    }
                    CompilerScope::Global => {
                        self.instructions.push(CompilerInstruction::LoadGlobal{idx: self.names.len()-1, register});
                        self.positions.push((expr.start, expr.end));
                    }
                }
            }
            NodeType::Function => {
                raise_error("Function definition is not an expression", ErrorType::FunctionNotExpression, &expr.start, self.info);
            }
            NodeType::Call => {
                self.instructions.push(CompilerInstruction::InitArgs{start: expr.start, end: expr.end});
                self.positions.push((expr.start, expr.end));
                for arg in expr.data.get_data().nodearr.expect("Node.nodearr is not present") {
                    self.compile_expr(arg, register);
                    self.instructions.push(CompilerInstruction::AddArgument{register});
                    self.positions.push((expr.start, expr.end));
                }
                self.names.push(stringobject::string_from(self.vm.clone(), expr.data.get_data().raw.get("name").expect("Node.raw.name not found").clone()));
                let nameidx = self.names.len() - 1;
                self.instructions.push(CompilerInstruction::LoadName{idx: nameidx, register});
                self.positions.push((expr.start, expr.end));
                self.instructions.push(CompilerInstruction::Call{callableregister: register, register});
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Return => {
                self.compile_expr(expr.data.get_data().nodes.get("expr").expect("Node.nodes.expr not found"), register);
                self.instructions.push(CompilerInstruction::Return{register});
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Unary => {
                self.compile_expr(expr.data.get_data().nodes.get("expr").expect("Node.nodes.expr not found"), CompilerRegister::R1);

                match register {
                    CompilerRegister::NA => {}
                    _ => {
                        match expr.data.get_data().op.expect("Node.op is not present") {
                            OpType::Neg => {
                                self.instructions.push(CompilerInstruction::UnaryNeg{register});
                                self.positions.push((expr.start, expr.end));
                            }
                            _ => {
                                unimplemented!();
                            }
                        }
                    }
                }
            }
        }
    }
}