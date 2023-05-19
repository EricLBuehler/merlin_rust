//Generate bytecode from AST

use crate::{objects::{Object, intobject::IntObject}, parser::{self, nodes::{NodeType, BinaryOpType}, Position}};

pub struct Compiler {
    instructions: Vec<CompilerInstruction>,
    consts: Vec<Object>,
}

//first Position is start, second is end
#[derive(Clone, Copy, Debug)]
pub enum CompilerInstruction {
    LoadConstR1(usize, Position, Position), //load const from consts[index] into R1
    LoadConstR2(usize, Position, Position), //load const from consts[index] into R2
    BinaryAdd(Position, Position), //Sum R1 (right), and R2 (left)
    BinarySub(Position, Position), //SubtR1ct R2 (left), and R1 (right)
    BinaryMul(Position, Position), //Multiply R1 (right), and R2 (left)
    BinaryDiv(Position, Position), //Divide R1 (right) by R2 (left)
}

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
        return Compiler{instructions: Vec::new(), consts: Vec::new()};
    }

    pub fn generate_bytecode(&mut self, ast: Vec<Node>) -> Bytecode {
        for head_node in ast {
            self.compile_statement(head_node);
        }
        return Bytecode {instructions: self.instructions.clone(), consts: self.consts.clone()};
    }

    fn compile_statement(&mut self, expr: Node) {
        match expr.tp {
            NodeType::DECIMAL => {
                self.compile_expr(&expr, CompilerRegister::R1);
            }
            NodeType::BINARY => {
                self.compile_expr(&expr, CompilerRegister::NA);
            }
        }
    }

    fn compile_expr(&mut self, expr: &Node, register: CompilerRegister) {
        match expr.tp {
            NodeType::DECIMAL => {
                let int = IntObject::from_str(expr.data.get_data().raw.get("value").unwrap().to_string());
                
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
                        unreachable!();
                    }
                }
            }
            NodeType::BINARY => {
                self.compile_expr(expr.data.get_data().nodes.get("left").unwrap(), CompilerRegister::R1);
                self.compile_expr(expr.data.get_data().nodes.get("right").unwrap(), CompilerRegister::R2);

                match expr.data.get_data().op.unwrap() {
                    BinaryOpType::ADD => {
                        self.instructions.push(CompilerInstruction::BinaryAdd(expr.start, expr.end));
                    }
                    BinaryOpType::SUB => {
                        self.instructions.push(CompilerInstruction::BinarySub(expr.start, expr.end));
                    }
                    BinaryOpType::MUL => {
                        self.instructions.push(CompilerInstruction::BinaryMul(expr.start, expr.end));
                    }
                    BinaryOpType::DIV => {
                        self.instructions.push(CompilerInstruction::BinaryDiv(expr.start, expr.end));
                    }
                }
            }
        }
    }
}