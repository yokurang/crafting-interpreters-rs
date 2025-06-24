use crate::expr::Expr;
use crate::lexer::Token;
use crate::{Literal, TokenType};
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

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            // what to do if a syntax error occurs?
            Err(_) => None,
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
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
        self.primary()
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
                crate::lexer::report(token.line, " at end", message);
            }
            _ => {
                crate::lexer::report(token.line, &format!(" at '{}'", token.lexeme), message);
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
