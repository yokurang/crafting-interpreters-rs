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
In a complex `l-value` expression, the parser may only notice several tokens later. This is because the receiver of an assignment can be an expression, and, technically, an expression can be infinitely long.

For assignments, it is implemented as a right-associative operator. The trick for assignment is to parse the LHS of the assignment operator as it would for a normal subexpression / `r-value`. It is only when it discovers a `=` operator that the parser inspects the built-in node.
This trick works because any variable used to store a value is also a valid expression. This means that we can parse both sides of the `=` as `r-values` and only if we encounter the assignment operator do we re-interpret the LHS as a variable we can store values into.

#### Scope

A scope defines a region of code where a name maps to a particular entity. Multiple scopes enable the same name to map to 
different values or entities in the same program. This is because scopes are different contexts in the same program.

Lexical scope (or less commonly known as static scope) is a specific type of scoping where the text of the program itself shows
where a scope begins and ends. In lox, as in most programming languages, variables are lexically scoped. When you see an
expression that uses some variable, you can see which expression the variable refers to by reading where it is lexically scoped. 

Side note: Dynamic scoping means that the scope of a variable is determined at runtime by a sequence
of function calls rather than where it is declared in the source code. 

Scope and environment are close cousins. The former is the theoretical concept, and the latter is the data structure that implements it. As the interpreter
works through code, syntax tree nodes which affect scope will change the environment.
In C and lox, the scope is controlled through curly brackets, which is why it is called `block scope`.

The beginning of a curly bracket defines a new scope. The scope ends when the parser encounters a closing curly bracket.
Any variables defined in this block scope will go `out of scope` and disappear, i,e. Freed from memory.

The main idea behind block scoping is `encapsulation`. This means that code in one scope should not interfere with
the scope of another block.

Consider this example:

```
// How loud?
var volume = 11;

// Silence.
volume = 0;

// Calculate size of 3x4x5 cuboid.
{
  var volume = 3 * 4 * 5;
  print volume;
}
```

When the block goes out of scope, the variable `volume` should not be deleted from the global environment since it is a 
totally different variable from the `volume` variable in the block.

When a local variable has the same name as a variable in an enclosing scope, it `shadows` the outer variable. 
The outer variable is no longer accessible by the code in the local scope because it is shadowed by a local variable, though it is still there.

When we enter a new block scope, we need to preserve the environment before entering a new block scope. We do this by defining a new
environment for each block scope we encounter. When we exit a block scope, we restore the environment with the previous one.

The interpreter also needs to handle any outer variables that are not being shadowed. This means that the interpreter does not only look at the innermost environment when
trying to lookup a variable and its associated value, but also all previous environments in order towards the outermost environment.

This is implemented by chaining the environment together. This can be done by having each environment hold a reference to the immediate environment enclosing its scope. 
We walk from the innermost environment to the outermost environment. This is how shadowing variables are also implemented. This is called a `parent pointer tree`.

The methods for `get` and `assign` can be implemented recursively or iteratively.

#### Implicit Variable Declaration

Implicit variable declaration is when the assignment operator creates a new variable when that variable has not been defined.
Languages that use implicit declaration must decide what happens when it isn't clear whether a declaration or assignment is taking place.
For example, when a user assigns a new variable, which scope does it go into? Furthermore, how does it interact with shadowing? Python always creates a new variable in the local scope, even if the variable already exists in an enclosing or outer scope.

## Control Flow

In this chapter, we will make the lox language turing complete.

### Turing Machines (Briefly)

A long time ago people wanted to answer the questions "Can all true statements be proven?" and "Can all functions
that can be defined be computed?" and, more fundamentally, "What do we mean when we say a function is 'computable'?". It turns out the answer
to the first two questions is false, and they are deeply intertwined. Alan Turing devised a precise
answer to the last question—a definition of exactly what kind of functions are computable. They each crafted a tiny system with
a minimum set of machinery to compute any function belonging to a broad class. The answer to the first question
is found by showing that there exists a function whose result should be true which cannot be computed.

Turing's system is called a turing machine, and Church's system is called lambda calculus. Both are widely used models for computation,
and lambda calculus serves as the core paradigm of many functional programming languages.

