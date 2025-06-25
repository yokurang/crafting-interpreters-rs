use std::env;
use crafting_interpreters::runner::{run_file, run_prompt};
use crafting_interpreters::lexer::{Token, TokenType};
use crafting_interpreters::expr::Expr;
use crafting_interpreters::{define_ast, Literal};

// fn main() {
// 
//     let expression = Expr::Binary {
//         left: Box::new(Expr::Unary {
//             operator: Token {
//                 token_type: TokenType::Minus,
//                 lexeme: "-".to_string(),
//                 literal: Literal::Nil,
//                 line: 1,
//             },
//             right: Box::new(Expr::Literal {
//                 value: Literal::Number(123.0),
//             }),
//         }),
//         operator: Token {
//             token_type: TokenType::Star,
//             lexeme: "*".to_string(),
//             literal: Literal::Nil,
//             line: 1,
//         },
//         right: Box::new(Expr::Grouping {
//             expression: Box::new(Expr::Literal {
//                 value: Literal::Number(45.67),
//             }),
//         }),
//     };
// 
//     let mut printer = AstPrinter;
//     println!("{}", printer.print(&expression));
// }

// pub fn main() -> () {
//     let args: Vec<String> = env::args().collect();
//     // args always includes the program name in args[0]
//     match args.len() {
//         1 => {
//             run_prompt();
//         }
//         2 => {
//             run_file(&args[1]);
//         }
//         _ => {
//             println!("Usage: jlox [script]");
//             std::process::exit(64);
//         }
//     }
// }

fn main() -> std::io::Result<()> {
    let output_dir = "./generated";

    define_ast(
        output_dir,
        "Expr",
        vec![
            "Binary   : Box<Expr> left, Token operator, Box<Expr> right",
            "Grouping : Box<Expr> expression",
            "Literal  : LiteralValue value",
            "Unary    : Token operator, Box<Expr> right",
        ],
    )?;

    define_ast(
        output_dir,
        "Stmt",
        vec![
            "Expression : Box<Expr> expression",
            "Print      : Box<Expr> expression",
        ],
    )?;

    Ok(())
}
