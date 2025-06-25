use crate::evaluator::{Evaluator};
use crate::{runtime_error, Stmt};

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
    pub fn new() -> Self {
        Self 
    }
    
    pub fn interpret(&self, statements: Vec<Stmt>) {
        let mut evaluator = Evaluator::new();

        for stmt in statements {
            if let Err(err) = evaluator.execute(&stmt) {
                runtime_error(err);
                break;
            }
        }
    }


}

