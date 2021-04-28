use std::collections::HashMap;

use crate::{
    ast::{Expr, FunctionStmt, Stmt},
    interpreter::Interpreter,
    token::Token,
};

pub struct Resolver<'interp> {
    interpreter: &'interp mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}
impl<'interp> Resolver<'interp> {
    pub fn new(interpreter: &'interp mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
        }
    }

    pub fn resolve(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve(statements);
                self.end_scope();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer);
                }
                self.define(name);
            }
            Stmt::Function(fun) => {
                self.declare(&fun.name);
                self.define(&fun.name);
                self.resolve_function(fun, FunctionType::Function);
            }
            Stmt::Expression { expression } => {
                self.resolve_expr(expression);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(branch) = else_branch {
                    self.resolve_stmt(branch);
                }
            }
            Stmt::Print { expression } => {
                self.resolve_expr(expression);
            }
            Stmt::Return { value, .. } => {
                if self.current_function == FunctionType::None {
                    todo!("Can't return from top-level code.");
                }
                if let Some(value) = value {
                    self.resolve_expr(value);
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
            }
            Stmt::Class { name, methods } => {
                self.declare(name);
                self.define(name);
                self.begin_scope();
                self.scopes.last_mut().unwrap().insert("this".into(), true);
                for method in methods {
                    self.resolve_function(
                        method,
                        FunctionType::Method,
                    );
                }
                self.end_scope()
            }
        }
    }

    fn resolve_expr(&mut self, expression: &Expr) {
        match expression {
            Expr::Variable { name } => {
                if let Some(false) = self.scopes.last().and_then(|it| it.get(&name.lexeme)) {
                    todo!(
                        "Can't read local variable in its own initializer. {}",
                        name.line
                    )
                }
                self.resolve_local(expression, name);
            }
            Expr::Assign { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expression, name);
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(callee);
                for argument in arguments {
                    self.resolve_expr(argument);
                }
            }
            Expr::Get { object, .. } => {
                self.resolve_expr(object);
            }
            Expr::Set { object, value, .. } => {
                self.resolve_expr(value);
                self.resolve_expr(object);
            }
            Expr::Grouping { expression } => {
                self.resolve_expr(expression);
            }
            Expr::Literal { .. } => {}
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Unary { right, .. } => {
                self.resolve_expr(right);
            }
            Expr::This { keyword } => {
                self.resolve_local(expression, keyword);
            }
        }
    }

    fn resolve_local(&mut self, expression: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(expression, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        fun: &FunctionStmt,
        kind: FunctionType,
    ) {
        let enclosing_function = self.current_function;
        self.current_function = kind;

        self.begin_scope();
        for param in &fun.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve(&fun.body);
        self.end_scope();

        self.current_function = enclosing_function;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                todo!("Already variable with this name in this scope.");
            }
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionType {
    None,
    Function,
    Method,
}
