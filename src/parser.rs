use std::{
    convert::{TryFrom, TryInto},
    iter::Peekable,
};

use crate::ast;
use crate::lexer;

impl TryFrom<lexer::Token> for ast::BinOp {
    type Error = String; // Token is not a valid BinOp
    fn try_from(t: lexer::Token) -> Result<Self, Self::Error> {
        use ast::BinOp;
        use lexer::TokenKind;
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
        use lexer::TokenKind;
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
) -> Result<ast::Expr, String> {
    expression(tokens)
}

fn expression(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, String> {
    comma(tokens)
}

fn comma(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, String> {
    let mut expr = equality(tokens)?;
    while matches_any(tokens, vec![lexer::TokenKind::Comma]) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into()?;
        let right = equality(tokens)?;
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
        };
    }
    Ok(expr)
}

fn equality(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, String> {
    use crate::lexer::TokenKind::*;
    let mut expr = comparison(tokens)?;
    while matches_any(tokens, vec![NotEquals, Equals]) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into()?;
        let right: ast::Expr = comparison(tokens)?;
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
        };
    }
    Ok(expr)
}

fn comparison(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, String> {
    use crate::lexer::TokenKind::*;
    let mut expr = term(tokens)?;
    while matches_any(
        tokens,
        vec![GreaterThan, GreaterThanEquals, LessThan, LessThanEquals],
    ) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into()?;
        let right: ast::Expr = term(tokens)?;
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
        };
    }
    Ok(expr)
}

fn term(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, String> {
    use crate::lexer::TokenKind::*;
    let mut expr = factor(tokens)?;
    while matches_any(tokens, vec![Minus, Plus]) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into()?;
        let right: ast::Expr = factor(tokens)?;
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
        };
    }
    Ok(expr)
}

fn factor(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, String> {
    use crate::lexer::TokenKind::*;
    let mut expr = unary(tokens)?;
    while matches_any(tokens, vec![Slash, Star]) {
        let operator: ast::BinOp = tokens.next().unwrap().try_into()?;
        let right: ast::Expr = unary(tokens)?;
        expr = ast::Expr::Binary {
            left: expr.into(),
            operator,
            right: right.into(),
        };
    }
    Ok(expr)
}

fn unary(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, String> {
    use crate::lexer::TokenKind::*;
    if matches_any(tokens, vec![Bang, Minus]) {
        let operator: ast::UnaryOp = tokens.next().unwrap().try_into()?;
        let right = unary(tokens)?;
        Ok(ast::Expr::Unary {
            operator,
            right: right.into(),
        })
    } else {
        primary(tokens)
    }
}

fn primary(
    tokens: &mut Peekable<impl Iterator<Item = lexer::Token> + Clone>,
) -> Result<ast::Expr, String> {
    use crate::lexer::{KeywordKind::*, LiteralKind::*, TokenKind::*};
    if let Some(t) = tokens.next() {
        let expr = match t.kind {
            Keyword(True) => ast::Expr::Literal(ast::Literal::True),
            Keyword(False) => ast::Expr::Literal(ast::Literal::False),
            Keyword(Nil) => ast::Expr::Literal(ast::Literal::Nil),
            Literal(k) => match k {
                Number(n) => ast::Expr::Literal(ast::Literal::Number(n)),
                Str {
                    terminated: _,
                    value,
                } => ast::Expr::Literal(ast::Literal::Str(value)),
            },
            LeftParen => {
                let expr = expression(tokens)?;
                if let Some(t) = tokens.next() {
                    if t.kind == RightParen {
                        ast::Expr::Grouping(expr.into())
                    } else {
                        Err(format!(
                            "The token {:?} was not expected, a ')' was expected",
                            t.kind,
                        ))?
                    }
                } else {
                    Err(String::from("Expected ')' after grouped expression"))?
                }
            }
            tk => Err(format!(
                "lexer::Token \"{:?}\" at {} does not match a valid expression",
                tk, t.len
            ))?,
        };
        Ok(expr)
    } else {
        Err(String::from("The expression is does not have a leaf node"))
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
    use super::parse;
    use crate::ast::{BinOp::*, Expr::*, Literal::*};
    use crate::lexer::{tokenize, TokenKind};

    #[test]
    fn parse_comma_operator() {
        let mut tokens = tokenize("1,2,3")
            .filter(|t| t.kind != TokenKind::Whitespace)
            .peekable();
        let ast = parse(&mut tokens).unwrap();
        let expected = Binary {
            left: Binary {
                left: Literal(Number(1.0)).into(),
                operator: Comma,
                right: Literal(Number(2.0)).into(),
            }
            .into(),
            operator: Comma,
            right: Literal(Number(3.0)).into(),
        };

        assert_eq!(ast, expected);
    }
}
