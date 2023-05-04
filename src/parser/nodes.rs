use std::{collections::HashMap, fmt::Debug};
use crate::parser::Position;

#[derive(Debug)]
pub struct Node {
    start: Position,
    end: Position,
    tp: NodeType,
    data: Box<dyn NodeData>,
}

impl Node {
    pub fn new(start: Position, end: Position, tp: NodeType, data: Box<dyn NodeData>) -> Node{
        return Node {start, end, tp, data};
    } 
}

#[derive(Debug)]
pub enum NodeType {
    DECIMAL,
    BINARY,
}

#[derive(Debug)]
pub struct NodeValue<'a> {
    raw: HashMap<String, String>,
    nodes: HashMap<String, &'a Node>,
    op: Option<BinaryOpType>,
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
        return NodeValue {raw: HashMap::new(), nodes: HashMap::new(), op: None};
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
        value.raw.insert(String::from("value"), self.value.to_owned());
        return value;
    }
}

// ========================

#[derive(Debug, Copy, Clone)]
pub enum BinaryOpType {
    ADD,
}

pub struct BinaryNode {
    pub left: Node,
    pub right: Node,
    pub op: BinaryOpType,
}

impl NodeData for BinaryNode {
    fn get_data(&self) -> NodeValue {
        let mut value = NodeValue::new();
        value.nodes.insert(String::from("left"), &self.left);
        value.nodes.insert(String::from("right"), &self.right);
        value.op = Some(self.op);
        return value;
    }
}