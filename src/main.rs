use std::{env, process};
use rlox::tree_walk;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.len() {
        0 => {
            let result = tree_walk::run_prompt();
            if let Err(error) = result {
                eprintln!("{}", error);
            }
        }
        1 => {
            tree_walk::run_file(args[0].to_string());
        }
        _ => {
            println!("Usage: rlox [script]");
            process::exit(64);
        }
    }
}
