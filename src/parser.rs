use std::{error::Error, fmt::Display};

use crate::{
    ast::{Expr, FunctionStmt, Stmt},
    token::{Token, TokenKind},
    value::RuntimeValue,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn exact(&mut self, kinds: &[TokenKind]) -> bool {
        for &kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParserError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(parser_error(self.peek(), message))
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    pub fn parse(mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = vec![];
        while !self.is_at_end() {
            if let Ok(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        let stmt = if self.exact(&[TokenKind::Class]) {
            self.class_declaration()
        } else if self.exact(&[TokenKind::Fun]) {
            Ok(Stmt::Function(self.function("function")?))
        } else if self.exact(&[TokenKind::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match stmt {
            Ok(s) => Ok(s),
            Err(e) => {
                self.synchronize();
                Err(e)
            }
        }
    }

    fn class_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Expect class name.")?;
        self.consume(TokenKind::LeftBrace, "Expect '{' before class body.")?;

        let mut methods = vec![];
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenKind::RightBrace, "Expect '}' after class body.")?;
        Ok(Stmt::Class { name, methods })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Expect variable name.")?;

        let initializer = if self.exact(&[TokenKind::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.exact(&[TokenKind::For]) {
            self.for_statement()
        } else if self.exact(&[TokenKind::If]) {
            self.if_statement()
        } else if self.exact(&[TokenKind::Print]) {
            self.print_statement()
        } else if self.exact(&[TokenKind::Return]) {
            self.return_statement()
        } else if self.exact(&[TokenKind::While]) {
            self.while_statement()
        } else if self.exact(&[TokenKind::LeftBrace]) {
            Ok(Stmt::Block {
                statements: self.block()?,
            })
        } else {
            self.expression_statement()
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statemets = vec![];
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statemets.push(self.declaration()?);
        }
        self.consume(TokenKind::RightBrace, "Expect '}' after block.")?;
        Ok(statemets)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?.into();
        let else_branch = if self.exact(&[TokenKind::Else]) {
            Some(self.statement()?.into())
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?.into();

        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.exact(&[TokenKind::Semicolon]) {
            None
        } else if self.exact(&[TokenKind::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenKind::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        let condition_semicolon =
            self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenKind::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenKind::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression {
                        expression: increment,
                    },
                ],
            }
        }

        body = Stmt::While {
            condition: condition.unwrap_or(Expr::Literal {
                value: Token {
                    kind: TokenKind::True,
                    lexeme: "true".into(),
                    literal: RuntimeValue::Bool(true),
                    line: condition_semicolon.line,
                    scanner_index: condition_semicolon.scanner_index,
                },
            }),
            body: body.into(),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            }
        }

        Ok(body)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParserError> {
        let keyword = self.previous();
        let value = if !self.check(TokenKind::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon, "Expect ';' after 'return'.")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn function(&mut self, kind: &str) -> Result<FunctionStmt, ParserError> {
        let name = self.consume(TokenKind::Identifier, &format!("Expect {} name.", kind))?;
        self.consume(
            TokenKind::LeftParen,
            &format!("Expect '(' after {} name", kind),
        )?;
        let mut parameters = vec![];
        if !self.check(TokenKind::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(ParserError {
                        token: self.peek(),
                        message: "Can't have more than 255 arguments.".into(),
                    });
                } // TODO! Report but don't print error
                parameters.push(self.consume(TokenKind::Identifier, "Expect parameter name.")?);
                if !self.exact(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenKind::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block()?;
        Ok(FunctionStmt {
            name,
            params: parameters,
            body,
        })
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.or()?;

        if self.exact(&[TokenKind::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable { name, .. } = expr {
                Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                })
            } else if let Expr::Get { name, object } = expr {
                Ok(Expr::Set {
                    name,
                    object,
                    value: value.into(),
                })
            } else {
                Err(ParserError {
                    token: equals,
                    message: "Invalid assignment target.".into(),
                })
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.and()?;
        while self.exact(&[TokenKind::Or]) {
            let operator = self.previous();
            let right = self.and()?.into();
            expr = Expr::Logical {
                left: expr.into(),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;
        while self.exact(&[TokenKind::And]) {
            let operator = self.previous();
            let right = self.equality()?.into();
            expr = Expr::Logical {
                left: expr.into(),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        while self.exact(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;

        while self.exact(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while self.exact(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while self.exact(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.exact(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: right.into(),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.primary()?;
        loop {
            if self.exact(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.exact(&[TokenKind::Dot]) {
                let name =
                    self.consume(TokenKind::Identifier, "Expect property name after '.'.")?;
                expr = Expr::Get {
                    object: expr.into(),
                    name,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParserError> {
        let mut arguments = vec![];
        if !self.check(TokenKind::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(ParserError {
                        token: self.peek(),
                        message: "Can't have more than 255 arguments.".into(),
                    });
                } // TODO! Report but don't print error
                arguments.push(self.expression()?);
                if !self.exact(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenKind::RightParen, "Expect ')' after arguments.")?;
        Ok(Expr::Call {
            callee: callee.into(),
            paren,
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        if self.exact(&[TokenKind::False]) {
            Ok(Expr::Literal {
                value: self.previous(),
            })
        } else if self.exact(&[TokenKind::True]) {
            Ok(Expr::Literal {
                value: self.previous(),
            })
        } else if self.exact(&[TokenKind::Nil]) {
            Ok(Expr::Literal {
                value: self.previous(),
            })
        } else if self.exact(&[TokenKind::Number, TokenKind::String]) {
            Ok(Expr::Literal {
                value: self.previous(),
            })
        } else if self.exact(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
            Ok(Expr::Grouping {
                expression: expr.into(),
            })
        } else if self.exact(&[TokenKind::This]) {
            Ok(Expr::This {
                keyword: self.previous(),
            })
        } else if self.exact(&[TokenKind::Identifier]) {
            Ok(Expr::Variable {
                name: self.previous(),
            })
        } else {
            Err(parser_error(self.peek(), "Expect expression."))
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
                _ => self.advance(),
            };
        }
    }
}

fn report(line: usize, wher: &str, message: &str) {
    println!("[Line {}] Error {}: {}", line, wher, message);
    // hadError = true;
}
fn parser_error(token: Token, message: &str) -> ParserError {
    report(token.line, &format!("at '{}'", token.lexeme), message);
    ParserError {
        token,
        message: message.to_string(),
    }
}

#[derive(Debug)]
pub struct ParserError {
    token: Token,
    message: String,
}
impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for ParserError {}
