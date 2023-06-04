//Generate bytecode from AST

use crate::objects::utils::object_repr_safe;
use crate::Arc;
use crate::{
    errors::{raise_error, ErrorType},
    fileinfo::FileInfo,
    interpreter::VM,
    objects::{codeobject, intobject, listobject, stringobject, Object},
    parser::{
        self,
        nodes::{NodeType, OpType},
        Position,
    },
};
use colored::Colorize;
use hashbrown::HashMap;
use std::marker::PhantomData;

pub struct Compiler<'a> {
    instructions: Vec<CompilerInstruction>,
    consts: Vec<Object<'a>>,
    names: HashMap<String, i32>,
    info: &'a FileInfo<'a>,
    vm: Arc<VM<'a>>,
    scope: CompilerScope,
    positions: Vec<(Position, Position)>,
    register_index: i32,
    register_max: i32,
}

pub enum CompilerScope {
    Local,
    Global,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompilerInstruction {
    LoadConst {
        index: usize,
        register: CompilerRegister,
    }, //consts[index]
    BinaryAdd {
        a: CompilerRegister,
        b: CompilerRegister,
        result: CompilerRegister,
    },
    BinarySub {
        a: CompilerRegister,
        b: CompilerRegister,
        result: CompilerRegister,
    },
    BinaryMul {
        a: CompilerRegister,
        b: CompilerRegister,
        result: CompilerRegister,
    },
    BinaryDiv {
        a: CompilerRegister,
        b: CompilerRegister,
        result: CompilerRegister,
    },
    CopyRegister {
        from: CompilerRegister,
        to: CompilerRegister,
    },
    MakeFunction {
        nameidx: usize,
        argsidx: usize,
        codeidx: usize,
        out: CompilerRegister,
    }, //All are consts
    Call {
        callableregister: CompilerRegister,
        result: CompilerRegister,
        arg_registers: Vec<CompilerRegister>,
    },
    Return {
        register: CompilerRegister,
    },
    UnaryNeg {
        a: CompilerRegister,
        result: CompilerRegister,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerRegister {
    R(i32),
    V(i32),
}

impl From<CompilerRegister> for i32 {
    fn from(value: CompilerRegister) -> Self {
        match value {
            CompilerRegister::V(v) => v,
            CompilerRegister::R(v) => v,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Bytecode<'a> {
    pub instructions: Vec<CompilerInstruction>,
    pub consts: Vec<Object<'a>>,
    pub names: HashMap<String, i32>,
    pub positions: Vec<(Position, Position)>,
    pub n_registers: i32,
    pub n_variables: i32,
    _marker: PhantomData<&'a ()>,
}

type Node = parser::nodes::Node;

macro_rules! increment_reg_num {
    ($this:ident) => {
        $this.register_index += 1;
        if $this.register_index > $this.register_max {
            $this.register_max = $this.register_index;
        };
    };
}

#[derive(Clone)]
struct RegisterContext {
    value: CompilerRegister,
    left: Option<CompilerRegister>,
    leftctx: Option<Box<RegisterContext>>,
    right: Option<CompilerRegister>,
    rightctx: Option<Box<RegisterContext>>,
    args: Option<Vec<CompilerRegister>>,
    registers: i32, //How many registers this instruction needs
}

impl<'a> Compiler<'a> {
    pub fn new(info: &'a FileInfo<'a>, vm: Arc<VM<'a>>, scope: CompilerScope) -> Compiler<'a> {
        Compiler {
            instructions: Vec::new(),
            consts: Vec::new(),
            names: HashMap::new(),
            info,
            vm,
            scope,
            positions: Vec::new(),
            register_index: 0,
            register_max: 0,
        }
    }

    pub fn generate_bytecode(&mut self, ast: &Vec<Node>) -> Arc<Bytecode<'a>> {
        for head_node in ast {
            self.compile_statement(head_node);
        }
        Arc::new(Bytecode {
            instructions: self.instructions.clone(),
            consts: self.consts.clone(),
            names: self.names.clone(),
            positions: self.positions.clone(),
            n_registers: self.register_max,
            n_variables: self.names.len() as i32,
            _marker: PhantomData,
        })
    }

    fn compile_statement(&mut self, expr: &Node) {
        match expr.tp {
            NodeType::Decimal => {
                let ctx = self.compile_expr_values(expr);
                self.compile_expr_operation(expr, ctx);
            }
            NodeType::Binary => {
                let ctx = self.compile_expr_values(expr);
                self.compile_expr_operation(expr, ctx);
            }
            NodeType::Identifier => {
                let ctx = self.compile_expr_values(expr);
                self.compile_expr_operation(expr, ctx);
            }
            NodeType::StoreNode => {
                let ctx = self.compile_expr_values(expr);
                self.compile_expr_operation(expr, ctx);
            }
            NodeType::Function => {
                let mut registers = 0;
                let name = expr
                    .data
                    .get_data()
                    .raw
                    .get("name")
                    .expect("Node.raw.name not found")
                    .clone();
                self.consts
                    .push(stringobject::string_from(self.vm.clone(), name.clone()));
                let nameidx = self.consts.len() - 1;

                let mut args = Vec::new();
                for arg in expr.data.get_data().args.expect("Node.args is not present") {
                    args.push(stringobject::string_from(self.vm.clone(), arg));
                }
                self.consts
                    .push(listobject::list_from(self.vm.clone(), args));
                let argsidx = self.consts.len() - 1;

                let mut compiler = Compiler::new(self.info, self.vm.clone(), CompilerScope::Local);
                let bytecode = compiler.generate_bytecode(
                    expr.data
                        .get_data()
                        .nodearr
                        .expect("Node.nodearr is not present"),
                );
                self.consts
                    .push(codeobject::code_from(self.vm.clone(), bytecode));
                let codeidx = self.consts.len() - 1;

                self.instructions.push(CompilerInstruction::MakeFunction {
                    nameidx,
                    argsidx,
                    codeidx,
                    out: CompilerRegister::R(self.register_index),
                });
                increment_reg_num!(self);
                registers += 1;

                self.positions.push((expr.start, expr.end));

                self.names.insert(name, self.names.len() as i32);
                self.instructions.push(CompilerInstruction::CopyRegister {
                    from: CompilerRegister::R(self.register_index - 1),
                    to: CompilerRegister::V((self.names.len() - 1) as i32),
                });
                self.positions.push((expr.start, expr.end));
                self.register_index -= registers;
            }
            NodeType::Call => {
                let ctx = self.compile_expr_values(expr);
                self.compile_expr_operation(expr, ctx);
            }
            NodeType::Return => {
                let ctx = self.compile_expr_values(expr);
                self.compile_expr_operation(expr, ctx);
            }
            NodeType::Unary => {
                let ctx = self.compile_expr_values(expr);
                self.compile_expr_operation(expr, ctx);
            }
        }
    }

    fn raise_exc_pos(&mut self, exc_obj: Object<'a>, start: Position, end: Position) -> ! {
        let header: String = match object_repr_safe(&exc_obj) {
            crate::objects::MethodValue::Some(v) => v,
            _ => {
                unimplemented!()
            }
        };
        let location: String = format!(
            "{}:{}:{}",
            self.info.name,
            start.line + 1,
            start.startcol + 1
        );
        println!("{}", header.red().bold());
        println!("{}", location.red());
        let lines = Vec::from_iter(self.info.data.split(|num| *num as char == '\n'));

        let snippet: String = format!(
            "{}",
            String::from_utf8(
                lines
                    .get(start.line)
                    .expect("Line index out of range")
                    .to_vec()
            )
            .expect("utf8 conversion failed")
            .blue()
        );
        let mut arrows: String = String::new();
        for idx in 0..snippet.len() {
            if idx >= start.startcol && idx < end.endcol {
                arrows += "^";
            } else {
                arrows += " ";
            }
        }
        let linestr = (start.line + 1).to_string().blue().bold();
        println!("{} | {}", linestr, snippet);
        println!("{} | {}", " ".repeat(linestr.len()), arrows.green());

        //Should this happen??
        VM::terminate(self.vm.clone());
    }

    fn compile_expr_values(&mut self, expr: &Node) -> RegisterContext {
        match expr.tp {
            NodeType::Decimal => {
                let res = RegisterContext {
                    value: CompilerRegister::R(self.register_index),
                    left: None,
                    leftctx: None,
                    right: None,
                    rightctx: None,
                    args: None,
                    registers: 1,
                };
                increment_reg_num!(self);
                res
            }
            NodeType::Binary => {
                let old = self.register_index;

                let left = self.compile_expr_values(
                    expr.data
                        .get_data()
                        .nodes
                        .get("left")
                        .expect("Node.nodes.left not found"),
                );
                let right = self.compile_expr_values(
                    expr.data
                        .get_data()
                        .nodes
                        .get("right")
                        .expect("Node.nodes.right not found"),
                );

                let res = RegisterContext {
                    value: CompilerRegister::R(old),
                    left: Some(left.value),
                    leftctx: Some(Box::new(left)),
                    right: Some(right.value),
                    rightctx: Some(Box::new(right)),
                    args: None,
                    registers: 1,
                };
                increment_reg_num!(self);
                res
            }
            NodeType::StoreNode => {
                let old = self.register_index;
                let expr = self.compile_expr_values(
                    expr.data
                        .get_data()
                        .nodes
                        .get("expr")
                        .expect("Node.nodes.expr not found"),
                );

                
                RegisterContext {
                    value: CompilerRegister::R(old),
                    left: Some(expr.value),
                    leftctx: Some(Box::new(expr)),
                    right: None,
                    rightctx: None,
                    args: None,
                    registers: 0,
                }
            }
            NodeType::Identifier => {
                let var = self.names.get_key_value(
                    expr.data
                        .get_data()
                        .raw
                        .get("name")
                        .expect("Node.raw.name not found"),
                );
                
                RegisterContext {
                    value: CompilerRegister::V(match var {
                        Some(v) => *v.1,
                        None => -1,
                    }),
                    left: None,
                    leftctx: None,
                    right: None,
                    rightctx: None,
                    args: None,
                    registers: 0,
                }
            }
            NodeType::Call => {
                let name = *expr
                    .data
                    .get_data()
                    .nodes
                    .get("name")
                    .expect("Node.nodes.name not found");
                let callable = self.compile_expr_values(name);

                let mut args = Vec::new();
                let mut args_registers = 0;
                for arg in expr
                    .data
                    .get_data()
                    .nodearr
                    .expect("Node.nodearr is not present")
                {
                    let arg = self.compile_expr_values(arg);
                    args.push(arg.value);
                    args_registers += arg.registers;
                }

                let res = RegisterContext {
                    value: CompilerRegister::R(self.register_index),
                    left: Some(callable.value),
                    leftctx: Some(Box::new(callable)),
                    right: None,
                    rightctx: None,
                    args: Some(args),
                    registers: 1 + args_registers,
                };
                increment_reg_num!(self);
                res
            }
            NodeType::Return => {
                let var = self.compile_expr_values(
                    expr.data
                        .get_data()
                        .nodes
                        .get("expr")
                        .expect("Node.nodes.expr not found"),
                );

                
                RegisterContext {
                    value: var.value,
                    left: None,
                    leftctx: Some(Box::new(var)),
                    right: None,
                    rightctx: None,
                    args: None,
                    registers: 0,
                }
            }
            NodeType::Unary => {
                let old = self.register_index;
                let var = self.compile_expr_values(
                    expr.data
                        .get_data()
                        .nodes
                        .get("expr")
                        .expect("Node.nodes.expr not found"),
                );

                let res = RegisterContext {
                    value: CompilerRegister::R(old),
                    left: Some(var.value),
                    leftctx: Some(Box::new(var)),
                    right: None,
                    rightctx: None,
                    args: None,
                    registers: 0,
                };
                increment_reg_num!(self);
                res
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn compile_expr_operation(&mut self, expr: &Node, ctx: RegisterContext) {
        match expr.tp {
            NodeType::Decimal => {
                let int = intobject::int_from_str(
                    self.vm.clone(),
                    expr.data
                        .get_data()
                        .raw
                        .get("value")
                        .expect("Node.raw.value not found")
                        .to_string(),
                );

                maybe_handle_exception!(self, int, expr.start, expr.end);

                self.consts.push(int.unwrap());

                self.instructions.push(CompilerInstruction::LoadConst {
                    index: self.consts.len() - 1,
                    register: ctx.value,
                });
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Binary => {
                self.compile_expr_operation(
                    expr.data
                        .get_data()
                        .nodes
                        .get("left")
                        .expect("Node.nodes.left not found"),
                    *ctx.leftctx.unwrap(),
                );
                self.compile_expr_operation(
                    expr.data
                        .get_data()
                        .nodes
                        .get("right")
                        .expect("Node.nodes.right not found"),
                    *ctx.rightctx.unwrap(),
                );

                match expr.data.get_data().op.expect("Node.op is not present") {
                    OpType::Add => {
                        self.instructions.push(CompilerInstruction::BinaryAdd {
                            a: ctx.left.unwrap(),
                            b: ctx.right.unwrap(),
                            result: ctx.value,
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    OpType::Sub => {
                        self.instructions.push(CompilerInstruction::BinarySub {
                            a: ctx.left.unwrap(),
                            b: ctx.right.unwrap(),
                            result: ctx.value,
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    OpType::Mul => {
                        self.instructions.push(CompilerInstruction::BinaryMul {
                            a: ctx.left.unwrap(),
                            b: ctx.right.unwrap(),
                            result: ctx.value,
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    OpType::Div => {
                        self.instructions.push(CompilerInstruction::BinaryDiv {
                            a: ctx.left.unwrap(),
                            b: ctx.right.unwrap(),
                            result: ctx.value,
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    _ => {
                        unimplemented!();
                    }
                }
            }
            NodeType::StoreNode => {
                self.compile_expr_operation(
                    expr.data
                        .get_data()
                        .nodes
                        .get("expr")
                        .expect("Node.nodes.expr not found"),
                    *ctx.leftctx.unwrap(),
                );

                self.names.insert(
                    expr.data
                        .get_data()
                        .raw
                        .get("name")
                        .expect("Node.raw.name not found")
                        .clone(),
                    self.names.len() as i32,
                );

                self.instructions.push(CompilerInstruction::CopyRegister {
                    from: ctx.left.unwrap(),
                    to: CompilerRegister::V((self.names.len() - 1) as i32),
                });
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Identifier => {}
            NodeType::Function => {
                raise_error(
                    "Function definition is not an expression",
                    ErrorType::FunctionNotExpression,
                    &expr.start,
                    self.info,
                );
            }
            NodeType::Call => {
                let name = *expr.data.get_data().nodes.get("name").expect("Node");
                self.compile_expr_operation(name, *ctx.leftctx.unwrap());

                for arg in expr
                    .data
                    .get_data()
                    .nodearr
                    .expect("Node.nodearr is not present")
                {
                    let argctx = self.compile_expr_values(arg);
                    self.compile_expr_operation(expr, argctx);
                }
                self.instructions.push(CompilerInstruction::Call {
                    callableregister: ctx.left.unwrap(),
                    result: CompilerRegister::R(self.register_index - 1),
                    arg_registers: ctx.args.unwrap(),
                });
                increment_reg_num!(self);
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Return => {
                self.compile_expr_operation(
                    expr.data
                        .get_data()
                        .nodes
                        .get("expr")
                        .expect("Node.nodes.expr not found"),
                    *ctx.leftctx.unwrap(),
                );
                self.instructions.push(CompilerInstruction::Return {
                    register: ctx.value,
                });
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Unary => {
                self.compile_expr_operation(
                    expr.data
                        .get_data()
                        .nodes
                        .get("expr")
                        .expect("Node.nodes.expr not found"),
                    *ctx.leftctx.unwrap(),
                );

                match expr.data.get_data().op.expect("Node.op is not present") {
                    OpType::Add => {
                        self.instructions.push(CompilerInstruction::UnaryNeg {
                            a: ctx.left.unwrap(),
                            result: ctx.value,
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    _ => {
                        unimplemented!();
                    }
                }
            }
        }

        self.register_index -= ctx.registers;
    }
}
