use crate::evaluator::{Evaluator, RuntimeError, Value};
use crate::parser::Expr;

pub struct Interpreter;

/*
Some languages are statically typed, meaning that type errors are detected and reported
at compile time. Other languages are dynamically typed, meaning that type checking is deferred
up until an operation is to be performed during runtime.

Statically typed languages are good because you have confidence that your program
will never have a type error. The more type checks you defer to runtime, the more this
confidence erodes.
*/

impl Interpreter {
    pub fn interpret(expression: &Expr) {
        let mut evaluator = Evaluator::new();
        match evaluator.evaluate(expression) {
            Ok(value) => println!("{}", Self::stringify(&value)),
            Err(err) => Self::runtime_error(err),
        }
    }

    fn stringify(value: &Value) -> String {
        match value {
            Value::Nil => "nil".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
        }
    }

    fn runtime_error(err: RuntimeError) {
        eprintln!("[line {}] RuntimeError: {}", err.token.line, err.message);
    }
}
