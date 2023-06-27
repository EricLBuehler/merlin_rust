//Create AST from lexer-generated-tokens

pub mod nodes;

use crate::fileinfo::FileInfo;
use crate::lexer::{Lexer, Token, TokenType};

use crate::errors::{raise_error, ErrorType};

use crate::parser::nodes::Node;

mod precedence;
use precedence::Precedence;

use self::nodes::NodeType;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: Token,
    idx: usize,
    info: &'a FileInfo<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub startcol: usize,
    pub endcol: usize,
    pub line: usize,
}

impl Position {
    fn create_from_parts(startcol: usize, endcol: usize, line: usize) -> Position {
        Position {
            startcol,
            endcol,
            line,
        }
    }
}

//Atom: In-place (not left off after seq). If uses expr, then do not .reverse
//Expr, Statements, etc: Next (leave off on next)

pub fn new<'a>(lexer: Lexer, info: &'a FileInfo) -> Parser<'a> {
    let tokens: Vec<_> = lexer.collect();
    return Parser {
        tokens: tokens.to_owned(),
        current: tokens.first().expect("No tokens").to_owned(),
        idx: 1,
        info,
    };
}

macro_rules! allowed_to_vec {
    ($allowed: expr) => {
        $allowed
            .into_iter()
            .map(|itm| "'".to_owned() + itm + "'")
            .collect::<Vec<String>>()
            .join(", ")
    };
}

impl<'a> Parser<'a> {
    fn advance(&mut self) -> Token {
        self.idx += 1;
        if self.tokens.get(self.idx - 1).is_none() {
            self.current = Token {
                data: String::from("\0"),
                tp: TokenType::Eof,
                line: 0,
                startcol: 0,
                endcol: 0,
            };
            return self.current.to_owned();
        }
        self.current = self
            .tokens
            .get(self.idx - 1)
            .expect("Tokens index out of range")
            .to_owned();

        self.current.to_owned()
    }

    fn reverse(&mut self) -> Token {
        self.idx -= 1;
        if self.tokens.get(self.idx - 1).is_none() {
            self.current = Token {
                data: String::from("\0"),
                tp: TokenType::Eof,
                line: 0,
                startcol: 0,
                endcol: 0,
            };
            return self.current.to_owned();
        }
        self.current = self
            .tokens
            .get(self.idx - 1)
            .expect("Tokens index out of range")
            .to_owned();

        self.current.to_owned()
    }

    fn skip_newlines(&mut self) {
        while self.current_is_type(TokenType::Newline) {
            self.advance();
        }
    }