The essence of their results is that any programming language with a 
minimum set of expressiveness can compute any computable function. If your language can 
simulate a turing machine, since turing's machine can compute any computable function, by extension, so can your language.
A language that can do this is called turing-complete.

### Control Flow

There are two types of control flow: 

- Branching control flow or conditionals. These statements are composed of a conditional, consequent, and alternative. Only one of the consequent or alternative is executed at runtime.

- Looping control flow. These statements repeat a body of statements while a condition is true, otherwise known as the terminating condition.

Besides if statements and control operators, we also have the logical operators, namely the `and` and `or` statements.

The logical `and` operator returns true if both operands are true, else false. The logical `or` operator returns true if either of the operands is true. A neat way to think about the logical `and` operator is as an if statement which
returns if the first operand is false, else returns the result of the second operand. Likewise, the logical `or` operator can be thought of as an if statement that returns if the first operand evaluates to true, else returns the result of evaluating the second operand.

The two new operators are low in the precedence table, with `or` lower than `and`.

For loops have three parts:

1. An initializer that is executed once before anything else. It is usually an expression, but we can allow for variable declarations for flexibility.
In this case, the variable is scoped over the body of the for loop.
2. The condition is an expression such that the for loop is executed for as long as the condition is true. In other words, the for loop exists only when the condition evaluates to false.
3. Finally, there is an increment. This is an arbitrary expression that gets executed after the body statement of the for loop is evaluated each time. The value of the expression is discarded, so it must produce some side effect.

In this scenario, for loops are convenient wrappers for common code patterns that already exist in the language. This is called syntactic sugar. 
However, in sophisticated language implementations, every language feature that requires back-end support and optimization is expensive.

We can work around this issue by desugaring the syntactic sugar. Desugaring describes a process where the front end takes code using syntactic sugar and translates it to a more primitive form that the back end already knows.

## Functions

Function calls are typically made to functions with names. The name of the function being called is not part of the call syntax.
The callee-the function being called-can be any expression that evaluates to a function. Because of this, the function call syntax needs to be high in the precedence order.

In this example `getCallback()();`, there are two expressions. The first pair of parentheses has `getCallback` as its callee. It calls the `getCallback` function and returns its result.
The second pair of parentheses applies to the result of `getCallback()`, implying that the `getCallback()` function returns another function and the parentheses call that function.

It is the parentheses after an expression that indicate a function call. We can think of a parenthesis as a sort of postfix operator.

In functional languages, the normal way of defining a function that takes multiple arguments is as a series
of nested functions. Each function takes one argument and returns a new function. That function consumes the next argument and returns a new function, and so on.
Eventually, all the arguments are consumed and the nested function call is complete. This process is called currying.

This rule requires at least one argument expression, followed by zero or more other expressions,e ach preceded by a commona. To handle zero-argument calls, the call rule itself considers
the entire `arguments` production to be optional. 
In a complex `l-value` expression, the parser may only notice several tokens later. This is becauase the receiver of an assignment can be an expression, and, technically, an expression can be infinitely long.

### Callable Objects

Before implementing LoxCallable, we need to make the visit method more robust. It currently
ignores a couple of failure modes that we cannot pretend will not occur. First, what happens if the callee is not actually
something you can call? For example `"test"();`

Strings are not callable in Lox, The runtime representation of a Lox string is a Java string, so when
we cast that to LoxCallable,, the JVM will throw a ClassCastException. We do not want our interpreter
to vomit out some nasty Java stack trace and die. Instead, we need to check
the type ourselves first.

We still throw an exception, but now we’re throwing our own exception type, one that the interpreter knows to catch and report gracefully.

### Checking Arity

The other problem relates to the function's arity. Arity is the fancy
term for the number of arguments a function or operations expect. Unary operators have arity one,
binary operators have arity two, etc. With functions, the arity is determined by the number
of parameters it declares. 

Different languages take different approaches to the problem of a function being supplied the wrong number
of arity. Of course, most statically typed languages check this at compile time nad refuse to compile
the code if the argument count does not match the function's arity. JavaScript discords
any extra arguments you pass. If you do not pass enough, it fills in the missing parameters
with `undefined`, a value that represents null but also not really. Python is stricter. It raises
a runtime error if the argument list is too short or too long. 

