//Generate bytecode from AST

use crate::{objects::{Object, intobject}, parser::{self, nodes::{NodeType, BinaryOpType}, Position}};

pub struct Compiler {
    instructions: Vec<CompilerInstruction>,
    consts: Vec<Object>,
}

//first Position is start, second is end
#[derive(Clone, Copy, Debug)]
pub enum CompilerInstruction {
    LoadConstR1(usize, Position, Position), //load const from consts[index] into R1
    LoadConstR2(usize, Position, Position), //load const from consts[index] into R2
    BinaryAdd(CompilerRegister, Position, Position), //Sum R1 (right), and R2 (left). Result in specified register
    BinarySub(CompilerRegister, Position, Position), //Subtract R2 (left) from R1 (right). Result in specified register
    BinaryMul(CompilerRegister, Position, Position), //Multiply R1 (right), and R2 (left). Result in specified register
    BinaryDiv(CompilerRegister, Position, Position), //Divide R1 (right) by R2 (left). Result in specified register
}

#[derive(Clone, Copy, Debug)]
pub enum CompilerRegister {
    R1,
    R2,
    NA,
}

pub struct Bytecode {
    pub instructions: Vec<CompilerInstruction>,
    pub consts: Vec<Object>,
}

type Node = parser::nodes::Node;

impl Compiler {
    pub fn new() -> Compiler {
        Compiler{instructions: Vec::new(), consts: Vec::new()}
    }

    pub fn generate_bytecode(&mut self, ast: Vec<Node>) -> Bytecode {
        for head_node in ast {
            self.compile_statement(head_node);
        }
        Bytecode {instructions: self.instructions.clone(), consts: self.consts.clone()}
    }

    fn compile_statement(&mut self, expr: Node) {
        match expr.tp {
            NodeType::Decimal => {
                self.compile_expr(&expr, CompilerRegister::NA);
            }
            NodeType::Binary => {
                self.compile_expr(&expr, CompilerRegister::R1);
            }
        }
    }

    fn compile_expr(&mut self, expr: &Node, register: CompilerRegister) {
        match expr.tp {
            NodeType::Decimal => {
                let int = intobject::int_from_str(expr.data.get_data().raw.get("value").unwrap().to_string());
                
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
        }
    }
}