mod fileinfo;
use fileinfo::FileInfo;

mod lexer;

mod parser;

mod errors;

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

    let ast = parser::new(lexer, &file_info).generate_ast();
    println!("{:?}", ast);
}