use std::collections::HashMap;
use std::rc::Rc;
use crate::evaluator::{Evaluator};
use crate::{runtime_error, ClockFn, Environment, Expr, Resolver, RuntimeError, Stmt, Token, Value};
pub struct Interpreter {
    globals: Environment,
    env:     Environment,   // current (can start equal to globals)
    locals: HashMap<Expr, usize>,
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
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        let mut resolver = Resolver::new(self); // Pass `self` as a mutable reference
        resolver.resolve_stmt(&statements); // resolve the statements (loop internally)

        let mut evaluator = Evaluator::new(self.env.clone());

        // Execute each statement
        for stmt in statements {
            if let Err(err) = evaluator.execute(&stmt) {
                runtime_error(err);
                break;
            }
        }

        // Keep `self.env` in sync in case the program created globals
        self.env = evaluator.environment;
    }


    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        // This will store how deep each variable is in the environment
        // The depth here represents how many scopes away the variable is from the current one
        self.locals.insert(expr.clone(), depth);
    }

    pub fn lookup_variable(&mut self, name: Token, expr: Expr) -> Result<Value, RuntimeError> {
        // Check if the variable is local by looking it up in the `locals` map
        if let Some(&distance) = self.locals.get(&expr) {
            // If found in the locals, use `get_at` to access it from the correct environment
            return self.env.get_at(distance, &name.lexeme);
        }

        // If not found locally, look for it in the global environment
        self.globals.get(&name)
    }

}
