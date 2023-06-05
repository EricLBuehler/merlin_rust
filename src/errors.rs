//Pretty error generation

use colored::Colorize;

#[derive(Clone, FromPrimitive)]
pub enum ErrorType {
    UnexpectedToken,
    UnknownKeyword,
    UnexpectedEOF,
    FunctionNotExpression,
    TrailingAtomics,
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", repr_err(self.to_owned()))
    }
}

pub fn repr_err(tp: ErrorType) -> &'static str {
    match tp {
        ErrorType::UnexpectedToken => "Unexpected token: This token is not in an appropriate spot.",
        ErrorType::UnknownKeyword => "Unknown keyword: Keyword was specified that does not exist.",
        ErrorType::UnexpectedEOF => {
            "Unexpected EOF: While parsing, encountered end-of-file (EOF) that is not valid."
        }
        ErrorType::FunctionNotExpression => {
            "Function is not an expression: Functions may not be used as expressions"
        }
        ErrorType::TrailingAtomics => {
            "Trailing atomic tokens are not allowed: Code like: `1a` or `a 1` is not allowed."
        }
    }
}

pub fn raise_error(
    error: &str,
    errtp: ErrorType,
    pos: &crate::parser::Position,
    info: &crate::fileinfo::FileInfo,
) -> ! {
    let header: String = format!("error[E{:0>3}]: {}", errtp as u8 + 1, error);
    let location: String = format!("{}:{}:{}", info.name, pos.line + 1, pos.startcol + 1);
    println!("{}", header.red().bold());
    println!("{}", location.red());
    let lines = Vec::from_iter(info.data.split(|num| *num as char == '\n'));

    let snippet: String = format!(
        "{}",
        String::from_utf8(
            lines
                .get(pos.line)
                .expect("Line index out of range")
                .to_vec()
        )
        .expect("utf8 conversion failed")
        .blue()
    );
    let mut arrows: String = String::new();
    for idx in 0..snippet.len() {
        if idx >= pos.startcol && idx < pos.endcol {
            arrows += "^";
        } else {
            arrows += " ";
        }
    }
    let linestr = (pos.line + 1).to_string().blue().bold();
    println!("{} | {}", linestr, snippet);
    println!("{} | {}", " ".repeat(linestr.len()), arrows.green());
    std::process::exit(1);
}
