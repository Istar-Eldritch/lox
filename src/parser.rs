use std::{
    convert::{TryFrom, TryInto},
    fmt::Display,
    iter::Peekable,
};

use crate::{
    ast,
    lexer::{Token, TokenKind},
};
use crate::{
    ast::Stmt,
    lexer::{self, KeywordKind},
};

#[derive(Debug)]
pub struct LoxSyntaxError {
    message: String,
    index: usize,
    len: usize,
}

impl std::error::Error for LoxSyntaxError {}

impl Display for LoxSyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Error: {} at {} until {}",
            self.message, self.index, self.len
        )
    }
}

impl TryFrom<lexer::Token> for ast::BinOp {
    type Error = String; // Token is not a valid BinOp
    fn try_from(t: lexer::Token) -> Result<Self, Self::Error> {
        use ast::BinOp;
        let op = match t.kind {
            TokenKind::Equals => BinOp::Equals,
            TokenKind::NotEquals => BinOp::NotEquals,
            TokenKind::GreaterThan => BinOp::GreaterThan,
            TokenKind::GreaterThanEquals => BinOp::GreaterThanEquals,
            TokenKind::LessThan => BinOp::LessThan,
            TokenKind::LessThanEquals => BinOp::LessThanEquals,
            TokenKind::Minus => BinOp::Substraction,
            TokenKind::Plus => BinOp::Sum,
            TokenKind::Star => BinOp::Product,
            TokenKind::Slash => BinOp::Division,
            TokenKind::Comma => BinOp::Comma,
            tk => Err(format!("{:?} is not a valid binary operator", tk))?,
        };
        Ok(op)
    }
}

impl TryFrom<lexer::Token> for ast::UnaryOp {
    type Error = String;

    fn try_from(t: lexer::Token) -> Result<Self, Self::Error> {
        use ast::UnaryOp;
        let op = match t.kind {
            TokenKind::Minus => UnaryOp::Negate,
            TokenKind::Bang => UnaryOp::LogicNegate,
            tk => Err(format!("{:?} is not a valid unary operation", tk))?,
        };
        Ok(op)
    }
}

pub fn parse<P: Iterator<Item = lexer::Token> + Clone>(
    tokens: &mut Peekable<P>,
) -> Result<Vec<ast::Stmt>, LoxSyntaxError> {
    let mut statements = Vec::new();
    while tokens.peek().is_some() {
        statements.push(statement(tokens)?)
    }
    Ok(statements)
}

fn statement(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Stmt, LoxSyntaxError> {
    match tokens.peek() {
        Some(t) if t.kind == TokenKind::Keyword(KeywordKind::Print) => {
            tokens.next();
            print_statement(tokens)
        }
        _ => expression_statement(tokens),
    }
}

fn print_statement(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Stmt, LoxSyntaxError> {
    let expr = expression(tokens)?;
    match tokens.next() {
        Some(t) if t.kind == TokenKind::Semicolon => Ok(Stmt::Print(expr)),
        _ => Err(LoxSyntaxError {
            message: String::from("Expected ';' after value."),
            index: expr.index() + expr.len(),
            len: 0,
        }),
    }
}

fn expression_statement(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Stmt, LoxSyntaxError> {
    let expr = expression(tokens)?;
    match tokens.next() {
        Some(t) if t.kind == TokenKind::Semicolon => Ok(Stmt::Expression(expr)),
        _ => Err(LoxSyntaxError {
            message: String::from("Expected ';' after value."),
            index: expr.index() + expr.len(),
            len: 0,
        }),
    }
}

fn expression(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    ternary(tokens)
}

fn ternary(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    let mut expr = comma(tokens)?;
    if let Some(t) = tokens.peek() {
        if t.kind == TokenKind::Interrogation {
            tokens.next();
            let left = ternary(tokens)?;
            if let Some(t) = tokens.next() {
                if t.kind == TokenKind::Colon {
                    let right = ternary(tokens)?;
                    let index = expr.index();
                    let len = right.index() + right.len() - index;
                    expr = ast::Expr::Ternary {
                        condition: expr.into(),
                        left: left.into(),
                        right: right.into(),
                        index,
                        len,
                    };
                } else {
                    Err(LoxSyntaxError {
                        message: String::from(
                            "Ternary operation missing one branch, expected colon instead",
                        ),
                        index: t.index,
                        len: t.len,
                    })?;
                }
            } else {
                Err(LoxSyntaxError {
                    message: String::from("Ternary operation missing one branch, expected colon"),
                    index: left.index(),
                    len: left.len(),
                })?;
            }
        }
    };
    Ok(expr)
}

fn comma(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    let mut expr = equality(tokens)?;
    while matches_any(tokens, vec![lexer::TokenKind::Comma]) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into().unwrap();
        let right = equality(tokens)?;
        let index = expr.index();
        let len = right.index() + right.len() - expr.index();
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
            index,
            len,
        };
    }
    Ok(expr)
}

fn equality(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    use crate::lexer::TokenKind::*;
    let mut expr = comparison(tokens)?;
    while matches_any(tokens, vec![NotEquals, Equals]) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into().unwrap();
        let right: ast::Expr = comparison(tokens)?;
        let index = expr.index();
        let len = right.index() + right.len() - expr.index();
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
            index,
            len,
        };
    }
    Ok(expr)
}

