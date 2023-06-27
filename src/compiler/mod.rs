//Generate bytecode from AST
use crate::objects::{exceptionobject, RawObject};
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
use itertools::{izip, Itertools};
use std::fmt::Debug;
use std::marker::PhantomData;
use trc::Trc;

pub struct Compiler<'a> {
    instructions: Vec<CompilerInstruction<'a>>,
    consts: Vec<Object<'a>>,
    names: HashMap<String, i32>,
    info: &'a FileInfo<'a>,
    vm: Trc<VM<'a>>,
    positions: Vec<(Position, Position)>,
    register_index: i32,
    register_max: i32,

    undef_index: i32,
    undef_names: HashMap<i32, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompilerInstruction<'a> {
    BinaryAdd {
        a: CompilerRegister,
        b: CompilerRegister,
        result: CompilerRegister,
        i: usize,
    },
    BinarySub {
        a: CompilerRegister,
        b: CompilerRegister,
        result: CompilerRegister,
        i: usize,
    },
    BinaryMul {
        a: CompilerRegister,
        b: CompilerRegister,
        result: CompilerRegister,
        i: usize,
    },
    BinaryDiv {
        a: CompilerRegister,
        b: CompilerRegister,
        result: CompilerRegister,
        i: usize,
    },
    CopyRegister {
        from: CompilerRegister,
        to: CompilerRegister,
        i: usize,
    },
    MakeFunction {
        nameidx: usize,
        argsidx: usize,
        codeidx: usize,
        out: CompilerRegister,
    }, //All are in consts
    Call {
        callableregister: CompilerRegister,
        result: CompilerRegister,
        arg_registers: Vec<RegisterContext>,
        i: usize,
    },
    Return {
        register: CompilerRegister,
        i: usize,
    },
    UnaryNeg {
        a: CompilerRegister,
        result: CompilerRegister,
        i: usize,
    },
    BuildList {
        result: CompilerRegister,
        value_registers: Vec<CompilerRegister>,
        i: usize,
    },
    BuildDict {
        result: CompilerRegister,
        key_registers: Vec<CompilerRegister>,
        value_registers: Vec<CompilerRegister>,
        i: usize,
    },
    MakeClass {
        name: String,
        methods: HashMap<i32, String>,
        out: CompilerRegister,
        bytecode: Trc<Bytecode<'a>>,
    },
    AttrLoad {
        left: CompilerRegister,
        attridx: CompilerRegister,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerRegister {
    R(usize),
    V(usize),
    C(usize),
}

impl From<CompilerRegister> for usize {
    fn from(value: CompilerRegister) -> Self {
        match value {
            CompilerRegister::V(v) => v,
            CompilerRegister::R(v) => v,
            CompilerRegister::C(v) => v,
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Bytecode<'a> {
    pub instructions: Vec<CompilerInstruction<'a>>,
    pub consts: Vec<Object<'a>>,
    pub names: HashMap<i32, String>,
    pub positions: Vec<(Position, Position)>,
    pub n_registers: i32,
    pub n_variables: i32,
    _marker: PhantomData<&'a ()>,
}

impl Debug for Bytecode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.instructions)
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterContext {
    pub value: CompilerRegister,
    left: Option<CompilerRegister>,
    leftctx: Option<Box<RegisterContext>>,
    right: Option<CompilerRegister>,
    rightctx: Option<Box<RegisterContext>>,
    args: Option<Vec<RegisterContext>>,
    mapping: Option<(Vec<RegisterContext>, Vec<RegisterContext>)>,
    registers: i32, //How many registers this instruction needs - if it uses other exprs this should be 0
}

impl<'a> Compiler<'a> {
    pub fn new(info: &'a FileInfo<'a>, vm: Trc<VM<'a>>) -> Compiler<'a> {
        Compiler {
            instructions: Vec::new(),
            consts: Vec::new(),
            names: HashMap::new(),
            info,
            vm,
            positions: Vec::new(),
            register_index: 0,
            register_max: 0,
            undef_index: 0,
            undef_names: HashMap::new(),
        }
    }

    pub fn generate_bytecode(&mut self, ast: &Vec<Node>) -> Trc<Bytecode<'a>> {
        for head_node in ast {
            self.compile_statement(head_node);
        }
        Trc::new(Bytecode {
            instructions: self.instructions.clone(),
            consts: self.consts.clone(),
            names: self.names.iter().map(|(k, v)| (*v, k.clone())).collect(),
            positions: self.positions.clone(),
            n_registers: self.register_max,
            n_variables: self.names.len() as i32,
            _marker: PhantomData,
        })
    }

    fn compile_statement(&mut self, expr: &Node) {
        match expr.tp {
            NodeType::Decimal
            | NodeType::Binary
            | NodeType::Identifier
            | NodeType::StoreNode
            | NodeType::Call
            | NodeType::Return
            | NodeType::Unary
            | NodeType::String
            | NodeType::List
            | NodeType::Dict
            | NodeType::AttrLoad => {
                let ctx = self.compile_expr_values(expr);
                self.compile_expr_operation(expr, ctx);
            }
            NodeType::Class => {
                let mut registers = 0;
                let name = expr
                    .data
                    .get_data()
                    .raw
                    .get("name")
                    .expect("Node.raw.name not found")
                    .clone();

                let mut compiler = Compiler::new(self.info, self.vm.clone());
                let bytecode = compiler.generate_bytecode(
                    expr.data
                        .get_data()
                        .nodearr
                        .expect("Node.nodearr is not present"),
                );

                self.instructions.push(CompilerInstruction::MakeClass {
                    name: name.clone(),
                    methods: bytecode.names.clone(),
                    out: CompilerRegister::R(self.register_index.try_into().unwrap()),
                    bytecode,
                });
                increment_reg_num!(self);
                registers += 1;

                self.positions.push((expr.start, expr.end));

                self.names.insert(name, self.names.len() as i32);
                self.instructions.push(CompilerInstruction::CopyRegister {
                    from: CompilerRegister::R((self.register_index - 1).try_into().unwrap()),
                    to: CompilerRegister::V(self.names.len() - 1),
                    i: self.instructions.len(),
                });
                self.positions.push((expr.start, expr.end));

                self.register_index -= registers;
            }
            NodeType::Function => {
                let mut registers = 0;
                let name_str = expr
                    .data
                    .get_data()
                    .raw
                    .get("name")
                    .expect("Node.raw.name not found")
                    .clone();

                let name = stringobject::string_from(self.vm.clone(), name_str.clone());
                let mut nameidx = usize::MAX;
                for (i, var) in self.consts.iter().enumerate() {
                    if unsafe {
                        (var.tp.eq.unwrap())(var.clone(), name.clone())
                            .unwrap()
                            .internals
                            .bool
                    } {
                        nameidx = i;
                        break;
                    }
                }
                if nameidx == usize::MAX {
                    self.consts.push(name);
                    nameidx = self.consts.len() - 1;
                }

                let mut names = HashMap::new();
                let mut args = Vec::new();
                for (i, arg) in expr
                    .data
                    .get_data()
                    .args
                    .expect("Node.args is not present")
                    .iter()
                    .enumerate()
                {
                    args.push(stringobject::string_from(self.vm.clone(), arg.clone()));

                    names.insert(arg.to_string(), i as i32);
                }

                let args = listobject::list_from(self.vm.clone(), args);
                let mut argsidx = usize::MAX;
                for (i, var) in self.consts.iter().enumerate() {
                    if unsafe {
                        (var.tp.eq.unwrap())(var.clone(), args.clone())
                            .unwrap()
                            .internals
                            .bool
                    } {
                        argsidx = i;
                        break;
                    }
                }
                if argsidx == usize::MAX {
                    self.consts.push(args);
                    argsidx = self.consts.len() - 1;
                }

                let mut compiler = Compiler::new(self.info, self.vm.clone());
                compiler.names = names;
                let bytecode = compiler.generate_bytecode(
                    expr.data
                        .get_data()
                        .nodearr
                        .expect("Node.nodearr is not present"),
                );

                let code = codeobject::code_from(self.vm.clone(), bytecode);
                let mut codeidx = usize::MAX;
                for (i, var) in self.consts.iter().enumerate() {
                    if unsafe {
                        (var.tp.eq.unwrap())(var.clone(), code.clone())
                            .unwrap()
                            .internals
                            .bool
                    } {
                        codeidx = i;
                        break;
                    }
                }
                if codeidx == usize::MAX {
                    self.consts.push(code);
                    codeidx = self.consts.len() - 1;
                }

                self.instructions.push(CompilerInstruction::MakeFunction {
                    nameidx,
                    argsidx,
                    codeidx,
                    out: CompilerRegister::R(self.register_index.try_into().unwrap()),
                });
                increment_reg_num!(self);
                registers += 1;

                self.positions.push((expr.start, expr.end));

                self.names.insert(name_str, self.names.len() as i32);
                self.instructions.push(CompilerInstruction::CopyRegister {
                    from: CompilerRegister::R((self.register_index - 1).try_into().unwrap()),
                    to: CompilerRegister::V(self.names.len() - 1),
                    i: self.instructions.len(),
                });
                self.positions.push((expr.start, expr.end));
                self.register_index -= registers;
            }
        }
    }

    fn raise_exc_pos(&mut self, exc_obj: Object<'a>, start: Position, end: Position) -> ! {
        let header: String = match RawObject::object_repr_safe(exc_obj) {
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

    //Compile the values of the node - load them all.
    //Only increment the register_idx if new data is being added.
    //That is - the node is atomic and does not need any other nodes.
    fn compile_expr_values(&mut self, expr: &Node) -> RegisterContext {
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

                maybe_handle_exception_pos!(self, int, expr.start, expr.end);
                let mut idx = usize::MAX;
                for (i, var) in self.consts.iter().enumerate() {
                    if unsafe {
                        (var.tp.eq.unwrap())(var.clone(), int.unwrap())
                            .unwrap()
                            .internals
                            .bool
                    } {
                        idx = i;
                        break;
                    }
                }
                if idx == usize::MAX {
                    self.consts.push(int.unwrap());
                    idx = self.consts.len() - 1;
                }

                let res = RegisterContext {
                    value: CompilerRegister::C(idx),
                    left: None,
                    leftctx: None,
                    right: None,
                    rightctx: None,
                    args: None,
                    mapping: None,
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

                RegisterContext {
                    value: CompilerRegister::R(old.try_into().unwrap()),
                    left: Some(left.value),
                    leftctx: Some(Box::new(left)),
                    right: Some(right.value),
                    rightctx: Some(Box::new(right)),
                    args: None,
                    mapping: None,
                    registers: 0,
                }
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
                    value: CompilerRegister::R(old.try_into().unwrap()),
                    left: Some(expr.value),
                    leftctx: Some(Box::new(expr)),
                    right: None,
                    rightctx: None,
                    args: None,
                    mapping: None,
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

                if var.is_none() {
                    let name = expr
                        .data
                        .get_data()
                        .raw
                        .get("name")
                        .expect("Node.raw.name not found")
                        .clone();
                    let exc = exceptionobject::nameexc_from_str(
                        self.vm.clone(),
                        &format!("Name '{}' not defined", name),
                        expr.start,
                        expr.end,
                    );
                    self.raise_exc_pos(exc, expr.start, expr.end);
                }

                RegisterContext {
                    value: CompilerRegister::V(match var {
                        Some(v) => (*v.1).try_into().unwrap(),
                        None => {
                            self.undef_index -= 1;
                            self.undef_names.insert(
                                self.undef_index,
                                expr.data
                                    .get_data()
                                    .raw
                                    .get("name")
                                    .expect("Node.raw.name not found")
                                    .clone(),
                            );
                            self.undef_index.try_into().unwrap()
                        }
                    }),
                    left: None,
                    leftctx: None,
                    right: None,
                    rightctx: None,
                    args: None,
                    mapping: None,
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
                let old = self.register_index;
                let callable = self.compile_expr_values(name);

                let mut args = Vec::new();
                for arg in expr
                    .data
                    .get_data()
                    .nodearr
                    .expect("Node.nodearr is not present")
                {
                    let arg = self.compile_expr_values(arg);
                    args.push(arg);
                }

                RegisterContext {
                    value: CompilerRegister::R(old.try_into().unwrap()),
                    left: Some(callable.value),
                    leftctx: Some(Box::new(callable)),
                    right: None,
                    rightctx: None,
                    args: Some(args),
                    mapping: None,
                    registers: 0,
                }
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
                    mapping: None,
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

                RegisterContext {
                    value: CompilerRegister::R(old.try_into().unwrap()),
                    left: Some(var.value),
                    leftctx: Some(Box::new(var)),
                    right: None,
                    rightctx: None,
                    args: None,
                    mapping: None,
                    registers: 0,
                }
            }
            NodeType::String => {
                let str = stringobject::string_from(
                    self.vm.clone(),
                    expr.data
                        .get_data()
                        .raw
                        .get("value")
                        .expect("Node.raw.value not found")
                        .to_string(),
                );

                let mut idx = usize::MAX;
                for (i, var) in self.consts.iter().enumerate() {
                    if unsafe {
                        (var.tp.eq.unwrap())(var.clone(), str.clone())
                            .unwrap()
                            .internals
                            .bool
                    } {
                        idx = i;
                        break;
                    }
                }
                if idx == usize::MAX {
                    self.consts.push(str);
                    idx = self.consts.len() - 1;
                }

                let res = RegisterContext {
                    value: CompilerRegister::C(idx),
                    left: None,
                    leftctx: None,
                    right: None,
                    rightctx: None,
                    args: None,
                    mapping: None,
                    registers: 1,
                };
                increment_reg_num!(self);
                res
            }
            NodeType::List => {
                let old = self.register_index;
                let mut args = Vec::new();
                for arg in expr
                    .data
                    .get_data()
                    .nodearr
                    .expect("Node.nodearr is not present")
                {
                    let arg = self.compile_expr_values(arg);
                    args.push(arg);
                }

                RegisterContext {
                    value: CompilerRegister::R(old.try_into().unwrap()),
                    left: None,
                    leftctx: None,
                    right: None,
                    rightctx: None,
                    args: Some(args),
                    mapping: None,
                    registers: 0,
                }
            }
            NodeType::Dict => {
                let old = self.register_index;
                let mut keys = Vec::new();
                for (arg, _) in expr
                    .data
                    .get_data()
                    .mapping
                    .expect("Node.mapping is not present")
                {
                    let arg = self.compile_expr_values(arg);
                    keys.push(arg);
                }

                let mut values = Vec::new();
                for (_, arg) in expr
                    .data
                    .get_data()
                    .mapping
                    .expect("Node.mapping is not present")
                {
                    let arg = self.compile_expr_values(arg);
                    values.push(arg);
                }

                RegisterContext {
                    value: CompilerRegister::R(old.try_into().unwrap()),
                    left: None,
                    leftctx: None,
                    right: None,
                    rightctx: None,
                    args: None,
                    mapping: Some((keys, values)),
                    registers: 0,
                }
            }
            NodeType::AttrLoad => {
                let old = self.register_index;

                let left = self.compile_expr_values(
                    expr.data
                        .get_data()
                        .nodes
                        .get("left")
                        .expect("Node.nodes.left not found"),
                );

                RegisterContext {
                    value: CompilerRegister::R(old.try_into().unwrap()),
                    left: Some(left.value),
                    leftctx: Some(Box::new(left)),
                    right: None,
                    rightctx: None,
                    args: None,
                    mapping: None,
                    registers: 0,
                }
            }
            NodeType::Class | NodeType::Function => {
                unreachable!()
            }
        }
    }

    //Generate the actual instructions that use the RegisterContexts from the value compilation.
    //Do not increment the register number here!
    fn compile_expr_operation(&mut self, expr: &Node, ctx: RegisterContext) {
        match expr.tp {
            NodeType::Decimal => {}
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
                            i: self.instructions.len(),
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    OpType::Sub => {
                        self.instructions.push(CompilerInstruction::BinarySub {
                            a: ctx.left.unwrap(),
                            b: ctx.right.unwrap(),
                            result: ctx.value,
                            i: self.instructions.len(),
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    OpType::Mul => {
                        self.instructions.push(CompilerInstruction::BinaryMul {
                            a: ctx.left.unwrap(),
                            b: ctx.right.unwrap(),
                            result: ctx.value,
                            i: self.instructions.len(),
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    OpType::Div => {
                        self.instructions.push(CompilerInstruction::BinaryDiv {
                            a: ctx.left.unwrap(),
                            b: ctx.right.unwrap(),
                            result: ctx.value,
                            i: self.instructions.len(),
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

                let idx = if self.names.contains_key(
                    expr.data
                        .get_data()
                        .raw
                        .get("name")
                        .expect("Node.raw.name not found"),
                ) {
                    *self
                        .names
                        .get(
                            expr.data
                                .get_data()
                                .raw
                                .get("name")
                                .expect("Node.raw.name not found"),
                        )
                        .unwrap()
                } else {
                    self.names.insert(
                        expr.data
                            .get_data()
                            .raw
                            .get("name")
                            .expect("Node.raw.name not found")
                            .clone(),
                        self.names.len() as i32,
                    );
                    (self.names.len() - 1) as i32
                };

                self.instructions.push(CompilerInstruction::CopyRegister {
                    from: ctx.left.unwrap(),
                    to: CompilerRegister::V(idx.try_into().unwrap()),
                    i: self.instructions.len(),
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
                let name = *expr
                    .data
                    .get_data()
                    .nodes
                    .get("name")
                    .expect("Node.nodes.name not found");
                self.compile_expr_operation(name, *ctx.leftctx.unwrap());

                for arg in izip!(
                    expr.data
                        .get_data()
                        .nodearr
                        .expect("Node.nodearr is not present"),
                    ctx.args.as_ref().unwrap()
                ) {
                    self.compile_expr_operation(arg.0, arg.1.clone());
                }
                self.instructions.push(CompilerInstruction::Call {
                    callableregister: ctx.left.unwrap(),
                    result: ctx.value,
                    arg_registers: ctx.args.unwrap(),
                    i: self.instructions.len(),
                });
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
                    i: self.instructions.len(),
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
                            i: self.instructions.len(),
                        });
                        self.positions.push((expr.start, expr.end));
                    }
                    _ => {
                        unimplemented!();
                    }
                }
            }
            NodeType::String => {}
            NodeType::List => {
                for arg in izip!(
                    expr.data
                        .get_data()
                        .nodearr
                        .expect("Node.nodearr is not present"),
                    ctx.args.as_ref().unwrap()
                ) {
                    self.compile_expr_operation(arg.0, arg.1.clone());
                }
                self.instructions.push(CompilerInstruction::BuildList {
                    result: ctx.value,
                    value_registers: ctx.args.unwrap().iter().map(|x| x.value).collect_vec(),
                    i: self.instructions.len(),
                });
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Dict => {
                for ((key, _), keyctx) in izip!(
                    expr.data
                        .get_data()
                        .mapping
                        .expect("Node.mapping is not present"),
                    &ctx.mapping.as_ref().unwrap().0
                ) {
                    self.compile_expr_operation(key, keyctx.clone());
                }
                for ((_, value), valuectx) in izip!(
                    expr.data
                        .get_data()
                        .mapping
                        .expect("Node.mapping is not present"),
                    &ctx.mapping.as_ref().unwrap().1
                ) {
                    self.compile_expr_operation(value, valuectx.clone());
                }
                self.instructions.push(CompilerInstruction::BuildDict {
                    result: ctx.value,
                    key_registers: ctx
                        .mapping
                        .as_ref()
                        .unwrap()
                        .0
                        .iter()
                        .map(|x| x.value)
                        .collect_vec(),
                    value_registers: ctx.mapping.unwrap().1.iter().map(|x| x.value).collect_vec(),
                    i: self.instructions.len(),
                });
                self.positions.push((expr.start, expr.end));
            }
            NodeType::Class => {
                raise_error(
                    "Class definition is not an expression",
                    ErrorType::FunctionNotExpression,
                    &expr.start,
                    self.info,
                );
            }
            NodeType::AttrLoad => {
                self.compile_expr_operation(
                    expr.data
                        .get_data()
                        .nodes
                        .get("left")
                        .expect("Node.nodes.left not found"),
                    *ctx.leftctx.unwrap(),
                );

                let attr = stringobject::string_from(
                    self.vm.clone(),
                    expr.data
                        .get_data()
                        .raw
                        .get("attr")
                        .expect("Node.raw.attr not found")
                        .to_string(),
                );
                let mut idx = usize::MAX;
                for (i, var) in self.consts.iter().enumerate() {
                    if unsafe {
                        (var.tp.eq.unwrap())(var.clone(), attr.clone())
                            .unwrap()
                            .internals
                            .bool
                    } {
                        idx = i;
                        break;
                    }
                }
                if idx == usize::MAX {
                    self.consts.push(attr);
                    idx = self.consts.len() - 1;
                }

                self.instructions.push(CompilerInstruction::AttrLoad {
                    left: ctx.left.unwrap(),
                    attridx: CompilerRegister::C(idx),
                });
                self.positions.push((expr.start, expr.end));
            }
        }

        self.register_index -= ctx.registers;
    }
}