I think the latter is a better approach. Passing the wrong number of arguments is almost always a bug, and it’s a mistake I do make in practice. Given that, the sooner the implementation draws my attention to it, the better. So for Lox, we’ll take Python’s approach. Before invoking the callable, we check to see if the argument list’s length matches the callable’s arity.

That requires a new method on the LoxCallable interface to ask it its arity.

We could push the arity checking into the concrete implementation of call(). But, since we’ll have multiple classes implementing LoxCallable, that would end up with redundant validation spread across a few classes. Hoisting it up into the visit method lets us do it in one place.

## Native Functions

We can theoretically call functions, but we have no functions to call yet. Before we get to user-defined functions, now is a good time to introduce a vital but often overlooked facet of language implementations—native functions. These are functions that the interpreter exposes to user code but that are implemented in the host language (in our case Java), not the language being implemented (Lox).

Sometimes these are called primitives, external functions, or foreign functions. Since these functions can be called while the user’s program is running, they form part of the implementation’s runtime. A lot of programming language books gloss over these because they aren’t conceptually interesting. They’re mostly grunt work.

But when it comes to making your language actually good at doing useful stuff, the native functions your implementation provides are key. They provide access to the fundamental services that all programs are defined in terms of. If you don’t provide native functions to access the file system, a user’s going to have a hell of a time writing a program that reads and displays a file.

Many languages also allow users to provide their own native functions. The mechanism for doing so is called a foreign function interface (FFI), native extension, native interface, or something along those lines. These are nice because they free the language implementer from providing access to every single capability the underlying platform supports. We won’t define an FFI for jlox, but we will add one native function to give you an idea of what it looks like.

## Telling Time

When we get to Part III and start working on a much more efficient implementation of Lox, we’re going to care deeply about performance. Performance work requires measurement, and that in turn means benchmarks. These are programs that measure the time it takes to exercise some corner of the interpreter.

We could measure the time it takes to start up the interpreter, run the benchmark, and exit, but that adds a lot of overhead—JVM startup time, OS shenanigans, etc. That stuff does matter, of course, but if you’re just trying to validate an optimization to some piece of the interpreter, you don’t want that overhead obscuring your results.

A nicer solution is to have the benchmark script itself measure the time elapsed between two points in the code. To do that, a Lox program needs to be able to tell time. There’s no way to do that now—you can’t implement a useful clock “from scratch” without access to the underlying clock on the computer.

So we’ll add clock(), a native function that returns the number of seconds that have passed since some fixed point in time. The difference between two successive invocations tells you how much time elapsed between the two calls. This function is defined in the global scope, so let’s ensure the interpreter has access to that.

Functions and variables here occupy the same namespace. In other languages, that may not be the case, i.e, they have separate namespaces.

## Function Declarations

We add a new production to the `declaration` rule we introduced back when we added
variables. Function declarations, like variables, bind a new name. That means they are allowed
only in places where a declaration is permitted.

A named function declaration is not really a single primitive operation. It is syntactic sugar for creating
a new function object, and binding that object to a new variable. If Lox had syntax
for annonymous functions, we would not need function declaration statements.

## Function Objects

After parsing syntax, we are normally ready to interpret, but first we need
to think about how to represent a lox function in Java. We need to keep track of the parameter list so that
we can bind them to the argument list when the function is called. We also need
to keep track of the function body for when the function is executed at runtime.

That is what the Function class is for. However, we do not want how the interpreter evaluates to bleed into
the syntax's frontend. For this reason, we wrap this class around a new class.

## Return Statements

Return statements are ways to return values out of the function to the callee.

If Lox were an expression-oriented language like Ruby or Scheme, the body would be an expression whose value is implicitly the function’s result. But in Lox, the body of a function is a list of statements which don’t produce values, so we need dedicated syntax for emitting a result. In other words, return statements. I’m sure you can guess the grammar already.

We’ve got one more—the final, in fact—production under the venerable statement rule. A return statement is the return keyword followed by an optional expression and terminated with a semicolon.

