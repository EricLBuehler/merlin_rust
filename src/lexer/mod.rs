//Generate tokens from text

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    DECIMAL,
    EOF,
    NEWLINE,
    UNKNOWN,
    PLUS,
}

pub struct Lexer<'life> {
    pub idx: usize,
    pub data: &'life [u8],
    pub current: u8,
    pub len: usize,
    pub line: usize,
    pub col: usize,
    pub info: crate::fileinfo::FileInfo<'life>,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub data: String,
    pub tp: TokenType,
    pub line: usize,
    pub startcol: usize, //Inclusive
    pub endcol: usize, //Exclusive
}


impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
       match *self {
           TokenType::DECIMAL => write!(f, "DECIMAL"),
           TokenType::EOF => write!(f, "EOF"),
           TokenType::NEWLINE => write!(f, "NEWLINE"),
           TokenType::UNKNOWN => write!(f, "UNKNOWN"),
           TokenType::PLUS => write!(f, "PLUS"),
       }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: '{}'", self.tp, self.data)
    }
}

pub fn new<'a>(data: &'a [u8], info: &crate::fileinfo::FileInfo<'a>) -> Lexer<'a> {
    return Lexer {
        idx: 0,
        data: data.clone(),
        current: data[0],
        len: data.len(),
        line: 0,
        col: 0,
        info: info.clone(),
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
    
    lexer.current = lexer.data[lexer.idx];
}

#[allow(dead_code)]
pub fn print_tokens(len: usize, tokens: &Vec<Token>) {
    println!("\n\nGenerated tokens:\n========================");
    println!("Token list ({} tokens)", len);
    println!("------------------------");
    let mut idx: usize = 1;
    for tok in tokens{
        println!("{} | {} {}", idx, tok, tok.line);
        idx+=1;
    }
    println!("========================");
}

fn add_char_token(lexer: &mut Lexer, tokens: &mut Vec<Token>, val: char, tp: TokenType) {
    tokens.push(Token {
        data: String::from(val),
        tp,
        line: lexer.line,
        startcol: lexer.col,
        endcol: lexer.col+1,
    });
    advance(lexer);
}

pub fn generate_tokens(lexer: &mut Lexer, kwds: &Vec<String>) -> (usize, Vec<Token>) {  
    let mut tokens = Vec::new();

    while lexer.current!=b'\0' {
        let cur: char = lexer.current.into();

        if cur.is_digit(10) {
            tokens.push(make_decimal(lexer));
        }
        else if cur == '\n' {
            add_char_token(lexer, &mut tokens, cur, TokenType::NEWLINE);
        }
        else if cur.is_whitespace() {
            advance(lexer);
        }
        else if cur == '+' {
            add_char_token(lexer, &mut tokens, cur, TokenType::PLUS);
        }
        else {
            tokens.push(Token {
                data: String::from(cur),
                tp: TokenType::UNKNOWN,
                line: lexer.line,
                startcol: lexer.col,
                endcol: lexer.col+1,
            });
            advance(lexer);
        }
    }

    tokens.push(Token {
        data: String::from("\\0"),
        tp: TokenType::EOF,
        line: lexer.line,
        startcol: lexer.col,
        endcol: lexer.col+1,
    });

    return (tokens.len(), tokens);
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