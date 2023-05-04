use colored::Colorize;

#[derive(Clone)]
pub enum ErrorType {
    UnexpectedToken,
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", repr_err(self.to_owned()))
    }
}

pub fn repr_err(tp: ErrorType) -> &'static str {
    match tp {
        ErrorType::UnexpectedToken => "unexpected token",
    }
}

pub fn raise_error(error: &str, errtp: ErrorType, pos: &crate::parser::Position, info: &crate::fileinfo::FileInfo) -> !{
    let header: String = format!("error[E{:0>3}]: {}", errtp as u8 + 1, error);
    let location: String = format!("{}:{}:{}", info.name, pos.line+1, pos.startcol+1);
    println!("{}", header.red().bold());
    println!("{}", location.red());
    let lines = Vec::from_iter(info.data.split(|num| *num as char == '\n'));

    let snippet: String = format!("{}", String::from_utf8(lines.get(pos.line).unwrap().to_vec()).unwrap().blue());
    let mut arrows: String = String::new();
    for idx in 0..snippet.len() {
        if (idx as usize)>=pos.startcol && (idx as usize)<pos.endcol {
            arrows += "^";
        }
        else {
            arrows += " ";
        }
    }
    let linestr = (pos.line+1).to_string().blue().bold();
    println!("{} | {}", linestr, snippet);
    println!("{} | {}", " ".repeat(linestr.len()), arrows.green());
    std::process::exit(1);
}