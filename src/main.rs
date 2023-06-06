use clap::Parser;
use colored::Colorize;
use std::time::Instant;
extern crate num;
#[macro_use]
extern crate num_derive;

mod fileinfo;

use fileinfo::FileInfo;
mod lexer;

mod parser;

mod errors;

#[macro_use]
mod objects;

mod compiler;

mod interpreter;
mod stats;

#[cfg(not(target_has_atomic = "ptr"))]
mod mutexrc;
#[cfg(not(target_has_atomic = "ptr"))]
type Arc = mutexrc::Arc;

#[cfg(target_has_atomic = "ptr")]
use std::sync::Arc;

pub struct TimeitHolder {
    baseline: u128,
    time: f64,
}

fn run_file(file: &String, time: Option<i32>) {
    let res = std::fs::read_to_string(file);
    let file_data = match res {
        Ok(v) => v,
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

    if cfg!(debug_assertions) {
        println!("\n===== Running parser =====");
    }
    let ast = parser::new(lexer, &file_info).generate_ast();
    if cfg!(debug_assertions) {
        println!("===== Done with parsing =====");
    }

    let vm = Arc::new(interpreter::VM::new(file_info.clone()));
    objects::init_types(vm.clone());
    interpreter::VM::init_cache(vm.clone());

    if cfg!(debug_assertions) {
        println!("\n===== Running compiler =====");
    }

    let mut compiler = compiler::Compiler::new(&file_info, vm.clone());
    let bytecode = compiler.generate_bytecode(&ast);

    if cfg!(debug_assertions) {
        println!("{:?}", &bytecode.instructions);
        for c in &bytecode.consts {
            println!(
                "{} = 0x{:x}",
                objects::utils::object_repr(c),
                Arc::as_ptr(c) as u64
            );
        }
        println!("===== Done with compiler =====");
    }

    if cfg!(debug_assertions) {
        println!("\n===== Running interpreter =====");
    }

    if let Some(n_exec) = time {
        let mut min = f64::MAX;
        let mut baseline = u128::MAX;
        for _ in 0..1000 {
            let start = Instant::now();
            let delta = start.elapsed().as_nanos();
            if delta < baseline && delta > 0 {
                baseline = delta;
            }
        }

        let interpreter = interpreter::Interpreter::new(
            vm.clone().types.clone(),
            vm.clone().namespaces.clone(),
            vm.clone().clone(),
        );

        let refr = Arc::into_raw(vm.clone()) as *mut interpreter::VM;

        unsafe {
            (*refr).interpreters.push(Arc::new(interpreter));
            Arc::from_raw(refr);
        }

        let mut means = Vec::new();
        for _ in 0..n_exec {
            let mut holder = TimeitHolder { baseline, time: 0. };
            interpreter::VM::execute_timeit(vm.clone(), bytecode.clone(), &mut holder);
            let time = holder.time;
            if time < min && time >= 0. {
                min = time;
            }
            means.push(time);
        }
        println!("Best execution time: {:.3} ns.", min);
        println!("Best execution time: {:.3} µs.", min / 1000.0);
        println!("Best execution time: {:.3} ms.", min / 1000000.0);

        println!();

        let mean = {
            let mut sum = 0.0;
            for mean in &means {
                sum += mean
            }
            sum
        } / means.len() as f64;
        println!("Mean execution time: {:.3} ns.", mean);
        println!("Mean execution time: {:.3} µs.", mean / 1000.0);
        println!("Mean execution time: {:.3} ms.", mean / 1000000.0);
    } else {
        interpreter::VM::execute(vm, bytecode);
    }
    if cfg!(debug_assertions) {
        println!("\n===== Done with interpreter =====");
    }
}

//Version: major.minor
#[derive(Parser, Debug)]
#[command(author, version = "1.3", about, long_about = None)]
struct Args {
    /// File to execute
    #[arg(required = true, name = "file")]
    file: String,

    /// Run the code n times to get the best execution time (this is the most accurate because all others are worse due to external factor).
    /// No more tests are run if an error occurs.
    #[arg(long, short, name = "time", default_value_t = 0)]
    time: i32,

    /// Explain an error produced by the parser.
    #[arg(long, short, name = "explain", default_value_t = -1)]
    explain: i32,
}

fn main() {
    let args = Args::parse();

    let time = match args.time {
        0 => None,
        v => Some(v),
    };

    if args.explain > 0 {
        let err = num::FromPrimitive::from_i32(args.explain - 1);
        match err {
            Some(tp) => {
                println!(
                    "{}:",
                    format!("error[E{:0>3}]", (args.explain - 1) as u8 + 1)
                        .red()
                        .bold()
                );
                println!("{}", errors::repr_err(tp).green());
                return;
            }
            None => {
                println!(
                    "{}",
                    "Error number does not correspond to a valid error.".red()
                );
                return;
            }
        }
    }

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
