//Generate tokens from text

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    Decimal,
    Newline,
    Unknown,
    Plus,
    Eof,
    Asterisk,
    Slash,
    Hyphen,
    Equals,
    Identifier,
    LParen,
    RParen,
    LCurly,
    RCurly,
    Keyword,
    Comma,
    String,
    LSquare,
    RSquare,
    Colon,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Decimal => write!(f, "decimal"),
            Self::Newline => write!(f, "newline"),
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::Plus => write!(f, "plus"),
            Self::Eof => write!(f, "EOF"),
            Self::Asterisk => write!(f, "asterisk"),
            Self::Slash => write!(f, "slash"),
            Self::Hyphen => write!(f, "hyphen"),
            Self::Equals => write!(f, "equals"),
            Self::Identifier => write!(f, "identifier"),
            Self::LParen => write!(f, "l-paren"),
            Self::RParen => write!(f, "r-paren"),
            Self::LCurly => write!(f, "l-curly"),
            Self::RCurly => write!(f, "r-curly"),
            Self::Keyword => write!(f, "keyword"),
            Self::Comma => write!(f, "comma"),
            Self::String => write!(f, "string"),
            Self::LSquare => write!(f, "l-square"),
            Self::RSquare => write!(f, "r-square"),
            Self::Colon => write!(f, "colon"),
        }
    }
}

#[derive(Clone)]
pub struct Lexer<'life> {
    pub idx: usize,
    pub current: u8,
    pub len: usize,
    pub line: usize,
    pub col: usize,
    pub info: &'life crate::fileinfo::FileInfo<'life>,
    pub kwds: Vec<String>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let cur: char = self.current.into();

        if cur.is_ascii_digit() {
            Some(make_decimal(self))
        } else if cur.is_alphabetic() {
            Some(make_identifier(self))
        } else if cur == '"' {
            Some(make_string(self))
        } else if cur == '\n' {
            Some(add_char_token(self, cur, TokenType::Newline))
        } else if cur == '#' {
            advance(self);
            while (self.current as char) != '\n' && (self.current as char) != '\0' {
                advance(self);
            }
            self.next()
        } else if cur.is_whitespace() {
            advance(self);
            while (self.current as char).is_whitespace() {
                advance(self);
            }
            self.next()
        } else if cur == '+' {
            Some(add_char_token(self, cur, TokenType::Plus))
        } else if cur == '*' {
            Some(add_char_token(self, cur, TokenType::Asterisk))
        } else if cur == '/' {
            Some(add_char_token(self, cur, TokenType::Slash))
        } else if cur == '-' {
            Some(add_char_token(self, cur, TokenType::Hyphen))
        } else if cur == '=' {
            Some(add_char_token(self, cur, TokenType::Equals))
        } else if cur == '(' {
            Some(add_char_token(self, cur, TokenType::LParen))
        } else if cur == ')' {
            Some(add_char_token(self, cur, TokenType::RParen))
        } else if cur == '{' {
            Some(add_char_token(self, cur, TokenType::LCurly))
        } else if cur == '}' {
            Some(add_char_token(self, cur, TokenType::RCurly))
        } else if cur == ',' {
            Some(add_char_token(self, cur, TokenType::Comma))
        } else if cur == '[' {
            Some(add_char_token(self, cur, TokenType::LSquare))
        } else if cur == ']' {
            Some(add_char_token(self, cur, TokenType::RSquare))
        } else if cur == ':' {
            Some(add_char_token(self, cur, TokenType::Colon))
        } else if cur == '\0' {
            if self.len == 0 {
                self.len = 1;
                return Some(add_char_token(self, cur, TokenType::Eof));
            }
            None
        } else {
            Some(add_char_token(self, cur, TokenType::Unknown))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub data: String,
    pub tp: TokenType,
    pub line: usize,
    pub startcol: usize, //Inclusive
    pub endcol: usize,   //Exclusive
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: '{}'", self.tp, self.data)
    }
}

pub fn new<'a>(
    data: &'a [u8],
    info: &'a crate::fileinfo::FileInfo,
    kwds: Vec<String>,
) -> Lexer<'a> {
    Lexer {
        idx: 0,
        current: if !data.is_empty() { data[0] } else { b'\0' },
        len: data.len(),
        line: 0,
        col: 0,
        info,
        kwds,
    }
}

fn advance(lexer: &mut Lexer) {
    lexer.idx += 1;

    lexer.col += 1;

    if lexer.idx >= lexer.len {
        lexer.current = b'\0';
        return;
    }

    if lexer.current == b'\n' || lexer.current == b'\r' {
        lexer.line += 1;
        lexer.col = 0;
    }

    lexer.current = lexer.info.data[lexer.idx];
}

#[allow(dead_code)]
pub fn print_tokens(lexer: Lexer) {
    println!("Generated tokens:\n========================");
    println!("------------------------");
    let mut idx: usize = 1;
    for tok in lexer.into_iter() {
        println!("{} | {} {}", idx, tok, tok.line);
        idx += 1;
    }
    println!("========================");
}

pub fn add_char_token(lexer: &mut Lexer, val: char, tp: TokenType) -> Token {
    let res = Token {
        data: String::from(val),
        tp,
        line: lexer.line,
        startcol: lexer.col,
        endcol: lexer.col + 1,
    };
    advance(lexer);

    res
}

fn make_decimal(lexer: &mut Lexer) -> Token {
    let mut data = String::from("");
    let start = lexer.col;

    let mut end = lexer.col;
    let mut line = lexer.line;

    while (lexer.current as char).is_numeric() || lexer.current == b'_' {
        data.push(lexer.current as char);
        end = lexer.col;
        line = lexer.line;
        advance(lexer);
        if lexer.current == b'.' {
            data.push(lexer.current as char);
            advance(lexer);
        }
    }

    Token {
        data,
        tp: TokenType::Decimal,
        line,
        startcol: start,
        endcol: end + 1,
    }
}

fn make_identifier(lexer: &mut Lexer) -> Token {
    let mut data = String::from("");
    let start = lexer.col;

    let mut end = lexer.col;
    let mut line = lexer.line;

    while (lexer.current as char).is_alphanumeric() || lexer.current == b'_' {
        data.push(lexer.current as char);
        end = lexer.col;
        line = lexer.line;
        advance(lexer);
        if lexer.current == b'.' {
            data.push(lexer.current as char);
            advance(lexer);
        }
    }

    if lexer.kwds.contains(&data) {
        return Token {
            data,
            tp: TokenType::Keyword,
            line,
            startcol: start,
            endcol: end + 1,
        };
    }

    Token {
        data,
        tp: TokenType::Identifier,
        line,
        startcol: start,
        endcol: end + 1,
    }
}

fn make_string(lexer: &mut Lexer) -> Token {
    let mut data = String::from("");
    let start = lexer.col;

    let mut end = lexer.col;
    let mut line = lexer.line;
    advance(lexer);

    while (lexer.current as char).is_alphanumeric() && (lexer.current as char) != '"' {
        data.push(lexer.current as char);
        end = lexer.col;
        line = lexer.line;
        advance(lexer);
        if lexer.current == b'.' {
            data.push(lexer.current as char);
            advance(lexer);
        }
    }
    advance(lexer);

    Token {
        data,
        tp: TokenType::String,
        line,
        startcol: start,
        endcol: end + 2,
    }
}