    fn current_is_type(&self, tp: TokenType) -> bool {
        self.current.tp == tp
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

    fn raise_error(&mut self, error: &str, errtp: ErrorType) -> ! {
        raise_error(
            error,
            errtp,
            &Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            self.info,
        );
    }

    fn get_precedence(&self) -> Precedence {
        match self.current.tp {
            TokenType::Plus | TokenType::Hyphen => Precedence::Sum,
            TokenType::Asterisk | TokenType::Slash => Precedence::Product,
            _ => Precedence::Lowest,
        }
    }

    fn ensure_not_eof(&mut self, allowed: Vec<&str>) {
        if self.current_is_type(TokenType::Eof) {
            self.raise_error(
                &format!(
                    "Unexpected EOF (expected one of {}).",
                    allowed_to_vec!(allowed)
                ),
                ErrorType::UnexpectedEOF,
            )
        }
    }

    fn expect(&mut self, typ: TokenType) {
        if !self.current_is_type(typ.clone()) {
            self.raise_error(
                format!(
                    "Invalid or unexpected token (expected '{}', got '{}').",
                    typ, self.current.tp
                )
                .as_str(),
                ErrorType::UnexpectedToken,
            )
        }
    }

    fn expect_and<F>(&mut self, typ: TokenType, fun: F)
    where
        F: FnOnce(&Token) -> bool,
    {
        if !self.current_is_type(typ.clone()) && !fun(&self.current) {
            self.raise_error(
                format!(
                    "Invalid or unexpected token (expected '{}', got '{}').",
                    typ, self.current.tp
                )
                .as_str(),
                ErrorType::UnexpectedToken,
            )
        }
    }

    // ===========================================
    // ===========================================

    pub fn generate_ast(&mut self) -> Vec<Node> {
        self.block(None)
    }

    #[allow(clippy::type_complexity)]
    fn block(&mut self, allowed: Option<(&dyn Fn(&Token) -> bool, Vec<&str>)>) -> Vec<Node> {
        let mut nodes = Vec::new();

        while !self.current_is_type(TokenType::Eof) && !self.current_is_type(TokenType::RCurly) {
            if allowed.is_some() && !allowed.as_ref().unwrap().0(&self.current) {
                self.raise_error(
                    &format!(
                        "Invalid or unexpected token (expected one of {}).",
                        allowed_to_vec!(allowed.unwrap().1)
                    ),
                    ErrorType::UnexpectedToken,
                );
            }
            nodes.push(self.parse_statement());
            self.skip_newlines();
        }

        nodes
    }

    fn parse_statement(&mut self) -> Node {
        match self.current.tp {
            TokenType::Keyword => self.keyword(),
            _ => self.expr(Precedence::Lowest),
        }
    }

    fn is_atomic(&mut self) -> bool {
        matches!(self.current.tp, TokenType::Decimal)
            || matches!(self.current.tp, TokenType::Identifier)
            || matches!(self.current.tp, TokenType::Hyphen)
            || matches!(self.current.tp, TokenType::LParen)
            || matches!(self.current.tp, TokenType::String)
            || matches!(self.current.tp, TokenType::LCurly)
    }

    fn atom(&mut self) -> Option<Node> {
        match self.current.tp {
            TokenType::Decimal => Some(self.generate_decimal()),
            TokenType::Identifier => Some(self.generate_identifier()),
            TokenType::Hyphen => Some(self.generate_negate()),
            TokenType::LParen => Some(self.generate_grouped()),
            TokenType::String => Some(self.generate_string()),
            TokenType::LSquare => Some(self.generate_list()),
            TokenType::LCurly => Some(self.generate_dict()),
            _ => None,
        }
    }

    fn keyword(&mut self) -> Node {
        if self.current.data == "fn" {
            self.parse_fn()
        } else if self.current.data == "return" {
            self.parse_return()
        } else if self.current.data == "class" {
            self.parse_class()
        } else {
            self.raise_error("Unknown keyword.", ErrorType::UnknownKeyword);
        }
    }

    fn expr(&mut self, precedence: Precedence) -> Node {
        let mut left;

        let atomics = vec!["decimal", "identifier", "-", "(", "string", "["];

        match self.atom() {
            None => self.raise_error(
                &format!(
                    "Invalid or unexpected token (expected one of {}).",
                    allowed_to_vec!(atomics)
                ),
                ErrorType::UnexpectedToken,
            ),
            Some(val) => left = val,
        }

        if left.tp == NodeType::StoreNode {
            return left;
        }

        let prev = self.current.clone();
        self.advance();
        let mut i = 0;
        while !self.current_is_type(TokenType::Eof)
            && (precedence as u32) < (self.get_precedence() as u32)
        {
            match self.current.tp {
                TokenType::Plus | TokenType::Hyphen | TokenType::Asterisk | TokenType::Slash => {
                    left = self.generate_binary(left, self.get_precedence());
                }
                TokenType::LParen => {
                    left = self.generate_call(left);
                }
                _ => {
                    return left;
                }
            }
            i += 1;
        }

        if self.is_atomic()
            && i == 0
            && !self.current_is_type(TokenType::Eof)
            && !self.current_is_type(TokenType::Newline)
            && prev.tp != TokenType::Newline
        {
            self.raise_error(
                "Trailing atomic tokens are not allowed.",
                ErrorType::TrailingAtomics,
            );
        }

        left
    }

    // ============ Atomic ==============

    fn generate_decimal(&mut self) -> Node {
        nodes::Node::new(
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::Decimal,
            Box::new(nodes::DecimalNode {
                value: self.current.data.to_owned(),
            }),
        )
    }

    fn generate_identifier(&mut self) -> Node {
        let starttok = self.current.clone();
        let name: String = self.current.data.clone();
        if self.next_is_type(TokenType::Equals) {
            self.advance();
            self.advance();
            let expr = self.expr(Precedence::Lowest);
            return nodes::Node::new(
                Position::create_from_parts(starttok.startcol, starttok.endcol, starttok.line),
                Position::create_from_parts(
                    self.current.startcol,
                    self.current.endcol,
                    self.current.line,
                ),
                nodes::NodeType::StoreNode,
                Box::new(nodes::StoreNode { name, expr }),
            );
        }

        let res = nodes::Node::new(
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::Identifier,
            Box::new(nodes::IdentifierNode { name }),
        );
        if self.next_is_type(TokenType::LParen) {
            self.advance();
            return self.generate_call(res);
        }
        res
    }

    fn generate_negate(&mut self) -> Node {
        self.advance();

        let expr = self.expr(Precedence::Lowest);

        self.reverse();

        nodes::Node::new(
            expr.start,
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::Unary,
            Box::new(nodes::UnaryNode {
                expr,
                op: nodes::OpType::Neg,
            }),
        )
    }

    fn generate_grouped(&mut self) -> Node {
        self.advance();
        self.expr(Precedence::Lowest)
    }

    fn generate_string(&mut self) -> Node {
        nodes::Node::new(
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::String,
            Box::new(nodes::StringNode {
                value: self.current.data.to_owned(),
            }),
        )
    }

    fn generate_list(&mut self) -> Node {
        let start = Position::create_from_parts(
            self.current.startcol,
            self.current.endcol,
            self.current.line,
        );
        self.advance();
        let mut values = Vec::new();
        while !self.current_is_type(TokenType::RSquare) && !self.current_is_type(TokenType::Eof) {
            values.push(self.expr(Precedence::Lowest));
            if self.current_is_type(TokenType::RSquare) {
                self.advance();
                break;
            }
            self.expect(TokenType::Comma);
            self.advance();
        }
        let end = Position::create_from_parts(
            self.current.startcol,
            self.current.endcol,
            self.current.line,
        );

        nodes::Node::new(
            start,
            end,
            nodes::NodeType::List,
            Box::new(nodes::ListNode { values }),
        )
    }

    fn generate_dict(&mut self) -> Node {
        let start = Position::create_from_parts(
            self.current.startcol,
            self.current.endcol,
            self.current.line,
        );
        self.advance();
        let mut values = Vec::new();
        while !self.current_is_type(TokenType::RCurly) && !self.current_is_type(TokenType::Eof) {
            let key = self.expr(Precedence::Lowest);
            self.expect(TokenType::Colon);
            self.advance();
            let value = self.expr(Precedence::Lowest);
            values.push((key, value));

            if self.current_is_type(TokenType::RCurly) {
                self.advance();
                break;
            }
            self.expect(TokenType::Comma);
            self.advance();
        }
        let end = Position::create_from_parts(
            self.current.startcol,
            self.current.endcol,
            self.current.line,
        );

        nodes::Node::new(
            start,
            end,
            nodes::NodeType::Dict,
            Box::new(nodes::DictNode { values }),
        )
    }

    // ============ Expr ==============

    fn generate_binary(&mut self, left: Node, precedence: Precedence) -> Node {
        let tp = match self.current.tp {
            TokenType::Plus => nodes::OpType::Add,
            TokenType::Hyphen => nodes::OpType::Sub,
            TokenType::Asterisk => nodes::OpType::Mul,
            TokenType::Slash => nodes::OpType::Div,
            _ => {
                unreachable!()
            }
        };

        self.advance();

        nodes::Node::new(
            left.start,
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::Binary,
            Box::new(nodes::BinaryNode {
                left,
                right: self.expr(precedence),
                op: tp,
            }),
        )
    }

    fn generate_call(&mut self, left: Node) -> Node {
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
            self.advance();
        }

        nodes::Node::new(
            left.start,
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::Call,
            Box::new(nodes::CallNode { ident: left, args }),
        )
    }

