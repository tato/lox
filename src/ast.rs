use std::any::Any;

use lox_derive_ast::make_ast;

use crate::token::Token;

make_ast! {
    Binary => left: Box<Expr>, operator: Token, right: Box<Expr> ;
    Grouping => expression: Box<Expr> ;
    Literal => value: Box<dyn Any> ;
    Unary => operator: Token, right: Box<Expr> ;
}
