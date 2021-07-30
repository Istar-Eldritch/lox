mod lexer;

use clap::Clap;

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
    let tokens = lexer::tokenize(code);
    println!("{:?}", tokens.collect::<Vec<lexer::Token>>());
    Ok(())
}