fn comparison(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    use crate::lexer::TokenKind::*;
    let mut expr = term(tokens)?;
    while matches_any(
        tokens,
        vec![GreaterThan, GreaterThanEquals, LessThan, LessThanEquals],
    ) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into().unwrap();
        let right: ast::Expr = term(tokens)?;
        let index = expr.index();
        let len = right.index() + right.len() - expr.index();
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
            index,
            len,
        };
    }
    Ok(expr)
}

fn term(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    use crate::lexer::TokenKind::*;
    let mut expr = factor(tokens)?;
    while matches_any(tokens, vec![Minus, Plus]) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into().unwrap();
        let right: ast::Expr = factor(tokens)?;
        let index = expr.index();
        let len = right.index() + right.len() - expr.index();
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
            index,
            len,
        };
    }
    Ok(expr)
}

fn factor(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    use crate::lexer::TokenKind::*;
    let mut expr = unary(tokens)?;
    while matches_any(tokens, vec![Slash, Star]) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into().unwrap();
        let right: ast::Expr = unary(tokens)?;
        let index = expr.index();
        let len = right.index() + right.len() - expr.index();
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
            index,
            len,
        };
    }
    Ok(expr)
}

fn unary(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    use crate::lexer::TokenKind::*;
    if matches_any(tokens, vec![Bang, Minus]) {
        let op_token = tokens.next().unwrap();
        let index = op_token.index;
        let operator: ast::UnaryOp = op_token.try_into().unwrap();
        let right = unary(tokens)?;
        let len = right.index() + right.len() - index;

        Ok(ast::Expr::Unary {
            operator,
            right: right.into(),
            index,
            len,
        })
    } else {
        primary(tokens)
    }
}

