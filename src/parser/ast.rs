use crate::lexer::{Token, Literal};
use crate::expr::{Expr, Visitor};

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let mut result = String::from("(");
        result.push_str(name);

        for expr in exprs {
            result.push(' ');
            result.push_str(&expr.accept(self));
        }

        result.push(')');
        result
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Binary { left, operator, right } = expr {
            self.parenthesize(&operator.lexeme, &[left, right])
        } else {
            unreachable!()
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Grouping { expression } = expr {
            self.parenthesize("group", &[expression])
        } else {
            unreachable!()
        }
    }

    fn visit_literal_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Literal { value } = expr {
            match value {
                Literal::Nil => "nil".to_string(),
                Literal::Number(n) => n.to_string(),
                Literal::String(s) => format!("\"{}\"", s),
                Literal::Bool(b) => b.to_string(),
            }
        } else {
            unreachable!()
        }
    }

    fn visit_unary_expr(&mut self, expr: &Expr) -> String {
        if let Expr::Unary { operator, right } = expr {
            self.parenthesize(&operator.lexeme, &[right])
        } else {
            unreachable!()
        }
    }
}