The return value is optional to support exiting early from a function that doesn’t return a useful value. In statically typed languages, “void” functions don’t return a value and non-void ones do. Since Lox is dynamically typed, there are no true void functions. The compiler has no way of preventing you from taking the result value of a call to a function that doesn’t contain a return statement.

This means every Lox function must return something, even if it contains no return statements at all. We use nil for this, which is why LoxFunction’s implementation of call() returns null at the end. In that same vein, if you omit the value in a return statement, we simply treat it as equivalent to:

## Returning from Calls

Interpreting a return statement is tricky. You can return from anywhere
within the body of a function. When the return statement is executed, the interpreter
needs to jump all the way out of whatever context it is currently in and cause
the function call to complete, like some kind of control flow construct.

For example

```aiignore
Interpreter.visitReturnStmt()
Interpreter.visitIfStmt()
Interpreter.executeBlock()
Interpreter.visitBlockStmt()
Interpreter.visitWhileStmt()
Interpreter.executeBlock()
LoxFunction.call()
Interpreter.visitCallExpr()
```

We need to get from the top of the stack all the way back to `call()`. This seems like an exception to me.
We will execute a return statement and use an exception to implement this and unwind
the interpret past the visit methods of all of the containing statements back to the code that
began executing the body.

If we have a return value, we evaluate it. Otherwise, we use nil. 
Then we take that value and wrap it around a custom exception class and throw it.
The main advantage of this implementation is that we do need additional overhead from
the stacktrace.

This class wraps the return value with the accoutrement's Java requires for a runtime exception class. The weird super constructor call with those null and false arguments disables some JVM machinery that we don’t need. Since we’re using our exception class for control flow and not actual error handling, we don’t need overhead like stack traces.

For the record, I’m not generally a fan of using exceptions for control flow. But inside a heavily recursive tree-walk interpreter, it’s the way to go. Since our own syntax tree evaluation is so heavily tied to the Java call stack, we’re pressed to do some heavyweight call stack manipulation occasionally, and exceptions are a handy tool for that.

## Local Functions and Closures

Reminder: Closure means that a function is able to remember and access its lexical scope, meaning
the surrounding functions and variables, even after the outer function has finished executing.

LoxFunction’s implementation of call() creates a new environment where it binds the function’s parameters. When I showed you that code, I glossed over one important point: What is the parent of that environment?

Right now, it is always globals, the top-level global environment. That way, if an identifier isn’t defined inside the function body itself, the interpreter can look outside the function in the global scope to find it. In the Fibonacci example, that’s how the interpreter is able to look up the recursive call to fib inside the function’s own body—fib is a global variable.

But recall that in Lox, function declarations are allowed anywhere a name can be bound. That includes the top level of a Lox script, but also the inside of blocks or other functions. Lox supports local functions that are defined inside another function, or nested inside a block.

Local functions are functions that are defined inside another function or a block.

```aiignore
fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    print i;
  }

  return count;
}

var counter = makeCounter();
counter(); // "1".
counter(); // "2".
```

Here, count() uses i which is declared outside of itself in the containing function makeCounter. makeCounter
returns a reference to the count() function and then its own body finishes executing completely.

Meanwhile, the top-level code invokes the returned count() function. That executes
the body of count(), which assigns to and reads i, even though the function where i is defined has already exited.

If you have never encountered a language with nested functions before, this might seem crazy, but users
do expect it to work. However, if you try to run it now, you will get an undefined variable error because the
call to counter cannot find i since it is linked directly to the global environment, which does not have i.
We lost the environment where i was defined.

So at the point where the function is declared, we can see i. But when we return from makeCounter() and exit its body, the interpreter discards that environment. Since the interpreter doesn’t keep the environment surrounding count() around, it’s up to the function object itself to hang on to it.

This data structure is called a closure because it “closes over” and holds on to the surrounding variables where the function is declared. Closures have been around since the early Lisp days, and language hackers have come up with all manner of ways to implement them. For jlox, we’ll do the simplest thing that works. In LoxFunction, we add a field to store an environment.

The closure environment is a data structure that is active when the function is declared, not called.  It represents
the lexical scope surrounding the function declaration. Finally, when we call
the function, we use that environment as the call's parents instead of using the global environment.

