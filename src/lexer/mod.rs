//Generate tokens from text

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    DECIMAL,
    NEWLINE,
    UNKNOWN,
    PLUS,
    EOF,
    ASTERISK,
    SLASH,
    HYPHEN,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
       match *self {
           TokenType::DECIMAL => write!(f, "DECIMAL"),
           TokenType::NEWLINE => write!(f, "NEWLINE"),
           TokenType::UNKNOWN => write!(f, "UNKNOWN"),
           TokenType::PLUS => write!(f, "PLUS"),
           TokenType::EOF => write!(f, "EOF"),
           TokenType::ASTERISK => write!(f, "ASTERISK"),
           TokenType::SLASH => write!(f, "SLASH"),
           TokenType::HYPHEN => write!(f, "HYPHEN"),
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

        if cur.is_digit(10) {
            return Some(make_decimal(self));
        }
        else if cur == '\n' {
            return Some(add_char_token(self, cur, TokenType::NEWLINE));
        }
        else if cur == '#' {
            advance(self);
            while (self.current as char) != '\n' && (self.current as char) != '\0'{
                advance(self);
            }
            return self.next();
        }
        else if cur.is_whitespace() {
            advance(self);
            while (self.current as char).is_whitespace(){
                advance(self);
            }
            return self.next();
        }
        else if cur == '+' {
            return Some(add_char_token(self, cur, TokenType::PLUS));
        }
        else if cur == '*' {
            return Some(add_char_token(self, cur, TokenType::ASTERISK));
        }
        else if cur == '/' {
            return Some(add_char_token(self, cur, TokenType::SLASH));
        }
        else if cur == '-' {
            return Some(add_char_token(self, cur, TokenType::HYPHEN));
        }
        else if cur == '\0' {
            return None;
        }
        else {
            return Some(add_char_token(self, cur, TokenType::UNKNOWN));
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub data: String,
    pub tp: TokenType,
    pub line: usize,
    pub startcol: usize, //Inclusive
    pub endcol: usize, //Exclusive
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: '{}'", self.tp, self.data)
    }
}

pub fn new<'a>(data: &'a [u8], info: &'a crate::fileinfo::FileInfo, kwds: Vec<String>) -> Lexer<'a> {
    return Lexer {
        idx: 0,
        current: data[0],
        len: data.len(),
        line: 0,
        col: 0,
        info,
        kwds,
    }
}

fn advance(lexer: &mut Lexer) {
    lexer.idx+=1;

    lexer.col+=1;

    if lexer.idx >= lexer.len {
        lexer.current = b'\0';
        return;
    }

    if lexer.current == b'\n' || lexer.current == b'\r' {
        lexer.line+=1;
        lexer.col=0;
    }
    
    lexer.current = lexer.info.data[lexer.idx];
}

#[allow(dead_code)]
pub fn print_tokens(lexer: Lexer) {
    println!("Generated tokens:\n========================");
    println!("------------------------");
    let mut idx: usize = 1;
    for tok in lexer.into_iter(){
        println!("{} | {} {}", idx, tok, tok.line);
        idx+=1;
    }
    println!("========================");
}

pub fn add_char_token(lexer: &mut Lexer, val: char, tp: TokenType) -> Token {
    let res = Token {
        data: String::from(val),
        tp,
        line: lexer.line,
        startcol: lexer.col,
        endcol: lexer.col+1,
    };
    advance(lexer);
    return res;
}




fn make_decimal(lexer: &mut Lexer) -> Token {
    let mut data = String::from("");
    let start = lexer.col;

    let mut end = lexer.col;
    let mut line = lexer.line;

    let mut i = 0;
    while (lexer.current as char).is_numeric() || lexer.current==b'_' || (i==0 && lexer.current==b'.') {
        data.push(lexer.current as char);
        end=lexer.col;
        line=lexer.line;
        advance(lexer);
        if lexer.current == b'.' {
            data.push(lexer.current as char);
            advance(lexer);
        }
        i+=1;
    }
    
    let tok = Token {
        data: data,
        tp: TokenType::DECIMAL,
        line,
        startcol: start,
        endcol: end+1,
    };
    return tok;
}