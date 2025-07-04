use std::collections::HashMap;
use crate::{Literal, RuntimeError, Stmt, TokenType, Value};
use crate::lexer::Token;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    /// Bindings for *this* scope
    values: HashMap<String, Value>,

    /// Optional parent scope
    pub(crate) enclosing: Option<Box<Environment>>,
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

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Value) -> Result<(), RuntimeError> {
        // Get the correct ancestor environment at the given depth and mutably borrow it
        let ancestor = self.ancestor_mut(distance);
        ancestor.values.insert(name.lexeme.clone(), value); // Insert at the correct environment
        Ok(())
    }

    pub fn ancestor_mut(&mut self, distance: usize) -> &mut Environment {
        let mut environment = self;
        for _ in 0..distance {
            match &mut environment.enclosing {
                Some(parent) => environment = parent,
                None => panic!("Ancestor not found, should not happen"),
            }
        }
        environment
    }

    pub fn ancestor(&self, distance: usize) -> &Environment {
        let mut environment = self;
        for _ in 0..distance {
            match &environment.enclosing {
                Some(parent) => environment = parent,
                None => panic!("Ancestor not found, should not happen"),
            }
        }
        environment
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<Value, RuntimeError> {
        let ancestor = self.ancestor(distance);
        ancestor.values.get(name).cloned().ok_or_else(|| {
            let dummy_token = Token {
                token_type: TokenType::LeftParen,
                lexeme: name.to_string(),
                literal: Literal::Nil,
                line: 0, // default line number, could be adjusted
            };
            RuntimeError::new(dummy_token, format!("Undefined variable '{}'.", name))
        })
    }
}
