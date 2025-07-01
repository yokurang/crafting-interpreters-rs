use std::rc::Rc;
use crate::evaluator::{Evaluator};
use crate::{runtime_error, ClockFn, Environment, Stmt, Value};
pub struct Interpreter {
    globals: Environment,
    env:     Environment,   // current (can start equal to globals)
}

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
        let mut globals = Environment::new_global();

        // clock() is available everywhere
        globals.define(
            "clock".to_string(),
            Value::Callable(Rc::new(ClockFn)),
        );

        // start with the global env as “current”
        Self {
            env: globals.clone(),
            globals,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        let mut evaluator = Evaluator::new(self.env.clone());

        for stmt in statements {
            if let Err(err) = evaluator.execute(&stmt) {
                runtime_error(err);
                break;
            }
        }
        // keep `self.env` in sync in case the program created globals
        self.env = evaluator.environment;
    }
}
