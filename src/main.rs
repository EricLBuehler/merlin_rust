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


fn run_file(file: &String, time: Option<i32>) {
    let res = std::fs::read_to_string(file);
    let file_data = match res {
        Ok(v) => {
            v
        }
        Err(_) => {
            println!("File '{}' is unable to be opened or read.", file);
            return;
        }
    };

    run_data(file_data, file.clone(), time);
}

fn run_data(file_data: String, name: String, time: Option<i32>) {
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

    let mut compiler = compiler::Compiler::new(&file_info, vm.clone(), compiler::CompilerScope::Global);
    let bytecode = compiler.generate_bytecode(&ast);

    if cfg!(debug_assertions) {
        println!("{:?}", &bytecode.instructions);
        for c in &bytecode.consts {
            println!("{}", objects::utils::object_repr(c));
        }
        println!("===== Done with compiler =====");
    }

    if cfg!(debug_assertions) { println!("\n===== Running interpreter ====="); }


    if let Some(n_exec) = time {
        let mut sum = 0;
        for _ in 0..n_exec {
            let start = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Unexpected None").as_micros();
            vm.clone().execute(bytecode.clone());
            let end = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Unexpected None").as_micros();
            sum += end-start;
        }
        println!("Average execution time: {} µs.", sum / n_exec as u128);
        println!("Average execution time: {} ms.", (sum as f64 / n_exec as f64) / 1000.0);
    }
    else {
        vm.execute(bytecode);
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

    /// Run the code n times to get the average execution time
    #[arg(long, short, name = "time", default_value_t = 0)]
    time: i32
}

fn main() {
    let args = Args::parse();

    let time = match args.time {
        0 => {
            None
        }
        v=> {
            Some(v)
        }
    };

    run_file(&args.file, time);
}

#[cfg(test)]
mod tests {
    use crate::run_file;

    #[test]
    fn test_literals() {
        run_file(&String::from("tests/literals.me"), None);
    }

    #[test]
    fn test_operators() {
        run_file(&String::from("tests/operators.me"), None);
    }

    #[test]
    fn test_functions() {
        run_file(&String::from("tests/functions.me"), None);
    }
}