    // ============ Expr ==============

    fn parse_fn(&mut self) -> Node {
        let starttok = self.current.clone();
        self.advance();
        self.ensure_not_eof(vec!["identifier"]);
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
            self.advance();
        }
        self.expect(TokenType::LCurly);
        self.advance();
        self.skip_newlines();
        let code = self.block(None);
        self.skip_newlines();
        self.expect(TokenType::RCurly);
        self.advance();

        nodes::Node::new(
            Position::create_from_parts(starttok.startcol, starttok.endcol, starttok.line),
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::Function,
            Box::new(nodes::FunctionNode { name, args, code }),
        )
    }

    fn parse_return(&mut self) -> Node {
        self.advance();

        let expr = self.expr(Precedence::Lowest);

        nodes::Node::new(
            expr.start,
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::Return,
            Box::new(nodes::ReturnNode { expr }),
        )
    }

    fn parse_class(&mut self) -> Node {
        let starttok = self.current.clone();
        self.advance();
        self.ensure_not_eof(vec!["identifier"]);
        let name = self.current.data.clone();
        self.advance();

        self.expect(TokenType::LCurly);
        self.advance();
        self.skip_newlines();

        self.expect_and(TokenType::Keyword, |tok| tok.data == "fn");
        let code = self.block(Some((
            &|tok| tok.tp == TokenType::Keyword && tok.data == "fn",
            vec!["fn"],
        )));
        self.skip_newlines();
        self.expect(TokenType::RCurly);
        self.advance();

        nodes::Node::new(
            Position::create_from_parts(starttok.startcol, starttok.endcol, starttok.line),
            Position::create_from_parts(
                self.current.startcol,
                self.current.endcol,
                self.current.line,
            ),
            nodes::NodeType::Class,
            Box::new(nodes::ClassNode {
                name,
                methods: code,
            }),
        )
    }
}
