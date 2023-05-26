mod fileinfo;
 
use std::sync::Arc;

use fileinfo::FileInfo;
mod lexer;

mod parser;

mod errors;

mod objects;

mod compiler;

mod interpreter;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    debug_assert!(args.len()==2);

    let filename = &args[1];    
    
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

    let file_data_bytes = file_data.as_bytes();

    let file_info = FileInfo {
        data: file_data_bytes,
        name: filename.to_owned(),
    };

    let keywords = vec![String::from("fn")];
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

    vm.execute(bytecode);
    if cfg!(debug_assertions) { println!("\n===== Done with interpreter ====="); }
}