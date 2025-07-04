use crate::interpreter::Interpreter; // Assuming Interpreter is the same as Evaluator
use crate::parser::{parser, Expr, ParseError, Visitor}; // Importing the Expr and Stmt enums
use crate::lexer::{Literal};
use crate::{error, Stmt, StmtVisitor, Token, Value};
use crate::RuntimeError;
/*
Since the resolver needs to visit every node in the syntax tree, it implements
the visitor abstraction we already have in place. Only a few kinds of nodes are interesting
when it comes to resolving variables.

- A block statement introduces a new scope for the statement it contains
- A function declaration introduces a new scope for its body and binds its parameters to that scope
- A variable declaration adds a new variable to the current scope
- A variable and assignment expression need to have their variable resolved

The rest of the nodes do not do anything special. However, we still need to
implement visit methods to traverse into their subtrees. Even though a + operator does not have any variables to resolve, one of its operands might.

Lexical scopes are implemented via a stack of hashmaps. They are nested in the interpreter and the resolver. They behave like a stack. The interpreter implements that stack using a linked list - the chain of environments. In the resolver, it is implemented using a stack.

This field keeps track of the stack of scopes currently, uh, in scope. Each element in the stack is a Map representing a single block scope. Keys, as in Environment, are variable names. However, the values here are booleans.

The scope stack is only used for local block scopes. Variables declared at the top level in the global scope are not tracked by the resolver since they are more dynamic. When resolving a variable, if we cannot find it in the stack of local scopes, we assume it must be global.

Since scopes are stored explicitly in a stack, when ending a scope, we pop that environment from the stack. This represents exiting a scope.

What happens when the initializer for a local variable refers to a variable with
the same name as the variable being declared?

1. Run the initializer, then put the new variable in scope. Here, the new local would be initialised with other, the value of the global variable.
2. Put the new variable in scope, then run the initializer. This means you could observe a variable before initialized, so we would need to figure out
what value it would have then. Probably nil. That means the new local a would be re-initialised to its own implicitly initialied value, nil.
3. Make it a error to reference a variable in its initializer. Have the interpreter fail either at compile time or runtime if an initializer mentiones the variable being initialized.

Do either of those first two options look like something a user actually wants? Shadowing is rare and often an error, so initializing a shadowing variable based on the value of the shadowed one seems unlikely to be deliberate.

The second option is even less useful. The new variable will always have the value nil. There is never any point in mentioning it by name. You could use an explicit nil instead.

Since the first two options are likely to mask user errors, we’ll take the third. Further, we’ll make it a compiler error instead of a runtime one. That way, the user is alerted to the problem before any code is run.

In order to do that, as we visit expressions, we need to know if we’re inside the initializer for some variable. We do that by splitting binding into two steps. The first is declaring it.

This looks, for good reason, a lot like the code in environment for evaluating
We start at the innermost scope and work outwards, looking in each map for a matching game. If we find the variable, we resolve it, passing in the number of scopes between the current innermost scope and the scope where the variable was found. So, if the variable was found in the current scope, we pass in zero. If it is in the immediately enclosing scope, 1.

If we walk through all of the block scopes and never find the variable, we leave it unresolved and assume it is global. We will get to the implementation of that resolve() later.
*/

use std::collections::HashMap;
use crate::FunctionType::Initializer;
use crate::Value::Nil;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,  // Interpreter is passed as a mutable reference
    scopes: Vec<HashMap<String, bool>>, // Stack of scopes
    current_function: FunctionType,
    current_class: ClassType,
}

#[derive(Debug, PartialEq)]
pub enum ClassType {
    None,
    Class,
    Subclass,
}

#[derive(Debug, PartialEq)]
pub enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    // the resolve statements apply the visitor pattern to the appropriate stmt syntax tree node
    pub fn resolve_stmt(&mut self, statements: &Vec<Stmt>) {
        for stmt in statements {
            self.resolve_stmt_single(stmt); // resolve each statement
        }
    }

    fn resolve_stmt_single(&mut self, stmt: &Stmt) {
        stmt.accept(self).expect("TODO: panic message");  // Visit the statement to resolve it
    }

    /*
    A declaration adds the variable to the innermost scope so that the variable shadows any other variables with the same name in outer scopes. We mark it as not ready yet by binding its name to false in the scope map. The value associated with a key in the scope map represents whether or not we have finished resolving that variable's initializer.

    After declaring the variable, we resolve its initializer expression in that same scope where the new variable now exists but is unavailable. Once the initializer expression is done, the variable is ready. We do this by defining it.

    We set the variable's value in the scope map to true to mark it as fully initialized and ready for use.
    */
    fn declare(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), false);
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        // Traverse the scopes stack from innermost to outermost
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                // Let the interpreter know how deep the variable is in the scope
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    // the resolve function applies the correct visitor pattern based on the expr syntax tree node
    fn resolve_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        expr.accept(self)
    }

    fn resolve_function(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
        declaration: FunctionType,
    ) {
        self.begin_scope();
        // Declare parameters as local variables inside the function
        for param in params {
            self.declare(&param.lexeme);
            self.define(&param.lexeme);
        }

        // Resolve the body of the function
        self.resolve_stmt(body);

        self.end_scope();
    }
}

