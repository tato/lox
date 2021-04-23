// use lox_derive_ast::make_ast;
use crate::token::Token;
use crate::value::LoxValue;

pub trait Visitor {
    type Return;
    fn visit(&mut self, expr: &mut Expr) -> Self::Return;
}
#[derive(Debug)]
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
        value: LoxValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn accept<R>(&mut self, visitor: &mut dyn Visitor<Return = R>) -> R {
        visitor.visit(self)
    }
}