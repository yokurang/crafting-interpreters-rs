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

#### Global Variables

Before going to lexical scoping, the easiest kinds of variables are global variables. In name binding, there are two new constructs:

1. Variable declarations - these statements binds a name to the value of an expression

2. Variable expressions - these expressions access the value stored in a variable name

Variable declerations are statements, and to embed this into the grammar, we need to distinguish variable declerations as its own 
'kind' of statement. This is to disallow certain kinds of statements like 

```
if (monday) {
    var stuff = "cool"
}
```

Since these kinds of statement declarations do not make sense.

To access the value embedded in a variable, we add a new `IDENTIFIER` clause under the body of the `primary` production rule.

#### Environments

The binding between a variable name and its associated expression value is stored in a data structure called the environment.
Environments are traditionally thought of (and implemented) as a hash map where the keys are the variable names and the values are the
variable's values.
Note that some languages do not let you assign a variable once it has been defined. For example, OCaml or Rust. Assigning a variable after it has been defined is called mutating the variable.

Mutating a variable is a side effect, and some languages view side effects as risky.
Note that for lox, global variables already support reassignment. So, adding support for assignments only require us to define the assignment operator and the relevant implementation details to detect the token and perform the relevant operations when evaluating it.

Note that assignment is an expression, not a statement. It is normally, the assignment operator has the lowest precedence.

The tricky part is that a recursive-descent parser cannot look ahead far enough to tell whether it is parsing
an assignment or not until after it has evaluated the left operand of an AST node where the operator is `=`.
The difference for assignment operators is that the left-hand side of an assignment operator is not an expression that evaluates to a value,
but a pseudo-expression where you can assign expressions.

Consider the following example:

```
var a = "before";
a = "value";
```

On the second line, we do not evaluate `a`, but we figure out what variable `a` refers to so that we can reassign the value of that variable. The classic terms for them is `l-value` and `r-value`.
All the expressions we have seen so far which evaluate to a value are called `r-values`. Expressions which evaluate to a storage location you can assign to are called `l-values`.

We want the syntax tree to reflect that an `l-value` is not evaluated like a normal expression. That's why the `Expr.Assign` node has a token for the left-hand side, not an expression. The problem is that the parser does not know it is parsing an `l-value` until it hits the `=` operator.
In a complex `l-value` expression, the parser may only notice several tokens later. This is becauase the receiver of an assignment can be an expression, and, technically, an expression can be infinitely long.