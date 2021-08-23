mod ast;
mod interpreter;
mod lexer;
mod parser;

use std::{
    cell::RefCell,
    io::{stderr, Write},
    rc::Rc,
};

use clap::Clap;

use crate::lexer::TokenKind;
use interpreter::{Environment, Interpretable};

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
    let env = Rc::new(RefCell::new(Environment::new()));

    execute(&mut code, env).unwrap_or_else(|e| {
        let stde = stderr();
        let mut stdew = stde.lock();
        stdew.write_all(&format!("{}\n", e).into_bytes()).unwrap();
    });
    Ok(())
}

fn repl() {
    let stdin = std::io::stdin();
    println!("Running repl");
    let env = Rc::new(RefCell::new(Environment::new()));

    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).expect("Error reading input");
        execute(&mut buffer, env.clone()).unwrap_or_else(|e| {
            let stde = stderr();
            let mut stdew = stde.lock();
            stdew.write_all(&format!("{}\n", e).into_bytes()).unwrap();
        });
    }
}

fn execute(
    code: &mut str,
    env: Rc<RefCell<Environment>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tokens = lexer::tokenize(code)
        .filter(|t| t.kind != TokenKind::Whitespace)
        .peekable();
    let ast = parser::parse(&mut tokens)?;
    for stmt in ast {
        stmt.eval(env.clone())?;
    }
    Ok(())
}