This creates an environment chain that goes from the function's body out through
the environments where the function is declared, all the way to the global environment.

The runtime environment chain matches the textual nesting of the source code.
Now, as you can see, the interpreter can still find i when it needs to because it’s in the middle of the environment chain. Try running that makeCounter() example now. It works!

# Resolving and Binding

In designing a language or a program, correctness is extremely important, and we must ensure
correctness in the context of semantics. Semantic analysis is the process of analyzing the user's source code
and extracting the meaning behind that source code without running the code.    =

## Static Scope

Lexical scoping: The scope of a variable is determined by its position in the source code during compilation.

A variable's value refers to the most recent declaration of the variable in the innermost environment enclosing the expression
using the variable.

In straight line code, the declaration preceding in text will also precede the usage in time. 
However, that is not always true. Functions may defer a chunk of code such that
its dynamic temporal execution no longer mirrors the static textual ordering.

## Scopes and Mutable Environments

In the interpreter, environments are the dynamic manifestation of static scopes.The two
mostly stay in sync with each other-we create a new environment when we enter a new scope, and discard
it when we leave the scope. There is one other operation we perform on environments: binding a variable
in one.

When we have a function and call that function, we get a new environment for the body of the function. The function has closure, which means it captures
the environment where the function was declared.

The interpreter dynamically creates a new environment for the function body. It is empty since
the function does not declare any variables. The parent of that environment is the function's closure-the outer block environment enclosing the function.

Inside the body of the function, we print the value of a. The interpreter looks up this value by walking
the chain of environments. It gets all the way to the global environment before finding it there and printing
"global." 

I chose to implement environments in a way that I hoped would agree with your informal intuition around scopes. We tend to consider all of the code within a block as being within the same scope, so our interpreter uses a single environment to represent that. Each environment is a mutable hash table. When a new local variable is declared, it gets added to the existing environment for that scope.

```aiignore
{
  var a;
  // 1.
  var b;
  // 2.
}
```

At the first marked line, only a is in scope. At the second line, both a and b are. If you define a “scope” to be a set of declarations, then those are clearly not the same scope—they don’t contain the same declarations. It’s like each var statement splits the block into two separate scopes, the scope before the variable is declared and the one after, which includes the new variable.

But in our implementation, environments do act like the entire block is one scope, just a scope that changes over time. Closures do not like that. When a function is declared, it captures a reference to the current environment. The function should capture a frozen snapshot of the environment as it existed at the moment the function was declared. But instead, in the Java code, it has a reference to the actual mutable environment object. When a variable is later declared in the scope that environment corresponds to, the closure sees the new variable, even though the declaration does not precede the function.

TLDR: Closures should capture a frozen snapshot of the environment at the time the function was created. A function should not be able to see
the environment as it evolves over time, i.e, cannot access variables which are declared or re-defined after the function was created.

## Persistent Environments

There is a style of programming that uses persistent data structures. Persistent data structures can
never be directly modified. They are also called immutable. Instead, any "modification" to an
existing structure produces a new object that contains all of the original data and the new modification.
The original is left unchanged.

If we were to apply that technique to the environment, then every time you declared a variable it would
return a new environment that contained all of the previously declared variables along with the one with the
new name. Declaring a variable would do the implicit "split" where you have an environment before
and after the variable declaration. 

A closure retains a reference to the environment instance in play when the function was declared. Since any
declarations in that block would produce a new environment object, the closure would not see the new
variables and the bug would be fixed.

This is a legit way to solve the problem.
However, instead of making the data structure static,
we will bake the static resolution into the access operation itself.

## Semantic Analysis

An interpreter resolves a variable-tracks down which declaration the variable refers to-each time a variable is evaluated. 
If a variable is in a loop and is evaluated 1000 times, then the interpreter resolves it 1000 times.

We know static scope means that a variable usage always resolves to the same declaration.
Given that, why are we doing it dynamically every time? Doing so doesn’t just open the hole that leads to our annoying bug, it’s also needlessly slow.

