use std::{env, process};
use rlox::tree_walk;
use rlox::bytecode;
use std::{fs, io, result::Result};
use io::{BufRead, Error as IOError, Write};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.len() {
        0 => {
            if let Err(error) = run_prompt() {
                eprintln!("{}", error);
            }
        }
        1 => {
            let path = args[0].to_string();
            let content = fs::read_to_string(path).expect("File not found");
            run_interpreter(content);
        }
        _ => {
            println!("Usage: rlox [script]");
            process::exit(64);
        }
    }
}

fn run_prompt() -> Result<(), IOError> {
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

fn run_interpreter(script: String) {
    bytecode::run_interpreter(script);
}
