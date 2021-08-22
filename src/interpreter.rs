use std::{collections::HashMap, fmt::Display};

use crate::ast::{BinOp, Expr, Literal, Stmt};

pub struct Environment<'a> {
    scope: HashMap<String, Option<LoxResult>>,
    parent: Option<Box<&'a Environment<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Environment {
            scope: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(env: &'a Environment) -> Self {
        Environment {
            scope: HashMap::new(),
            parent: Some(Box::new(env)),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Option<LoxResult>> {
        if let Some(parent) = &self.parent {
            self.scope.get(key).or_else(|| parent.get(key))
        } else {
            self.scope.get(key)
        }
    }

    pub fn set(&mut self, key: String, value: Option<LoxResult>) {
        self.scope.insert(key, value);
    }
}

pub trait Interpretable {
    fn eval(&self, environment: &mut Environment) -> Result<LoxResult, LoxRuntimeError>;
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum LoxResult {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

impl Display for LoxResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Nil => write!(f, "Nil"),
        }
    }
}

#[derive(Debug)]
pub struct LoxRuntimeError {
    message: String,
    index: usize,
    len: usize,
}

impl std::error::Error for LoxRuntimeError {}

impl Display for LoxRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {} at {} until {}",
            self.message, self.index, self.len
        )
    }
}

impl LoxResult {
    fn get_type(&self) -> LoxType {
        match self {
            Self::Number(_) => LoxType::Number,
            Self::Str(_) => LoxType::Str,
            Self::Bool(_) => LoxType::Bool,
            Self::Nil => LoxType::Nil,
        }
    }

    fn unwrap_number(self) -> f64 {
        match self {
            Self::Number(n) => n,
            _ => panic!("LoxResult is not a number"),
        }
    }

    fn unwrap_string(self) -> String {
        match self {
            Self::Str(n) => n.clone(),
            _ => panic!("LoxResult is not a number"),
        }
    }
}

#[derive(PartialEq, Debug)]
enum LoxType {
    Number,
    Str,
    Bool,
    Nil,
}

impl Interpretable for Stmt {
    fn eval(
        &self,
        environment: &mut Environment,
    ) -> std::result::Result<LoxResult, LoxRuntimeError> {
        match self {
            Stmt::Expression(e) => e.eval(environment),
            Stmt::Print(e) => {
                e.eval(environment).map(|r| println!("{}", r))?;
                Ok(LoxResult::Nil)
            }
            Stmt::Variable(e, v) => {
                let value = match v {
                    Some(e) => Some(e.eval(environment)?),
                    _ => None,
                };
                environment.set(e.clone(), value);
                Ok(LoxResult::Nil)
            }
            Stmt::Block(stmts) => {
                let mut scoped_env = Environment::with_parent(environment);
                for stmt in stmts {
                    stmt.eval(&mut scoped_env)?;
                }
                Ok(LoxResult::Nil)
            }
        }
    }
}

impl Interpretable for Expr {
    fn eval(&self, env: &mut Environment) -> std::result::Result<LoxResult, LoxRuntimeError> {
        let res = match self {
            Self::Variable { index, len, value } => match env.get(value) {
                Some(value) => match value {
                    Some(res) => Ok(res.clone()),
                    _ => Ok(LoxResult::Nil),
                },
                _ => Err(LoxRuntimeError {
                    message: format!("The variable was not initialized before usage"),
                    index: *index,
                    len: *index + *len,
                }),
            }?,
            Self::Assign {
                index,
                len,
                key,
                value,
            } => {
                if let Some(_) = env.get(key) {
                    let res = value.eval(env)?;
                    env.set(key.clone(), Some(res.clone()));
                    res
                } else {
                    Err(LoxRuntimeError {
                        message: format!("Variable \"{}\" was not initialized", key),
                        index: *index,
                        len: *len,
                    })?
                }
            }
            Self::Literal {
                value,
                index: _,
                len: _,
            } => match value {
                Literal::Number(n) => LoxResult::Number(*n),
                Literal::Str(n) => LoxResult::Str(n.clone()),
                Literal::True => LoxResult::Bool(true),
                Literal::False => LoxResult::Bool(false),
                Literal::Nil => LoxResult::Nil,
            },
            Self::Unary {
                operator,
                right,
                index,
                len,
            } => {
                let right = right.eval(env)?;

                match operator {
                    crate::ast::UnaryOp::LogicNegate => match right {
                        LoxResult::Bool(b) => LoxResult::Bool(!b),
                        _ => Err(LoxRuntimeError {
                            message: format!("Cant negate type {:?}", right.get_type()),
                            index: *index,
                            len: *len,
                        })?,
                    },
                    crate::ast::UnaryOp::Negate => match right {
                        LoxResult::Number(n) => LoxResult::Number(-n),
                        _ => Err(LoxRuntimeError {
                            message: format!("Cant negate type {:?}", right.get_type()),
                            index: *index,
                            len: *len,
                        })?,
                    },
                }
            }
            Self::Grouping {
                expr,
                index: _,
                len: _,
            } => expr.eval(env)?,
            Self::Ternary {
                condition,
                left,
                right,
                index,
                len,
            } => {
                let condition = condition.eval(env)?;
                let condition = match condition {
                    LoxResult::Bool(b) => b,
                    r => Err(LoxRuntimeError {
                        message: format!(
                        "The condition of a ternary operator must resolve to a boolean but was {:?}",
                        r.get_type()
                    ), index: *index, len: *len})?,
                };
                if condition {
                    left.eval(env)?
                } else {
                    right.eval(env)?
                }
            }
            Self::Binary {
                left,
                right,
                operator,
                index,
                len,
            } => {
                let l = left.eval(env)?;
                let r = right.eval(env)?;
                if l.get_type() != r.get_type() && operator != &BinOp::Comma {
                    Err(LoxRuntimeError {
                        message: format!(
                            "Cant operate on {:?} and {:?}",
                            l.get_type(),
                            r.get_type(),
                        ),
                        index: *index,
                        len: *len,
                    })?;
                }
                let res = match operator {
                    BinOp::Sum => match l.get_type() {
                        LoxType::Number => LoxResult::Number(l.unwrap_number() + r.unwrap_number()),
                        LoxType::Str => LoxResult::Str(l.unwrap_string() + &r.unwrap_string()),
                        n => Err(LoxRuntimeError {
                            message: format!("Can't perform Sum on {:?}", n),
                            index: *index,
                            len: *len,
                        })?,
                    },
                    BinOp::Substraction => LoxResult::Number(l.unwrap_number() - r.unwrap_number()),
                    BinOp::Product => LoxResult::Number(l.unwrap_number() * r.unwrap_number()),
                    BinOp::Division => LoxResult::Number(l.unwrap_number() / r.unwrap_number()),
                    BinOp::Equals => LoxResult::Bool(l == r),
                    BinOp::GreaterThan => LoxResult::Bool(l > r),
                    BinOp::GreaterThanEquals => LoxResult::Bool(l >= r),
                    BinOp::LessThan => LoxResult::Bool(l < r),
                    BinOp::LessThanEquals => LoxResult::Bool(l <= r),
                    BinOp::NotEquals => LoxResult::Bool(l != r),
                    BinOp::Comma => r,
                };

                res
            }
        };
        Ok(res)
    }
}
