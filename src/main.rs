mod fileinfo;

#[macro_use]
extern crate lazy_static;
 
use fileinfo::FileInfo;
mod lexer;

mod parser;

mod errors;

mod objects;

mod compiler;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    debug_assert!(args.len()==2);

    let filename = &args[1];
    let file_data;
    
    
    let res = std::fs::read_to_string(filename);
    match res {
        Ok(_) => {
            file_data = res.unwrap();
        }
        Err(_) => {
            println!("File '{}' is unable to be opened or read.", filename);
            return;
        }
    }

    let file_data_bytes = file_data.as_bytes();

    let file_info = FileInfo {
        data: file_data_bytes.clone(),
        name: filename.to_owned(),
    };

    let keywords = vec![];
    let lexer = lexer::new(file_data_bytes, &file_info, keywords);

    lexer::print_tokens(lexer.to_owned());

    println!("\n===== Running parser =====");
    let ast = parser::new(lexer, &file_info).generate_ast();
    println!("===== Done with parsing =====");

    println!("\n===== Running type tests =====");
    let types = objects::init_types();
    println!("{}", objects::utils::object_repr(&types.get("str").unwrap().clone().get_bases()));
    println!("{}", objects::utils::object_repr(&objects::intobject::IntObject::from(1234567890)));
    println!("===== Done with type tests =====");

    println!("\n===== Running compiler =====");
    let mut compiler = compiler::Compiler::new();
    let bytecode = compiler.generate_bytecode(ast);

    println!("{:?}", bytecode.instructions);
    for c in bytecode.consts {
        println!("{}", objects::utils::object_repr(&c));
    }
    println!("===== Done with compiler =====");
}