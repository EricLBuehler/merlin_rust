use crate::parser::Position;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Node {
    pub start: Position,
    pub end: Position,
    pub tp: NodeType,
    pub data: Box<dyn NodeData>,
}

impl Node {
    pub fn new(start: Position, end: Position, tp: NodeType, data: Box<dyn NodeData>) -> Node {
        Node {
            start,
            end,
            tp,
            data,
        }
    }
}

#[derive(Debug)]
pub enum NodeType {
    Decimal,
    Binary,
    StoreNode,
    Identifier,
    Function,
    Call,
    Return,
    Unary,
}

#[derive(Debug)]
pub struct NodeValue<'a> {
    pub raw: hashbrown::HashMap<String, String>,
    pub nodes: hashbrown::HashMap<String, &'a Node>,
    pub op: Option<OpType>,
    pub nodearr: Option<&'a Vec<Node>>,
    pub args: Option<Vec<String>>,
}

pub trait NodeData {
    fn get_data(&self) -> NodeValue;
}

impl Debug for dyn NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeData{:?}", self.get_data())
    }
}

impl<'a> NodeValue<'a> {
    fn new() -> NodeValue<'a> {
        NodeValue {
            raw: hashbrown::HashMap::new(),
            nodes: hashbrown::HashMap::new(),
            op: None,
            nodearr: None,
            args: None,
        }
    }
}

//===================================================
//===================================================

pub struct DecimalNode {
    pub value: String,
}

impl NodeData for DecimalNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value
            .raw
            .insert(String::from("value"), self.value.to_owned());

        value
    }
}

// ========================

#[derive(Debug, Copy, Clone)]
pub enum OpType {
    Add,
    Sub,
    Mul,
    Div,
    Neg,
}

pub struct BinaryNode {
    pub left: Node,
    pub right: Node,
    pub op: OpType,
}

impl NodeData for BinaryNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("left"), &self.left);
        value.nodes.insert(String::from("right"), &self.right);
        value.op = Some(self.op);

        value
    }
}

// ========================

pub struct StoreNode {
    pub name: String,
    pub expr: Node,
}

impl NodeData for StoreNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("expr"), &self.expr);
        value.raw.insert(String::from("name"), self.name.clone());

        value
    }
}

// ========================

pub struct IdentifierNode {
    pub name: String,
}

impl NodeData for IdentifierNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.raw.insert(String::from("name"), self.name.clone());

        value
    }
}

// ========================

pub struct FunctionNode {
    pub name: String,
    pub args: Vec<String>,
    pub code: Vec<Node>,
}

impl NodeData for FunctionNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.raw.insert(String::from("name"), self.name.clone());
        value.nodearr = Some(&self.code);
        value.args = Some(self.args.clone());

        value
    }
}

// ========================

pub struct CallNode {
    pub ident: Node,
    pub args: Vec<Node>,
}

impl NodeData for CallNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("name"), &self.ident);
        value.nodearr = Some(&self.args);

        value
    }
}

// ========================

pub struct ReturnNode {
    pub expr: Node,
}

impl NodeData for ReturnNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("expr"), &self.expr);

        value
    }
}

// ========================

pub struct UnaryNode {
    pub expr: Node,
    pub op: OpType,
}

impl NodeData for UnaryNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("expr"), &self.expr);
        value.op = Some(self.op);

        value
    }
}
