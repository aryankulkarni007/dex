use std::env;
use std::fs;

mod ast;
mod lexer;
mod parser;
mod token;

use lexer::Lexer;
use parser::Parser;
#[allow(unused_imports)]
use token::Token;

fn read_file(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!("Usage: dex <file_path>");
            std::process::exit(1);
        }
    };

    match read_file(file_path) {
        Ok(content) => {
            println!("File read successfully");
            let mut lexer = Lexer::new(content);
            let tokens = lexer.tokenize();
            // for token in &tokens {
            //     println!("{:?} {:?}", token.kind, token.value);
            // }
            // println!("{:#?}", tokens);
            let mut parser = Parser::new(tokens);
            let decls = parser.parse();
            println!("{:#?}", decls);
        }
        Err(e) => {
            eprintln!("Error reading '{}': {}", file_path, e);
            std::process::exit(1);
        }
    }
}
