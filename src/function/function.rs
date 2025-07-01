use std::fmt;
use std::rc::Rc;

use crate::environment::Environment;
use crate::evaluator::{Evaluator, RuntimeError};
use crate::evaluator::{Value, LoxCallable};
use crate::parser::Stmt;


/*
Parameters are core to functions, especially the fact that a function
encapsulates its parameters - no other code outside the function can see them.
This means each function gets its own environment where it stores its variables.

Further, this environment must be created dynamically. Each function call gets its own environment.
Otherwise, recursion would break. If there are multiple calls to the same function in play at the same time,
each needs their own environment, even though they call to the same function.

If we pause the interpreter right at the point where it is about to print 1 in the innermost
nested function call, and the outer function calls the function to print 2 and 3, there must be
environments somehwere in memory that still store the fact thatn is bound to 3 in one context,
2 in another, and 1 in the innermost context.

That is why we create a new environment at each call, not at the function declaration. The `call()` method
we saw earlier does that. At the beginning of the call, it creates a new environment. It walks through
the parameter and arguments lists in lockstep. For each pair, it creates a new key-value pair and
adds it to the current environment.

Then `call()` ells the interpreter to execute the body of the function in this new
function-local environment. Up until now, the current environment was the environment
where the function was being called. Now, we teleport from there inside the new parameter space we have created
for the function.

This is all that’s required to pass data into the function. By using different environments when we execute the body, calls to the same function with the same code can produce different results.

Once the body of the function has finished executing, the environment is restored
to that which was previously active at the callsite. Finally, `call()` returns null, which returns `nil`
to the caller.

When we bind the parameters, we assume the parameter and argument lists have the same length. 
This is safe because `visit_call_expr()` checks the arityu before calling `call()`. It relies
on the function reporting its arity to do that. 
*/

/// A user-defined function object.
#[derive(Debug, Clone)]
pub struct LoxFunction {
    // keep an Rc so multiple closures can share the same declaration
    declaration: Rc<Stmt>,        // must be Stmt::Function
    closure:     Rc<Environment>,
}

impl LoxFunction {
    pub fn new(decl: Stmt, closure: Rc<Environment>) -> Self {
        Self {
            declaration: Rc::new(decl),
            closure,
        }
    }
}

/* ────────────────────── LoxCallable implementation ─────────────────────── */
impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        match &*self.declaration {
            Stmt::Function { params, .. } => params.len(),
            _ => 0,     // should never happen
        }
    }

    fn call(
        &self,
        interpreter: &mut Evaluator,
        mut arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {

        // ① new activation-record that chains to the captured environment
        let closure:Environment = (*self.closure).clone();
        let mut env = Environment::new_enclosed(closure);

        // ② bind parameters exactly as before …
        if let Stmt::Function { params, .. } = &*self.declaration {
            for (tok, arg) in params.iter().zip(arguments.drain(..)) {
                env.define(tok.lexeme.clone(), arg);
            }
        }

        // ③ execute body exactly as before
        if let Stmt::Function { body, .. } = &*self.declaration {
            match interpreter.execute_block(body, env) {
                Ok(())                              => Ok(Value::Nil),
                Err(RuntimeError::Return(v))        => Ok(v.unwrap_or(Value::Nil)),
                Err(e)                              => Err(e),
            }
        } else {
            unreachable!("LoxFunction without Function declaration");
        }
    }
}

/* ───────────────────────── Display helper (optional) ───────────────────── */

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Stmt::Function { name, .. } = &*self.declaration {
            write!(f, "<fn {}>", name.lexeme)
        } else {
            write!(f, "<fn>")
        }
    }
}