// Implementing StmtVisitor for Resolver
impl<'a> StmtVisitor<Result<(), RuntimeError>> for Resolver<'a> {
    fn visit_expression_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        if let Stmt::Expression { expression } = stmt {
            self.resolve_expr(expression)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        if let Stmt::Print { expression } = stmt {
            self.resolve_expr(expression)?;
        }
        Ok(())
    }

    // Resolving a variable declaration adds a new entry to the current innermost scope's map. We split the binding into two steps: Declaration and definition.

    fn visit_var_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        if let Stmt::Var { name, initializer, .. } = stmt {
            self.declare(&name.lexeme);  // Declare the variable
            if let Some(init) = initializer {
                self.resolve_expr(init)?; // Resolve initializer expression
            }
            self.define(&name.lexeme);  // Define the variable
        }
        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<(), RuntimeError> {
        self.begin_scope();
        for stmt in statements {
            self.resolve_stmt_single(stmt);
        }
        self.end_scope();
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<(), RuntimeError> {
        self.resolve_expr(condition)?;
        self.resolve_stmt_single(then_branch);

        if let Some(else_stmt) = else_branch {
            self.resolve_stmt_single(else_stmt);
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<(), RuntimeError> {
        self.resolve_expr(condition)?;
        self.resolve_stmt_single(body);
        Ok(())
    }

    fn visit_fun_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>
    ) -> Result<(), RuntimeError> {
        // Declare and define the function name in the current scope.
        self.declare(&name.lexeme);
        self.define(&name.lexeme);

        // Begin a new scope for the function body.
        self.begin_scope();

        // Declare and define each function parameter in the new scope.
        for param in params {
            self.declare(&param.lexeme);
            self.define(&param.lexeme);
        }

        // Resolve the statements (body) of the function in the new scope.
        self.resolve_stmt(body);

        // End the function's scope.
        self.end_scope();

        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Option<Box<Expr>>) -> Result<(), RuntimeError> {
        if let Some(v) = value {
            if self.current_function == FunctionType::Initializer {
                error(keyword.line, "Can't return a value from an initializer.")
            }
            self.resolve_expr(v)?;
        }
        Ok(())
    }


    /* declaring a class as a local variable here
    If the class declaration has a superclass, we create a new scope surrounding all of its methods. In that scope, we define the name "super". Once we are done resolving that class's methods, we discard that scope.
    */
    fn visit_class_stmt(
        &mut self,
        name: &Token,
        methods: &Vec<Result<Stmt, ParseError>>,
        superclass: &Option<Box<Expr>>
    ) -> Result<(), RuntimeError> {
        /*
        We store the previous value of the field in a local variable.
        */
        let enclosing_class = &self.current_class;
        self.current_class = ClassType::Class;
        // Declare the class in the current scope
        self.declare(&name.lexeme);
        self.define(&name.lexeme);

        if let Some(superclass_expr) = superclass {
            // Ensure that a class can't inherit from itself
            if let Expr::Variable { name: superclass_name, .. } = &**superclass_expr {
                if name.lexeme == superclass_name.lexeme {
                    return Err(RuntimeError::new(
                        superclass_name.clone(),
                        "A class cannot inherit from itself.".to_string(),
                    ));
                }
            }
            // Resolve the superclass expression
            self.resolve_expr(superclass_expr)?;
        }

        if let Some(superclass) = superclass {
            self.current_class = ClassType::Subclass;
            self.resolve_expr(superclass).expect("TODO: panic message");
        }

        if let Some(superclass) = superclass {
            self.begin_scope();  // Start a new scope
            self.scopes
                .last_mut()  // Access the current scope (mutably)
                .expect("No scope found.")  // Ensure the scope exists
                .insert("super".to_string(), true);  // Insert "super" in the scope
        }

        // Create a new environment for the class and push a new scope for "this"
        self.begin_scope();
        self.scopes.last_mut().unwrap().insert("this".to_string(), true);

        // Resolve methods inside the class
        for method in methods {
            if let Ok(Stmt::Function { name, params, body }) = method {
                let mut declaration = FunctionType::Method;
                // Resolve the method (similar to the visitFunctionStmt method)
                if name.lexeme.eq("init") {
                    declaration = FunctionType::Initializer;
                }
                self.resolve_function(&name, &params, &body, declaration);
            }
        }

        // End the scope for the "this" reference
        self.end_scope();
        if superclass.is_some() {
            self.end_scope();  // End the scope created for "super"
        }
        self.current_class = ClassType::None;

        Ok(())
    }
}

impl<'a> Visitor for Resolver<'a> {
    fn visit_literal_expr(&mut self, _value: &Literal) -> Result<Value, RuntimeError> {
        // No variables to resolve for literals
        Ok(Value::Nil)
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        self.resolve_expr(expr)
    }

    fn visit_unary_expr(&mut self, _operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
        self.resolve_expr(right)
    }

    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        _operator: &Token,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_variable_expr(&mut self, token: &Token, initializer: &Option<Box<Expr>>) -> Result<Value, RuntimeError> {
        // If we're referencing a variable in its own initializer, throw an error
        if self.scopes.last().unwrap().get(&token.lexeme).map_or(false, |&v| !v) {
            return Err(RuntimeError::new(
                token.clone(),
                format!("Can't read local variable in its own initializer."),
            ));
        }

        // Check if it's declared and resolved
        if let Some(init) = initializer {
            self.resolve_local(init, token);
        }

        // If it has an initializer, resolve that as well
        if let Some(init_expr) = initializer {
            self.resolve_expr(&init_expr)?;  // Resolve the expression inside the initializer
        }

        Ok(Value::Nil)  // Not necessary to return a value here, it's for the resolution
    }

    // we resolve the expression for the assigned value in case it also contains references to other variables. Then we use our existing resolve local method top resolve the variable that's being assigned to
    fn visit_assign_expr(&mut self, token: &Token, value: &Expr) -> Result<Value, RuntimeError> {
        // Resolve the value that the variable is being assigned
        self.resolve_expr(value)?;

        // Resolve the variable being assigned to
        self.resolve_local(value, token);

        Ok(Value::Nil)  // Not necessary to return a value here either
    }

    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        _operator: &Token,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        _paren: &Token,
        arguments: &[Expr],
    ) -> Result<Value, RuntimeError> {
        self.resolve_expr(callee)?;
        for arg in arguments {
            self.resolve_expr(arg)?;
        }
        Ok(Value::Nil)
    }


    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<Value, RuntimeError> {
        // since properties are looked up dynamically, they do not need to get resolved
        // During resolution, we recurse only into the expression to the left of the dot. The actual property access happens in the interpreter.
        self.resolve_expr(object)
    }

    fn visit_set_expr(&mut self, object: &Expr, name: &Token, value: &Expr) -> Result<Value, RuntimeError> {
        self.resolve_expr(value).expect("TODO: panic message");
        self.resolve_expr(object)
    }

    fn visit_this_expr(&mut self, this: &Token) -> Result<Value, RuntimeError> {
        if self.current_class == ClassType::None {
            error(this.line,"Can't use 'this' outside of a class.")
        }
        self.resolve_local(&Expr::This { keyword: this.clone() }, this);
        Ok(Nil)
    }

    /*
    It is a minor optimization, but we only create the superclass environment if the class actually has a superclass. There is no point in creating it when there is not a superclass since there would be no superclass to store in it anyway.
    */
    fn visit_super_expr(&mut self, keyword: &Token, method: &Token) -> Result<Value, RuntimeError> {
        let dummy_expr = Expr::Literal {
            value: Literal::Nil, // You can use any placeholder value here
        };
        
        if self.current_class == ClassType::None { 
            error(keyword.line, "Can't use 'super' outside of a class.")
        } else if self.current_class != ClassType::Subclass {
            error(keyword.line, "Can't use 'super' in a class with no superclass.")
        }

        // Resolve the "super" expression
        self.resolve_local(&dummy_expr, keyword);
        Ok(Value::Nil)
    }
}