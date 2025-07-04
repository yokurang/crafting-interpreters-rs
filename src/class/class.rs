use std::collections::HashMap;
use std::rc::Rc;
use crate::{Environment, Evaluator, LoxCallable, LoxFunction, RuntimeError, Stmt, Token, Value};

#[derive(Clone, Debug)]
pub struct LoxClass {
    superclass: Option<Box<LoxClass>>,
    name: String,
    methods: HashMap<String, LoxFunction>,

}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>, superclass: Option<Box<LoxClass>>) -> Self {
        Self { name, methods, superclass}
    }
    
    pub fn stringify(&self) -> String {
        self.name.clone()
    }

    pub fn find_method(&self, name: String) -> Option<LoxFunction> {
        // First, try to find the method in the current class's methods
        if let Some(method) = self.methods.get(&name) {
            return Some(method.clone());
        }

        // If no method found, check if there's a superclass and try to find the method there
        if let Some(ref superclass) = self.superclass {
            return superclass.find_method(name);
        }

        // If the method isn't found in the current class or its superclass, return None
        None
    }

    pub fn get_method(&self, name: &str) -> Option<&LoxFunction> {
        self.methods.get(name)
    }
}

/*
Since we bind the `init` method before we call it, it has to access to `this` in the body.
This, along with the arguments passed to the class, is all you need to set up the new instance
however you like.
*/

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        // If there is an initializer, that method's arity determines how many arguments
        // to pass when the class is called
        let initializer: Option<LoxFunction> = self.find_method("init".parse().unwrap());
        match initializer {
            Some(init) => {
                init.arity()
            }
            None => {
                // a class does not require an initializer. If not available, then arity is 0
                0
            }
        }
    }

    /*
    When you `call` a class, it instantiates a new LoxInstance for that class and returns it. The arity
    method is how the interpreter validates that you passed the right number of arguments to a callable. For now
    we will say you cannot pass any. When we get to user-defined constructors, we will revisit this.
    */
    fn call(
        &self,
        interpreter: &mut Evaluator,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        /*
        When a class is called, after the LoxInstance is created, we look for an "init" method. If we find oine,
        we immediately bind and invoke it like a normal method call. The argument list is fowarded along.
        */
        let instance = LoxInstance::new(self.clone());

        // Look for the "init" method of the class and call it if it exists
        if let Some(init_method) = self.find_method("init".parse().unwrap()) {
            // Bind the init method to the instance and call it
            init_method
                .bind(instance.clone())
                .call(interpreter, arguments)?;
        }

        // Return the initialized instance
        Ok(Value::LoxInstance(instance))
    }
}


/*
First,w e evaluate the expression of the property we are trying to access. In Lox, only instances have properties.
If the object is some other type, we throw a runtime error.

If the object is a LoxInstance, we ask it to look up the property. A lox instance will manage
a state called fields. Each key in the map is the name of the property, and the value is the value of the property.
Using a hashmap is fast enough for most cases, but more advanced techniques involve `hidden classes` which use advanced caching techniques.

Fields are the bits of state stored in a class instance. Properties are the expressions a get expression can return of a class instance.
Every field is a property, but not every property is a field.

Setters are expression which set the value of an object. 
They appear on the left side of an assignment.

Setters do not chain. However, the reference to call allows any high-precedence expression
before the last dot, including any number of getters.


*/

#[derive(Debug, Clone)]
pub struct LoxInstance {
    klass: LoxClass,
    fields: HashMap<String, Value>, // Stores properties of the instance
}

impl LoxInstance {
    pub fn new(klass: LoxClass) -> Self {
        LoxInstance {
            klass,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone()); // Return the value of the property
        }

        // If the property is a method, bind it to the current instance (this)
        if let Some(method) = self.klass.find_method(name.lexeme.clone()) {
            return Ok(Value::Callable(Rc::new(method.bind(self.clone())))); // Bind the method
        }

        // If the property doesn't exist, throw a runtime error
        Err(RuntimeError::new(
            name.clone(),
            format!("Undefined property '{}'.", name.lexeme),
        ))
    }
    
    pub fn set(&mut self, name: &Token, value: &Value) {
        self.fields.insert(name.clone().lexeme, value.clone());
    }
    
    pub fn stringify(&self) -> String {
        format!("{} instance", self.klass.stringify())
    }
}