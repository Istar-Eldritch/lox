mod ast;
mod interpreter;
mod lexer;
mod parser;

use std::{
    collections::HashMap,
    io::{stderr, Write},
};

use clap::Clap;

use crate::lexer::TokenKind;
use interpreter::{Interpretable, LoxResult};

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
    let mut code = std::fs::read_to_string(file_path).expect("Error reading file");
    let mut env = HashMap::new();

    execute(&mut code, &mut env).unwrap_or_else(|e| {
        let stde = stderr();
        let mut stdew = stde.lock();
        stdew.write_all(&format!("{}\n", e).into_bytes()).unwrap();
    });
    Ok(())
}

fn repl() {
    let stdin = std::io::stdin();
    println!("Running repl");
    let mut env = HashMap::new();

    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).expect("Error reading input");

        execute(&mut buffer, &mut env).unwrap_or_else(|e| {
            let stde = stderr();
            let mut stdew = stde.lock();
            stdew.write_all(&format!("{}\n", e).into_bytes()).unwrap();
        });
    }
}

fn execute(
    code: &mut str,
    env: &mut HashMap<String, Option<LoxResult>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tokens = lexer::tokenize(code)
        .filter(|t| t.kind != TokenKind::Whitespace)
        .peekable();
    let ast = parser::parse(&mut tokens)?;
    for stmt in ast {
        stmt.eval(env)?;
    }
    Ok(())
}
