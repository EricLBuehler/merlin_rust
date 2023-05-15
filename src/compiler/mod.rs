use crate::{objects::{Object, intobject::IntObject}, parser::{self, nodes::NodeType, Position}};

pub struct Compiler {
    instructions: Vec<CompilerInstruction>,
    consts: Vec<Object>,
}

//first Position is start, second is end
#[derive(Clone, Copy, Debug)]
pub enum CompilerInstruction {
    LoadConst(usize, Position, Position), //index
    BinaryAdd(Position, Position), //TOS is right, TOS-1 is left
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
                self.compile_expr(&expr);
            }
            NodeType::BINARY => {
                self.compile_expr(&expr);
            }
        }
    }

    fn compile_expr(&mut self, expr: &Node) {
        match expr.tp {
            NodeType::DECIMAL => {
                let int = IntObject::from_str(expr.data.get_data().raw.get("value").unwrap().to_string());
                debug_assert!(int.is_some());
                self.consts.push(int.unwrap());
                self.instructions.push(CompilerInstruction::LoadConst(self.consts.len()-1, expr.start, expr.end));
            }
            NodeType::BINARY => {
                self.compile_expr(expr.data.get_data().nodes.get("left").unwrap());
                self.compile_expr(expr.data.get_data().nodes.get("right").unwrap());
                self.instructions.push(CompilerInstruction::BinaryAdd(expr.start, expr.end));
            }
        }
    }
}