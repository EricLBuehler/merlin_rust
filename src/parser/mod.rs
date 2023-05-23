//Create AST from lexer-generated-tokens

pub mod nodes;

use crate::fileinfo::FileInfo;
use crate::lexer::{Lexer, Token, TokenType};

use crate::errors::{raise_error, ErrorType};

use crate::parser::nodes::Node;

mod precedence;
use precedence::Precedence;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: Token,
    idx: usize,
    info: &'a FileInfo<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub startcol: usize,
    pub endcol: usize,
    pub line: usize,
}

impl Position {
    fn create_from_parts(startcol: usize, endcol: usize, line: usize) -> Position {
        Position { startcol, endcol, line}
    }
}

//Atom: In-place (not left off after seq)
//Expr, Statements, etc: Next (leave off on next)

pub fn new<'a>(lexer: Lexer, info: &'a FileInfo) -> Parser<'a> {
    let tokens: Vec<_> = lexer.collect();
    return Parser   {   tokens: tokens.to_owned(), 
                        current: tokens.first().unwrap().to_owned(),
                        idx: 1,
                        info,
                    };
}

impl<'a> Parser<'a> {
    fn advance(&mut self) -> Token {
        self.idx += 1;
        if self.tokens.get(self.idx-1).is_none() {
            self.current = Token {
                data: String::from("\0"),
                tp: TokenType::Eof,
                line: 0,
                startcol: 0,
                endcol: 0,
            };
            return self.current.to_owned();
        }
        self.current = self.tokens.get(self.idx-1).unwrap().to_owned();
        
        self.current.to_owned()
    }

    fn skip_newlines(&mut self) {
        while self.current_is_type(TokenType::Newline) {
            self.advance();
        }
    }

    fn current_is_type(&self, tp: TokenType) -> bool {
        self.current.tp  == tp
    }

    fn raise_error(&mut self, error: &str, errtp: ErrorType) -> !{
        raise_error(error, errtp, &Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), self.info);
    }

    fn get_precedence(&self) -> Precedence {
        match self.current.tp {
            TokenType::Plus | TokenType::Hyphen => {
                Precedence::Sum
            },
            TokenType::Asterisk | TokenType::Slash => {
                Precedence::Product
            },
            _ => {
                Precedence::Lowest
            },
        }
    }

    // ===========================================
    // ===========================================

    pub fn generate_ast(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        
        while !self.current_is_type(TokenType::Eof) {
            nodes.push(self.parse_statement());
            self.skip_newlines();
        }

        nodes
    }

    fn parse_statement(&mut self) -> Node {
        match self.current.tp {
            _ => {
                self.expr(Precedence::Lowest)
            }
        }
    }

    fn is_atomic(&mut self) -> bool {
        matches!(self.current.tp, TokenType::Decimal)
    }

    fn atom(&mut self) -> Option<Node> {
        match self.current.tp {
            TokenType::Decimal => Some(self.generate_decimal()),
            _ => None,
        }
    }

    fn expr(&mut self, precedence: Precedence) -> Node {
        let mut left;
        
        match self.atom() {
            None => self.raise_error("Invalid or unexpected token.", ErrorType::UnexpectedToken),
            Some(val) => { left = val },
        }
        
        self.advance();
        while !self.current_is_type(TokenType::Eof) && (precedence as u32) < (self.get_precedence() as u32){
            match self.current.tp {
                TokenType::Plus |
                TokenType::Hyphen |
                TokenType::Asterisk |
                TokenType::Slash => {
                    left = self.generate_binary(left, self.get_precedence());
                }
                _ => {
                    return left;
                }
            }
        }
        if self.is_atomic() {
            self.raise_error("Invalid or unexpected token.", ErrorType::UnexpectedToken);
        }
        
        left
    }

    // ============ Atomic ==============

    fn generate_decimal(&mut self) -> Node {
        nodes::Node::new(Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                nodes::NodeType::Decimal, 
                                Box::new(nodes::DecimalNode {value: self.current.data.to_owned()}))
    }

    // ============ Expr ==============

    fn generate_binary(&mut self, left: Node, precedence: Precedence) -> Node {
        let tp = match self.current.tp {
            TokenType::Plus => nodes::BinaryOpType::Add,
            TokenType::Hyphen => nodes::BinaryOpType::Sub,
            TokenType::Asterisk => nodes::BinaryOpType::Mul,
            TokenType::Slash => nodes::BinaryOpType::Div,
            _ => {panic!()}};
            
        self.advance();
        
        nodes::Node::new(Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                nodes::NodeType::Binary, 
                                Box::new(nodes::BinaryNode {left, right: self.expr(precedence), op: tp}))
    }
}