use std::any::Any;

// use lox_derive_ast::make_ast;

use crate::token::{LiteralValue, Token};

// make_ast! {
//     Binary => left: Box<Expr>, operator: Token, right: Box<Expr> ;
//     Grouping => expression: Box<Expr> ;
//     Literal => value: Box<dyn Any> ;
//     Unary => operator: Token, right: Box<Expr> ;
// }

trait Visitor {
    type Return;
    fn visit(&mut self, expr: &mut Expr) -> Self::Return;
}

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Option<Box<dyn LiteralValue>>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    }
}

impl Expr {
    fn accept<R>(&mut self, visitor: &mut dyn Visitor<Return=R>) -> R {
        visitor.visit(self)
    }
}

pub struct AstPrinter { }
impl Visitor for AstPrinter {
    type Return = String;
    fn visit(&mut self, expr: &mut Expr) -> Self::Return {
        match expr {
            Expr::Binary{left, operator, right} => {
                self.parenthesize(&operator.lexeme(), &mut [left, right])
            },
            Expr::Grouping{expression} => {
                self.parenthesize("group", &mut [expression])
            },
            Expr::Literal{value} => {
                match value {
                    None => "nil".to_string(),
                    Some(val) => val.to_string()
                }
            },
            Expr::Unary{operator, right} => {
                self.parenthesize(&operator.lexeme(), &mut [right])
            },
        }
    }
}
impl AstPrinter {
    pub fn print(&mut self, expr: &mut Expr) -> String {
        self.visit(expr)
    }

    fn parenthesize(&mut self, name: &str, exprs: &mut [&mut Expr]) -> String {
        let mut result = "(".to_string();
        result += name;
        for expr in exprs {
            result += " ";
            result += &expr.accept(self);
        }
        result += ")";
        result
    }
}


