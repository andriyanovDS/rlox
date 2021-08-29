use rlox;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.len() {
        0 => {
            let result = rlox::run_prompt();
            if let Err(error) = result {
                eprintln!("{}", error);
            }
        }
        1 => {
            rlox::run_file(args[0].to_string());
        }
        _ => {
            println!("Usage: rlox [script]");
            process::exit(64);
        }
    }
}
