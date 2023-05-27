use clap::Parser;
use std::sync::Arc;
use std::time::{SystemTime};

mod fileinfo;

use fileinfo::FileInfo;
mod lexer;

mod parser;

mod errors;

mod objects;

mod compiler;

mod interpreter;


fn run_file(filename: &String) {
    let res = std::fs::read_to_string(filename);
    let file_data = match res {
        Ok(_) => {
            res.unwrap()
        }
        Err(_) => {
            println!("File '{}' is unable to be opened or read.", filename);
            return;
        }
    };

    run_data(file_data, filename.clone());
}

fn run_data(file_data: String, name: String) {
    let file_data_bytes = file_data.as_bytes();

    let file_info = FileInfo {
        data: file_data_bytes,
        name,
    };

    let keywords = vec![String::from("fn"), String::from("return")];
    let lexer = lexer::new(file_data_bytes, &file_info, keywords);

    if cfg!(debug_assertions) {
        lexer::print_tokens(lexer.to_owned());
    }


    if cfg!(debug_assertions) { println!("\n===== Running parser ====="); }
    let ast = parser::new(lexer, &file_info).generate_ast();
    if cfg!(debug_assertions) { println!("===== Done with parsing ====="); }

    let vm = Arc::new(interpreter::VM::new());
    objects::init_types(vm.clone());

    if cfg!(debug_assertions) { println!("\n===== Running compiler ====="); }

    let mut compiler = compiler::Compiler::new(&file_info, vm.clone());
    let bytecode = compiler.generate_bytecode(&ast);

    if cfg!(debug_assertions) {
        println!("{:?}", &bytecode.instructions);
        for c in &bytecode.consts {
            println!("{}", objects::utils::object_repr(c));
        }
        println!("===== Done with compiler =====");
    }

    if cfg!(debug_assertions) { println!("\n===== Running interpreter ====="); }

    let mut start: u128 = 0;
    if cfg!(debug_assertions) {
        let dur_start = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        start = dur_start.as_micros(); // u128    
    }
    vm.execute(bytecode);
    if cfg!(debug_assertions) {
        let dur_end = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let end = dur_end.as_micros(); // u128  
        println!("{} us", end-start);
        println!("{} ms", ((end-start) as f64)/1000.0);
    }
    if cfg!(debug_assertions) { println!("\n===== Done with interpreter ====="); }
}


//Version: major.minor
#[derive(Parser, Debug)]
#[command(author, version = "1.1", about, long_about = None)]
struct Args {
    /// File to execute
    #[arg(required = true, name = "file")]
    file: String,
}

fn main() {
    let args = Args::parse();

    run_file(&args.file);
}

#[cfg(test)]
mod tests {
    use crate::run_file;

    #[test]
    fn test_literals() {
        run_file(&String::from("tests/literals.me"));
    }

    #[test]
    fn test_operators() {
        run_file(&String::from("tests/operators.me"));
    }

    #[test]
    fn test_functions() {
        run_file(&String::from("tests/functions.me"));
    }
}