A better solution is to resolve each variable use once. Write a chunk of code
that inspects the user's program, finds every variable mentioned, and figures out which declaration each
refers to. This process is an example of a semantic analysis. Where a parser tells only if a program
is grammatically correct (a syntactic analysis), semantic analysis goes farther and starts to figure out
what pieces of the program actually mean. In this case, our analysis will resolve variable bingings. We will know
that an expression is a variable and which variable it is.

There are a lot of ways we could store the binding between a variable and its
declaration. We will store the resolution in a way that makes the most of the
existing environment class. In the first (correct) evaluation, we look at three environments in the chain before finding the global declaration of a. Then, when the inner a is later declared in a block scope, it shadows the global one.

The next lookup walks the chain, finds a in the second environment and stops there. Each environment corresponds to a single lexical scope where variables are declared. If we could ensure a variable lookup always walked the same number of links in the environment chain, that would ensure that it found the same variable in the same scope every time.

To resolve a variable iusage, we only need to calculate how many jumps awayu the declared variable
will be in the environment chain. The interesting question ius when to do this calculation.

Since we are calculating a static property based on the source code, implement in the parser. That is the traditional home, and is where we’ll put it later in clox. It would work here too, but I want an excuse to show you another technique. We’ll write our resolver as a separate pass.

## A Variable Resolution Pass

After the parser produces these syntax treee, but before the interpreter starts executing it, we will doa  single
walk over the syntax tree to resolve all of the variables it contains. Additional passes between parsing 
and execution are common. If Lox had static types, we could slide a type checkeer. Optimizations
are often implemented in separate passes like this too. Basically, any work that does not rely on state
that is only available is done this way. 

The variable resolution pass works like a short of mini-interpreter. It walks the tree, visiting
each node, but a static analysis is different from dynamic execution:

1. There are no side effects. When the static analysis visits a print statement, it does not print anything. Calls
to native functions or other operations that reach the standard out buffer are stubbed and have no effect.
2. There is no control flow. Loops are visited only once. Both branches are visited in if statements. Logic operators are not short-circuited.

## Resolving Variable Expressions

Variable declarations-and function declarations, which we will get to-write to the scope maps.
Those maps are read when we resolve variable expressions. 

## Resolving Assignment Expressions

The other expression that references a variable is an assignment. We first resolve the expression 
in the assignment in case that expression contains other variables that need to be resolved. We then resolve the variable being assigned to.

## Resolving Function Declarations

Finally, functions. Functions ind both names and introduce a scope. The name of the function
itself is bound in the surrounding scope where the function is declared. Wehn we step into
the function's body, we bind its parameters to the inner scope of that function.

We first declare and define the name of the function in the current scope. Unlike variables, we define the name eargerly,
before resolving the function's body. This lets a function recursively refer to itself in its own body. 

Then we resolve the function's body. It is a separate method since we will now use it for resolving lox methods when we add classes later. 
It creates a new scope for the bodt and then binds the parameters into that inner-most scope in the function's body.

Once that is ready, it resolves the function's body in that scope. This is different from how
the interpreter handles function declarations. At runtime, declaring a function does not do anything with the function's body. 
The body does not get touched until later when the function is called. In static analysis, we immediately
traverse into the function's body right then and there. 

## Resolving Other Syntax Tree Nodes

That covers the interesting corners of the grammars. We handle every place where
a variable is declared, read, or written, and every place where a scope is created or destroyed.
Even though they are not affected by variable resolution, we also need visit methods for all other syntax tree
nodes to recurse into their subtrees. 

Resolution is different from interpretation in that there is no control flow. For an if statement, we
resolve both the consequent and the alternative. Where a dynamic execution only steps into one of the branches that is run,
a static analysis is the converse-it analyzes any branch that could be run. Since either one could be reached
during runtime, we resolve both branches. 

For while statements, we resolve the condition and the body exactly once. Likewise, for logical operators,
we resolve both operands. We do not short-circuit. 

## Interpreting Resolved Variables

Each time the resolver class visits a variable, it tells the interpreter
how many scopes there are between the current scope and the scope where the vairable is defined.
At runtime, this corresponds to exactly the number of environments between the current environment
and the environment where the variable can be found. 

