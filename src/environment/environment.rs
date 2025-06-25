use std::collections::HashMap;
use std::fmt::format;
use crate::evaluator::{Value};
use crate::lexer::Token;
use crate::evaluator::RuntimeError;

pub struct Environment {
    /* the keys are bare strings, not tokens.
    This is because a token represents a unit of code at a
    specific place in the source text, but when it comes to looking up variables,
    all identifier tokens with the same name should refer to the same variable, and hence
    the same value associated with that variable. Using raw strings ensure that all identifier
    tokens with the same name refer to the same variable in the environment.
    */
    values: HashMap<String, Value>
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new()
        }
    }

    // support adding a new name-value binding to the environment
    pub fn define(&mut self, name: String, value: Value) -> () {
        // notice that we do not need to check if the `name` key already
        // exists in the map before inserting. This aligns with the typical understanding
        // of how variable declarations work

        // side tip: when developing a new language, when in doubt,
        // follow the footsteps of what other people have done
        self.values.insert(name, value);
    }

    // support accessing the value associated with a variable
    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(val) = self.values.get(&name.lexeme) {
            Ok(val.clone())
        } else {
            /* say if we want to use a variable, what should we do?
            The important thing to note is that:
            1. Using a variable is different from referring to a variable. It is possible to refer to a variable in some lines
            of code without evaluating the chunks of code which refer to the invalid variable.
            2. Using a variable means evaluating it at runtime.
            3. If we make it a static error to not refer to variables before they are defined, then we cannot implement recursive functions.
            4. How recursion works: The function name can be referenced to since it is available in the AST / parse time. Furthermore, the function is bound to the function name during runtime, so all works well.
            5. Runtime errors are only thrown if variables are being used when it does not have any initialization during evaluation
            The trick is to allow referring to variables that are not defined as long as they are available in the AST, and only throw a runtime error
            if the variables being evaluated do not have an initialization. 
            Reminder: Declaration is to say that the function exists, definition implements the body of the function. 
            */
            Err(RuntimeError::new(
                name.clone(),
                format!("Undefined variable {}.", name.lexeme)
            ))
        }
    }
}