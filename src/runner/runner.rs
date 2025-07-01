use std::borrow::Cow;
use std::{fs, io};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::{Interpreter, Parser, Scanner, Token};

pub static HAD_ERROR: AtomicBool = AtomicBool::new(false);
pub static HAD_RUNTIMES: AtomicBool = AtomicBool::new(false);

pub fn run_file(path: &String) -> () {
    let bytes: Vec<u8> = fs::read(path).expect("Failed to read file");
    let source: Cow<str> = String::from_utf8_lossy(&bytes);
    run(&source.to_string());

    if HAD_ERROR.load(Ordering::Relaxed) {
        std::process::exit(65);
    }

    if HAD_RUNTIMES.load(Ordering::Relaxed) {
        std::process::exit(70);
    }
}

pub fn run_prompt() -> () {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line: String = String::new();
        let bytes_read = stdin.read_line(&mut line).unwrap();

        if bytes_read == 0 {
            break; // EOF or Control-D
        }

        run(&line);
        HAD_ERROR.store(false, Ordering::Relaxed);
    }
}

fn run(source: &String) -> () {
    let mut scanner: Scanner = Scanner::new(source.to_string());
    let tokens: &Vec<Token> = scanner.scan_tokens();

    let mut parser = Parser::new(tokens.clone());
    let statements = parser.parse();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(statements);
}