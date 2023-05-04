mod fileinfo;
use fileinfo::FileInfo;

mod lexer;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    assert!(args.len()==2);

    let filename: &String = &args[1];
    let file_data: String;
    
    
    let res: Result<String, std::io::Error> = std::fs::read_to_string(filename);
    match res {
        Ok(_) => {
            file_data = res.unwrap();
        }
        Err(_) => {
            println!("File '{}' is unable to be opened or read.", filename);
            return;
        }
    }

    let file_data_bytes: &[u8] = file_data.as_bytes();

    let file_info: FileInfo = FileInfo {
        data: file_data_bytes.clone(),
        name: filename.clone(),
    };

    let mut keywords: Vec<String> = vec![];
    let mut lexer: lexer::Lexer = lexer::new(file_data_bytes, &file_info);
    let (_, tokens) = lexer::generate_tokens(&mut lexer, &mut keywords);

    lexer::print_tokens(tokens.len(), &tokens);
}