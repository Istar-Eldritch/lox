mod ast;
mod interpreter;
mod lexer;
mod parser;

use clap::Clap;

use crate::lexer::TokenKind;
use interpreter::Interpretable;

#[derive(Clap, Debug)]
#[clap(name = "lox")]
struct Input {
    file_path: Option<String>,
}

fn main() {
    let input = Input::parse();
    if let Some(path) = input.file_path {
        run_file(path).unwrap();
    } else {
        repl()
    }
}

fn run_file(file_path: String) -> Result<(), ()> {
    let code = std::fs::read_to_string(file_path).expect("Error reading file");
    let tokens = lexer::tokenize(&code);
    println!("{:?}", tokens.collect::<Vec<lexer::Token>>());
    Ok(())
}

fn repl() {
    let stdin = std::io::stdin();
    println!("Running repl");
    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).expect("Error reading input");

        execute(&mut buffer).unwrap();
    }
}

fn execute(code: &mut str) -> Result<(), ()> {
    let mut tokens = lexer::tokenize(code)
        .filter(|t| t.kind != TokenKind::Whitespace)
        .peekable();
    let ast = parser::parse(&mut tokens).map_err(|e| {
        println!("{}", e);
    })?;
    println!("{:?}", ast.eval());
    Ok(())
}
