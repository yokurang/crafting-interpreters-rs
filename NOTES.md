## Notes

### Statements and State

Programming is to build semantic meaning from atomic pieces of the language. 
Having an interpreter that only supports evaluating numerical expressions is hardly programming. To expand on the current 
interpreter, we need to support binding expression values to names and remembering these names throughout the computation. This is how we can compose software.

To support bindings, the interpreter needs an internal state. When you define a
variable at the beginning of the program and refer to it at the end of the program, the interpreter
needs to store the variable name and the associated value in the meantime.

Statements by definition do not evaluate to a value. When they do not, statements produce a side effect. Defining variables or other named entities
is considered a side effect.

#### Statements

- An expression statement lets you place an expression where a statement is expected. they exist to evaluate expressions that have side effects. 

- A print statement evaluates an expression and displays the result to the user. 

```
program        → statement* EOF ;

statement      → exprStmt
               | printStmt ;

exprStmt       → expression ";" ;
printStmt      → "print" expression ";" ;
```

Side note: An End of File (EOF) token is a type of token which indicates that we have reached the end of a file. This is important for scanners and parsers to ensure they do not secretly
ignore any input.

Observing this grammar, we can see that there is no place for both an expression and statement are allowed.

Since the syntaxes are distinct, there is no need to implement them inheriting the same class. We can implement a statement as its own class to avoid potential mistakes where we pass a statement to a method expecting an expression.

