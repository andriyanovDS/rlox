use io::{BufRead, Error, Write};
use scanner::Scanner;
use std::{fs, io, result::Result};
mod scanner;
mod token;
mod token_type;
mod expression;

pub fn run_prompt() -> Result<(), Error> {
    print!("> ");
    io::stdout().flush().unwrap();

    for read_result in io::stdin().lock().lines() {
        let line = read_result?;
        run_interpreter(line);

        print!("> ");
        io::stdout().flush().unwrap();
    }
    Ok(())
}

pub fn run_file(path: String) {
    let content = fs::read_to_string(path).expect("File not found");
    run_interpreter(content);
}

fn run_interpreter(script: String) {
    let mut scanner = Scanner::new(script.as_str());
    let tokens = scanner.scan_tokens();
    for token in tokens {
        println!("token: {:?}", token);
    }
}