We want to store the resolution information somewhere, so we can use it when a variable or assignment expression is later executed. One obvious place
is in the syntax tree node itself. That is a fine approach, and many compilers do this too. 

We could do that, but it would require mucking around with the syntax tree. Instead,
we will take another approach and store it to the side in a map that associates each syntax tree node
with its resolved data. 

Interactive tools like IDEs often incrementally rephrase and re-resolve parts
of the user program. It may be challenging to find all the bits of state that need
recalculating when they are hiding in the foliage of the syntax tree. A benefit of storing
this data outside the nodes is that it makes it easy to discard the data.

You might think we need some sort of nested tree structure to avoid getting confused when there
are multiple expressions that reference the same variable, but each expression node is its own object with
its own unique identity. A single monolithic map doesn't have any trouble keeping them separated.

## Accessing a Resolved Variable

Accessing a resolved variable happens through the lookup function. First, we look up the resolved distance in the map.
Remember that we resolved only local variables. Globals are treated specially, and we do not
end up in the map (hence the name locals). So, if we cannot find a distance in the map, it must be global.
In that case, we look it up dynamically, directly in the global environment. That throws a runtime error if the variable is not defined.

If we do get a distance, we have a local variable, and we get to take advantage of the results of our static analysis. 
Instead of calling `get()`, we call this new method on the environment.

The old get() method dynamically walks the chain of enclosing environments, scouring each one to see if the variable might be hiding in there somewhere. But now we know exactly which environment in the chain will have the variable. We reach it using this helper method:

The helper method walks a fixed number of hops up the parent chain and returns the environment there.
Once we have that, `get_at()` simply returns the value of the variable in that environment's map.
It does not even have to check if the variable is there-we know it will be there because the resolver
found it before.

## Assigning to a Resolved Variable

We can also use a variable by assigning to it. The changes to visiting an assignment expression are similar.

In essence, once we know the number of jumps up the environment to access the variable, the respective `get()` and `assign()` functions only
need to traverse that many environments until it performs their respective operations. This
is guaranteed to be valid because the resolver ahs already done the work.


## Resolution Errors

Since we are doing semantic analysis pass, we have the opportunity to make the language's
semantics more precise. This will let us catch bugs early before running the code.

We do allow declaring multiple variables with the same name in the global scope, but doing so in a local scope is probably a mistake. If they knew the variable already existed, they would have assigned to it instead of using var. And if they didn’t know it existed, they probably didn’t intend to overwrite the previous one.

When we declare a variable in a local scope, we already know the names of every variable previously declared in that same scope. If we see a collision, we report an error.

## Invalid Return Errors

```aiignore
return "at top level";
```

This executes a return statement, but it’s not even inside a function at all. It’s top-level code. I don’t know what the user thinks is going to happen, but I don’t think we want Lox to allow this.

We can detect this statically. Much like we track scopes as we walk the tree, we can track whether or not we are inside a function.

We stash the previous value of the field in a local variable first. Remember, lox has local functions, so you can nest functions declarations arbitrarily deep. We need to track not just that we are in a function, but how deep we are in a function.

We could use an explicit stack of FunctionType values for that, but instead we will piggyback on the JVM. We store the previous value in a local on a Java stack.. 
When we are done resolving the function body, we restore the field to that value.

Now that we can always tell whether or not we’re inside a function declaration, we check that when resolving a return statement.

There is one more piece. Back at the main Lox class that stitches everything together, we are careful to not run the interpreter
if any parse errors are encountered. That check runs before the resolver so that we do not try to resolve syntactically invalid code.

But we also need to skip the interpreter if there are resolution errors, so we add another check.

You could imagine doing a lot more analysis here. For example, if we added break statements to Lox, we would probably want to ensure they are only used inside loops. 

We could go farther and report warnings for code that is not necessarily wrong but probably is not useful. 
For example, many IDEs will warn if you have unreachable code after a return statement or a local variable whose value is never read. 
All of that would be pretty easy to add to our static visiting pass, or as separate passes/

The choice of determining how many passes to implement is difficult. Though having multiple small, simpler passes are easier to maintain,
there is a real runtime cost to traversing the entire syntax tree multiple times (for each pass).
Consider bundling multiple passes into a single pass.