use log::error;
use crate::expr::Expr;
use crate::lexer::Token;
use crate::{report, Literal, Stmt, TokenType};
use crate::TokenType::{LeftParen, RightParen};
/*
The parser takes the tokens as input and produces an abstract syntax tree, a more information-rich
data structure, as output. As a reminder, tokens are the output of the lexer, which takes raw
source code as input and groups them together based on the lexical grammar to produce tokens,
a data structure which contains information later stages of the interpreter/compiler can use.

Abstract syntax trees are evaluated via post-order traversal since you must
evaluate the leaves first, then up to the root.

Regular language is the rules describing how a raw sequence of characters is grouped into tokens.
A formal grammar takes a set of atomic pieces called its `alphabet` to produce a sequence of alphabets
with semantic meaning. For a parser, the input is a sequence of tokens and the output is an expression.
Essentially, grammars define which outputs are valid and which outputs are invalid.
For grammars, if you start with a set of rules, you can define all the possible valid
sets of results. These rules are called productions, and the resulting results are called derivations.

Each production in a context-free grammar has a head, defining its name, and a body, defining what it produces.
Context-free grammar are either:
1. Terminal: This is a letter from the grammar's alphabet that is not able to produce any more results.
2. Non-Terminal: Non-terminal refers to the grammar's structure that is able to compose new variations.

Furthermore, there may be more than one production with the same name. If you encounter a name with more than one body,
you can choose whichever body you want.

Recursion where the recursive non-terminal has a production on both sides implies that it is not regular.
This recursion is what allows us to produce an infinite number of strings using a finite set of grammar rules.

Syntactic sugar is a term for notation that wraps around the atomic pieces of the language for convenience.

Context-free grammar and EBNF notation help crystallize the informal syntax design of your language.

Example:
expression     → literal
               | unary
               | binary
               | grouping ;

Literal → NUMBER | STRING | "true" | "false" | "nil" ;
grouping       → "(" expression ")" ;
unary          → ( "-" | "!" ) expression ;
binary         → expression operator expression ;
operator       → "==" | "!=" | "<" | "<=" | ">" | ">="
               | "+"  | "-"  | "*" | "/" ;

Here, the example uses quoted strings for literals whose values exactly match what their lexemes represent.
On the other hand, the example uses capitalization for literals whose values may vary from what their lexemes represent.

This grammar is actually ambiguous. This means there exists a string w which can be derived into more than one parse tree.
The tree-like structure produced from the grammar rules is called the syntax tree. It determines the syntax of the language.
Every grammar production becomes a node in the AST.

One way to design an AST is to implement a base class and then
implement subclasses for each production rule according to their specific structure

A lexer generates a sequence of tokens from a string of source code. The tokens are generated based on
lexical grammar roles. The parser's job is to reverse-engineer which production rules were used to generate the token.
If more than one possible sequence of production rules is possible for a given string of tokens, the grammar is
ambiguous. Otherwise, it is unambiguous.

One way to solve the ambiguity problem is to split the grammar rules into different levels.
Here, each level corresponds to one grammar rule, and a grammar rule can only call the next grammar rule in the precedence hierarchy.
This way, precedence and associativity and embedded into the grammar rules and solve ambiguity.
The important thing to keep in mind when designing languages this way is that the expression must terminate.

For a grammar rule, if the first symbol of the body is the same as the head of the rule, then it is called left-recursive.

expression     → equality
equality       → comparison ( ( "==" | "!=" ) comparison )*
comparison     → term ( ( ">" | "<" ) term )*
term           → factor ( ( "+" | "-" ) factor )*
factor         → unary ( ( "*" | "/" ) unary )*
unary          → ( "-" | "!" ) unary
              | primary
primary        → NUMBER | "(" expression ")"

Designing a language is also a modeling problem.
Some techniques in parsing include top-down or bottom-up. A top-up approach
is a literal translation of the grammar rules into code. A bottom-up approach
starts from the highest precedence and builds up the expression into a
syntax tree. recursive descend is called `recursive` because when a grammar rule
refers to itself, it is a call to a recursive function.

Side note: In functions, the arguments are the actual values passed to a function call,
whereas parameters are the variables supplied in the local body of a function call.

In building a parser, each method for a grammar rule produces a syntax tree which is returned to the caller.
When a non-terminal is encountered, a function for that rule is called.
This is why left-recursive grammar structures are terrible, because it can lead
to stack overflow errors since the parser will keep calling the same method over and over again.

With recursive descent, the parser's state - which production rule it is in the middle of recognising -
is managed by the call stack of the language. Each production rule is a call frame in the call stack.
In order to reset the parser's state, we need to clear the call frames in the call stack.

One way to do this is by throwing exceptions and catching it. When an exception is caught,
we can synchronise the tokens to get the parser in the right state.

In correcting the parser's state, we want to discard tokens until we reach the next statement.
The next statement can be identified via `;` or keywords. This strategy is not perfect,
but it is a good best-effort since we already reported the error correctly. When properly implemented,
it will discard tokens that would have caused cascading errors, so the parser can resume parsing
the tokens at the next statement.
*/
#[derive(Debug)]
struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error")
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /*
    The parser has two jobs:
    1. Given a valid sequence of tokens, generate the corresponding syntax tree
    2. If provided an invalid sequence of tokens, detect the error and report to the user

    Good syntax error handling is hard because the parser has to handle ill-informed tokens all the time
    and try to make sense of it and return an appropriate error message to the user to get them back
    to the right place.

    Parser hard requirements:
    1. Detect errors and report them. If a parser allows a malformed syntax tree, then later phases
    of the interpreter will fail.
    2. Prevent infinite loops. A parser must detect infinite loops to avoid hanging or crashing.

    Parser soft requirements:
    1. Be fast
    2. Report as many distinct errors as possible.
    3, Avoid cascading errors.
    Error recovery is when your paser keeps searching for errors and trying to
    recover the errors even after encountering one.

    Synchronisation is the process when a parser gets its state and the sequence
    of forthcoming tokens aligned such that the next token does match the rule being
    parsed after encountering an error. It uses keywords as `safe points` as reference points of
    correct states and discards any incoming / incorrect tokens to prevent falsely
    reporting cascading errors.

    Another way to handle errors is to include it in the grammar rules. When an error matches,
    the parser reports the error instead of generating a syntax tree.
    These are called error productions.
    */
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                // since the `self.declaration` function is repeatedly called to process
                // a sequence of statements, it is the perfect place to synchronize
                Ok(stmt) => statements.push(stmt),
                Err(error) => self.synchronize(),
            }
        }
        statements
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_tokens(&[TokenType::Var]) {
            match self.var_declaration() {
                Ok(stmt) => Ok(stmt),
                Err(error) => panic!("Error in processing a variable declaration.")
            }
        } else if self.match_tokens(&[TokenType::Fun]) {
            match self.function() {
                Ok(stmt) => Ok(stmt),
                Err(error) => panic!("Error in processing a function.")
            } 
        } else {
            self.statement()
        }
    }

    fn function(&mut self) -> Result<Stmt, ParseError> {
        // we can reuse this function later when processing class methods
        // 1. Function name
        let name = self.consume(TokenType::Identifier,
                                "Expect function name.")?;
        
        // 2. Parameter list
        self.consume(TokenType::LeftParen,
                     "Expect '(' after function name.")?;

        let mut params = Vec::new();
        // the first if statement checks for the zero-parameter case
        if !self.check(&TokenType::RightParen) {
            loop {
                // the loop statement keeps parsing arguments as long as we can find 
                // arguments separated by a comma
                if params.len() >= 255 {
                    // same error style as the book
                    return Err(Parser::error(self.peek(), "Can't have more than 255 parameters."));
                }

                params.push(
                    self.consume(TokenType::Identifier,
                                 "Expect parameter name.")?
                );

                // no more parameters?
                if !self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen,
                     "Expect ')' after parameters.")?;

        // 3. Body
        // consuming for a left brace here gives a more precise error message
        // because we expect a left brace since we are expecting a body from a function declaration
        self.consume(TokenType::LeftBrace,
                     "Expect '{' before function body.")?;

        // self.block() parses the braced statement list
        let body = self.block();

        Ok(Stmt::Function {
            name,
            params,
            body,
        })
    }
    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer = if self.match_tokens(&[TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else {
            // If no initializer, default to `nil`
            None
        };

        self.consume(TokenType::SemiColon, "Expect ';' after variable declaration.")?;

        Ok(Stmt::Var {
            name,
            initializer,
        })
    }

    // pub fn parse(&mut self) -> Vec<Stmt> {
    //     let mut statements = Vec::new();
    //
    //     while !self.is_at_end() {
    //         match self.statement() {
    //             Ok(stmt) => statements.push(stmt),
    //             Err(_) => self.synchronize(), // Skip erroneous tokens
    //         }
    //     }
    //
    //     statements
    // }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_stmt(TokenType::Print) {
            self.print_stmt()
        } else if self.match_stmt(TokenType::LeftBrace) {
            Ok(Stmt::Block {statements: self.block()})
        } else if self.match_stmt(TokenType::If) {
          self.if_stmt()  
        } else if self.match_stmt(TokenType::While) {
            self.while_stmt()
        } else if self.match_stmt(TokenType::For) {
            self.for_stmt()
        } else if self.match_stmt(TokenType::Return) {
            self.return_statement()
        } else {
            self.expr_stmt()
        }
    }
    
    fn match_stmt(&mut self, expected: TokenType) -> bool {
        if self.check(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /*
    Since an expression can start through a number of different tokens, 
    it is difficult to tell if a return value is present. Instead, we check if it’s absent.
    We check if the next token is a semicolon. If so, then it cannot be an expression,
    and we return None.
    */
    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone(); // capture the `return` token

        let value = if !self.check(&TokenType::SemiColon) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.consume(TokenType::SemiColon, "Expect ';' after return value.")?;

        Ok(Stmt::Return {
            keyword,
            value,
        })
    }


    fn print_stmt(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?; // Propagate error
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Print {
            expression: Box::new(value),
        })
    }

    fn for_stmt(&mut self) -> Result<Stmt, ParseError> {
        // "for" has already been consumed by the caller.
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer: Option<Stmt> = if self.match_tokens(&[TokenType::SemiColon]) {
            None
        } else if self.match_tokens(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_stmt()?)
        };

        let condition: Option<Expr> = if !self.check(&TokenType::SemiColon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::SemiColon,
            "Expect ';' after loop condition.",
        )?;

        let increment: Option<Expr> = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::RightParen,
            "Expect ')' after for clauses.",
        )?;

        let mut body: Stmt = self.statement()?; // {...} or single stmt

        if let Some(inc_expr) = increment {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression {
                        expression: Box::new(inc_expr),
                    },
                ],
            };
        }

        let cond_expr = condition.unwrap_or(Expr::Literal {
            value: Literal::Bool(true), // infinite loop if none
        });
        body = Stmt::While {
            condition: Box::new(cond_expr),
            body: Box::new(body),
        };

        if let Some(init_stmt) = initializer {
            body = Stmt::Block {
                statements: vec![init_stmt, body],
            };
        }

        Ok(body)
    }

    fn while_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;

        let else_branch = if self.match_tokens(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        
        Ok(Stmt::If {
            conditional: Box::new(condition),
            consequent: Box::new(then_branch),
            alternative: else_branch,
        })
    }


    fn block(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::<Stmt>::new();
        while (self.check(&TokenType::RightBrace) && !self.is_at_end()) {
            statements.push(self.declaration().unwrap());
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")
            .expect("Expect '}' after block.");
        statements
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?; // Propagate error
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Expression {
            expression: Box::new(expr),
        })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.or_expr()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        // parse the left side first
        let expr = self.or_expr()?;

        // look for “=”
        if self.match_tokens(&[TokenType::Equal]) {
            let equals = self.previous().clone();  // keep for error reporting
            let value  = self.assignment()?;       // recurse for right side

            // only a variable is a valid assignment target
            if let Expr::Variable { name, .. } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            // any other LHS → error
            return Err(ParseError);
        }

        // no “=”: just return the original expression
        Ok(expr)
    }

    fn or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and_expr()?;

        // While we see consecutive "or" tokens, fold them left-associatively
        while self.match_tokens(&[TokenType::Or]) {
            let operator = self.previous().clone();   // the consumed "or"
            let right = self.and_expr()?;          // parse RHS
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_tokens(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right    = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }


    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.term()?;
        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    // it is possible to write a helper method to generalize the method for each
    // production rule
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.unary()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary();

        loop {
            if self.match_tokens(&[LeftParen]) {
                // each time we see a '(' we call finish call to parse the call expression
                // using the previously parsed as the callee
                expr = self.finish_call(expr?);
            } else {
                break
            }
        }
        expr
    }
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();

        // if the token immediately following is a right parenthesis, then stop
        // else, parse the arguments as expressions
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    // throwing an error is valid only when the parser does not know what state
                    // it has anymore. However, in this case, the state is still fine
                    crate::utils::error(self.peek().line, "Can't have more than 255 arguments")
                }
                arguments.push(self.expression()?);
                // syntax check
                if !self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        // the fact that the parser looks ahead at upcoming tokens to decide
        // how to parse puts recursive descent under the category of predictive parsers
        match self.peek().token_type {
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Bool(false),
                })
            }

            TokenType::True => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Bool(true),
                })
            }

            TokenType::Nil => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Nil,
                })
            }

            TokenType::Number | TokenType::String => {
                let literal = self.peek().literal.clone();
                self.advance();
                Ok(Expr::Literal { value: literal })
            }

            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")
                    .expect("TODO: panic message");
                Ok(Expr::Grouping {
                    expression: Box::new(expr),
                })
            }

            TokenType::Identifier => {
                Ok(Expr::Variable {
                    name: self.previous().clone(),
                    initializer: None
                })
            }
            _ => Err(Parser::error(self.peek(), "Expected an expression.")),
        }
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(Parser::error(self.peek(), message))
        }
    }

    fn error(token: &Token, message: &str) -> ParseError {
        match token.token_type {
            TokenType::Eof => {
                report(token.line, " at end", message);
            }
            _ => {
                report(token.line, &format!(" at '{}'", token.lexeme), message);
            }
        }

        ParseError
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        };
        self.previous().clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        if self.current == 0 {
            panic!("Index error: tried to access previous token at position 0.")
        };
        &self.tokens[self.current - 1]
    }

    fn synchronize(&mut self) {
        // this function will discard tokens until we encounter a boundary
        // condition so that the parser can resume parsing the file at the
        // next statement
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
