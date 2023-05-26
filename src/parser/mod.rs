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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn reverse(&mut self) -> Token {
        self.idx -= 1;
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

    fn next_is_type(&mut self, tp: TokenType) -> bool {
        self.advance();
        if self.current.tp == tp {
            self.reverse();
            return true;
        }
        self.reverse();
        false
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

    fn ensure_not_eof(&mut self) {
        if self.current_is_type(TokenType::Eof) {
            self.raise_error("Unexpected EOF.", ErrorType::UnexpectedEOF)
        }
    }

    fn expect(&mut self, typ: TokenType) {
        if !self.current_is_type(typ.clone()) {
            self.raise_error(format!("Invalid '{}', got '{}'.", typ, self.current.tp).as_str(), ErrorType::UnexpectedEOF)
        }
    }

    // ===========================================
    // ===========================================

    pub fn generate_ast(&mut self) -> Vec<Node> {
        self.block()
    }

    fn block(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        
        while !self.current_is_type(TokenType::Eof) && !self.current_is_type(TokenType::RCurly) {
            nodes.push(self.parse_statement());
            self.skip_newlines();
        }

        nodes
    }

    fn parse_statement(&mut self) -> Node {
        match self.current.tp {
            TokenType::Keyword => {
                self.keyword()
            }
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
            TokenType::Identifier => Some(self.generate_identifier()),
            _ => None,
        }
    }

    fn keyword(&mut self) -> Node {
        if self.current.data == "fn" {
            self.parse_fn()
        }
        else {
            self.raise_error("Unknown keyword.", ErrorType::UnknownKeyword);
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
            self.reverse();
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

    fn generate_identifier(&mut self) -> Node {
        let name: String = self.current.data.clone();
        if self.next_is_type(TokenType::Equals) {
            self.advance();
            self.advance();
            let expr = self.expr(Precedence::Lowest);
            self.reverse();
            return nodes::Node::new(Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                    Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                    nodes::NodeType::StoreNode, 
                                    Box::new(nodes::StoreNode {name, expr}));
        }
        else if self.next_is_type(TokenType::LParen) {
            self.advance();
            self.advance();

            let mut args = Vec::new();            
            while !self.current_is_type(TokenType::RParen) && !self.current_is_type(TokenType::Eof) {
                args.push(self.expr(Precedence::Lowest));
                if self.current_is_type(TokenType::RParen) {
                    self.advance();
                    break;
                }
                self.expect(TokenType::Comma);
            }

            return nodes::Node::new(Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                    Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                    nodes::NodeType::Call, 
                                    Box::new(nodes::CallNode {name, args}));
        }

        nodes::Node::new(Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                    Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                    nodes::NodeType::Identifier, 
                                    Box::new(nodes::IdentifierNode {name}))
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

    // ============ Expr ==============

    fn parse_fn(&mut self) -> Node {
        self.advance();
        self.ensure_not_eof();
        let name = self.current.data.clone();
        let mut args = Vec::new();
        
        self.advance();
        self.expect(TokenType::LParen);
        self.advance();
        while !self.current_is_type(TokenType::RParen) && !self.current_is_type(TokenType::Eof) {
            self.expect(TokenType::Identifier);
            args.push(self.current.data.clone());
            self.advance();
            if self.current_is_type(TokenType::RParen) {
                self.advance();
                break;
            }
            self.expect(TokenType::Comma);
        }
        self.expect(TokenType::LCurly);
        self.advance();
        self.skip_newlines();
        let code = self.block();
        self.skip_newlines();
        self.expect(TokenType::RCurly);
        self.advance();
        
        nodes::Node::new(Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                Position::create_from_parts(self.current.startcol, self.current.endcol, self.current.line), 
                                nodes::NodeType::Function, 
                                Box::new(nodes::FunctionNode {name, args, code}))
    }
} 