fn primary(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, LoxSyntaxError> {
    use crate::lexer::{KeywordKind::*, LiteralKind::*, TokenKind::*};
    if let Some(t) = tokens.next() {
        let expr = match t.kind {
            Keyword(True) => ast::Expr::Literal {
                value: ast::Literal::True,
                index: t.index,
                len: t.len,
            },
            Keyword(False) => ast::Expr::Literal {
                value: ast::Literal::False,
                index: t.index,
                len: t.len,
            },
            Keyword(Nil) => ast::Expr::Literal {
                value: ast::Literal::Nil,
                index: t.index,
                len: t.len,
            },
            Literal(k) => match k {
                Number(n) => ast::Expr::Literal {
                    value: ast::Literal::Number(n),
                    index: t.index,
                    len: t.len,
                },
                Str {
                    terminated: _,
                    value,
                } => ast::Expr::Literal {
                    value: ast::Literal::Str(value),
                    index: t.index,
                    len: t.len,
                },
            },
            LeftParen => {
                let expr = expression(tokens)?;
                if let Some(t) = tokens.next() {
                    if t.kind == RightParen {
                        let index = expr.index();
                        let len = expr.len();
                        ast::Expr::Grouping {
                            expr: expr.into(),
                            index,
                            len,
                        }
                    } else {
                        Err(LoxSyntaxError {
                            message: format!(
                                "The token {:?} was not expected, a ')' was expected",
                                t.kind,
                            ),
                            index: t.index,
                            len: t.len,
                        })?
                    }
                } else {
                    Err(LoxSyntaxError {
                        message: String::from("Expected ')' after grouped expression"),
                        index: expr.index(),
                        len: expr.len(),
                    })?
                }
            }
            tk => Err(LoxSyntaxError {
                message: format!("Token \"{:?}\" does not match a valid expression", tk),
                index: t.index,
                len: t.len,
            })?,
        };
        Ok(expr)
    } else {
        // TODO: This should be captured and managed acordingly, the index and len are invalid (maybe a different type of error?)
        Err(LoxSyntaxError {
            message: String::from("The expression is does not have a leaf node"),
            index: 0,
            len: 0,
        })
    }
}

fn matches_any<P: Iterator<Item = lexer::Token> + Clone>(
    tokens: &Peekable<P>,
    to_match: Vec<crate::lexer::TokenKind>,
) -> bool {
    let mut tokens = tokens.clone();
    for kind in to_match {
        match tokens.peek() {
            Some(t) if t.kind == kind => {
                return true;
            }
            _ => {}
        };
    }
    false
}

#[cfg(test)]
mod tests {
    use super::expression;
    use crate::ast::{BinOp::*, Expr::*, Literal::*};
    use crate::lexer::{tokenize, TokenKind};

    #[test]
    fn parse_comma_operator() {
        let mut tokens = tokenize("1,2,3")
            .filter(|t| t.kind != TokenKind::Whitespace)
            .peekable();
        let ast = expression(&mut tokens).unwrap();
        let expected = Binary {
            left: Binary {
                left: Literal {
                    value: Number(1.0),
                    index: 0,
                    len: 1,
                }
                .into(),
                operator: Comma,
                right: Literal {
                    value: Number(2.0),
                    index: 2,
                    len: 1,
                }
                .into(),
                index: 0,
                len: 3,
            }
            .into(),
            operator: Comma,
            right: Literal {
                value: Number(3.0),
                index: 4,
                len: 1,
            }
            .into(),
            index: 0,
            len: 5,
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn parse_ternary_expression() {
        // simple
        let mut tokens = tokenize("true ? 1 : 2")
            .filter(|t| t.kind != TokenKind::Whitespace)
            .peekable();
        // println!("{:?}", tokens.clone().collect::<Vec<crate::lexer::Token>>());
        let ast = expression(&mut tokens).unwrap();
        let expected = Ternary {
            condition: Literal {
                value: True,
                index: 0,
                len: 4,
            }
            .into(),
            left: Literal {
                value: Number(1.0),
                index: 7,
                len: 1,
            }
            .into(),
            right: Literal {
                value: Number(2.0),
                index: 11,
                len: 1,
            }
            .into(),
            index: 0,
            len: 12,
        };

        assert_eq!(ast, expected);

        // eq on condition
        let mut tokens = tokenize("1 == 2 ? 1 : 2")
            .filter(|t| t.kind != TokenKind::Whitespace)
            .peekable();
        let ast = expression(&mut tokens).unwrap();
        let expected = Ternary {
            condition: Binary {
                left: Literal {
                    value: Number(1.0),
                    index: 0,
                    len: 1,
                }
                .into(),
                operator: Equals.into(),
                right: Literal {
                    value: Number(2.0),
                    index: 5,
                    len: 1,
                }
                .into(),
                index: 0,
                len: 6,
            }
            .into(),
            left: Literal {
                value: Number(1.0),
                index: 9,
                len: 1,
            }
            .into(),
            right: Literal {
                value: Number(2.0),
                index: 13,
                len: 1,
            }
            .into(),
            index: 0,
            len: 14,
        };

        assert_eq!(ast, expected);

        // binary op on branches
        let mut tokens = tokenize("true ? 1 - 2 : 1 + 2")
            .filter(|t| t.kind != TokenKind::Whitespace)
            .peekable();
        let ast = expression(&mut tokens).unwrap();
        let expected = Ternary {
            condition: Literal {
                value: True,
                index: 0,
                len: 4,
            }
            .into(),
            left: Binary {
                left: Literal {
                    value: Number(1.0),
                    index: 7,
                    len: 1,
                }
                .into(),
                operator: Substraction.into(),
                right: Literal {
                    value: Number(2.0),
                    index: 11,
                    len: 1,
                }
                .into(),
                index: 7,
                len: 5,
            }
            .into(),
            right: Binary {
                left: Literal {
                    value: Number(1.0),
                    index: 15,
                    len: 1,
                }
                .into(),
                operator: Sum.into(),
                right: Literal {
                    value: Number(2.0),
                    index: 19,
                    len: 1,
                }
                .into(),
                index: 15,
                len: 5,
            }
            .into(),
            index: 0,
            len: 20,
        };

        assert_eq!(ast, expected);

        // nested right
        let mut tokens = tokenize("true ? 1 : 2 ? 3 : 4")
            .filter(|t| t.kind != TokenKind::Whitespace)
            .peekable();
        let ast = expression(&mut tokens).unwrap();
        let expected = Ternary {
            condition: Literal {
                value: True,
                index: 0,
                len: 4,
            }
            .into(),
            left: Literal {
                value: Number(1.0),
                index: 7,
                len: 1,
            }
            .into(),
            right: Ternary {
                condition: Literal {
                    value: Number(2.0),
                    index: 11,
                    len: 1,
                }
                .into(),
                left: Literal {
                    value: Number(3.0),
                    index: 15,
                    len: 1,
                }
                .into(),
                right: Literal {
                    value: Number(4.0),
                    index: 19,
                    len: 1,
                }
                .into(),
                index: 11,
                len: 9,
            }
            .into(),
            index: 0,
            len: 20,
        };

        assert_eq!(ast, expected);

        // nested left
        let mut tokens = tokenize("true ? 1 ? 2 : 3 : 4")
            .filter(|t| t.kind != TokenKind::Whitespace)
            .peekable();
        let ast = expression(&mut tokens).unwrap();
        let expected = Ternary {
            condition: Literal {
                value: True,
                index: 0,
                len: 4,
            }
            .into(),
            left: Ternary {
                condition: Literal {
                    value: Number(1.0),
                    index: 7,
                    len: 1,
                }
                .into(),
                left: Literal {
                    value: Number(2.0),
                    index: 11,
                    len: 1,
                }
                .into(),
                right: Literal {
                    value: Number(3.0),
                    index: 15,
                    len: 1,
                }
                .into(),
                index: 7,
                len: 9,
            }
            .into(),
            right: Literal {
                value: Number(4.0),
                index: 19,
                len: 1,
            }
            .into(),
            index: 0,
            len: 20,
        };

        assert_eq!(ast, expected);
    }
}
