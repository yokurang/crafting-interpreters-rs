use std::collections::HashMap;
use crate::{RuntimeError, Stmt, Value};
use crate::lexer::Token;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    /// Bindings for *this* scope
    values: HashMap<String, Value>,

    /// Optional parent scope
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    /// Create the top-level (global) environment.
    pub fn new_global() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    /// Create a nested environment that owns its parent (`Box`).
    pub fn new_enclosed(enclosing: Environment) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        // Insert or shadow without extra checks.
        self.values.insert(name, value);
    }
    
    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(v) = self.values.get(&name.lexeme) {
            return Ok(v.clone());
        }
        if let Some(ref parent) = self.enclosing {
            return parent.get(name); // recursive borrow is fine
        }
        Err(RuntimeError::new(
            name.clone(),
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }
        if let Some(ref mut parent) = self.enclosing {
            return parent.assign(name, value); // recurse mutably
        }
        Err(RuntimeError::new(
            name.clone(),
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }
}
