use std::env;
use crafting_interpreters::runner::{run_file, run_prompt};

pub fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // args always includes the program name in args[0]
    match args.len() {
        1 => {
            run_prompt();
        }
        2 => {
            run_file(&args[1]);
        }
        _ => {
            println!("Usage: jlox [script]");
            std::process::exit(64);
        }
    }
    Ok(())
}
