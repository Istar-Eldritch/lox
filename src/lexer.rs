use std::{convert::TryFrom, iter::Peekable};

use self::KeywordKind::*;
use self::LiteralKind::*;
use self::TokenKind::*;

pub fn tokenize(mut code: &str) -> impl Iterator<Item = Token> + '_ {
    std::iter::from_fn(move || {
        let token = next_token(code);
        if token.kind == Eof {
            None
        } else {
            code = &code[token.len..];
            Some(token)
        }
    })
}

fn next_token(code: &str) -> Token {
    let mut chars = code.chars().peekable();
    let mut consumed = 1;
    let token_kind = match chars.next() {
        Some('(') => LeftParen,
        Some(')') => RightParen,
        Some('{') => LeftBrace,
        Some('}') => RightBrace,
        Some(',') => Comma,
        Some('.') => Dot,
        Some('-') => Minus,
        Some('+') => Plus,
        Some(';') => Semicolon,
        Some('/') => {
            if let Some('/') = chars.peek() {
                let (c, _) = consume_while(&mut chars, |c| c != '\u{000A}');
                consumed += c;
                Comment
            } else {
                Slash
            }
        }
        Some('*') => Star,
        Some('"') => {
            let (c, terminated) = consume_while(&mut chars, |c| c != '"');
            consumed += c;
            // Consume while does not consume the ending character '"', so we have do do it here
            if terminated {
                chars.next();
                consumed += 1;
            }
            Literal(Str { terminated })
        }
        Some('!') => {
            if let Some('=') = chars.peek() {
                NotEquals
            } else {
                Bang
            }
        }
        Some('=') => Assign,
        Some(c) if is_digit(c) => {
            let (c, _) = consume_while(&mut chars, is_digit);
            consumed += c;
            let mut foreview = chars.clone();
            if let Some(c) = chars.peek() {
                if *c == '.' {
                    foreview.next();
                    if let Some(c) = foreview.peek() {
                        if is_digit(*c) {
                            chars.next();
                            let (c, _) = consume_while(&mut chars, is_digit);
                            consumed += c + 1;
                        }
                    }
                }
            }
            Literal(Number)
        }
        Some(c) if is_whitespace(c) => {
            let (c, _) = consume_while(&mut chars, is_whitespace);
            consumed += c;
            Whitespace
        }
        Some(c) if is_ident_start(c) => {
            let (c, _) = consume_while(&mut chars, is_ident_continue);
            consumed += c;
            let s = &code[..consumed];
            if let Ok(k) = KeywordKind::try_from(s) {
                Keyword(k)
            } else {
                Identifier
            }
        }
        Some(_) => Unknown,
        _ => Eof,
    };
    Token::new(token_kind, consumed)
}

fn consume_while(
    chars: &mut Peekable<impl Iterator<Item = char>>,
    f: impl Fn(char) -> bool,
) -> (usize, bool) {
    let mut consumed = 0;
    let mut terminated = false;
    while let Some(c) = chars.peek() {
        if f(*c) {
            chars.next();
            consumed += 1;
        } else {
            terminated = true;
            break;
        }
    }
    (consumed, terminated)
}

pub fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

/// Shamefully stole this piece of code from the Rust language lexer implementation
/// True if `c` is considered a whitespace according to Rust language definition.
/// See [Rust language reference](https://doc.rust-lang.org/reference/whitespace.html)
/// for definitions of these classes.
pub fn is_whitespace(c: char) -> bool {
    // This is Pattern_White_Space.
    //
    // Note that this set is stable (ie, it doesn't change with different
    // Unicode versions), so it's ok to just hard-code the values.

    matches!(
        c,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

/// True if `c` is valid as a first character of an identifier.
pub fn is_ident_start(c: char) -> bool {
    // This is XID_Start OR '_' (which formally is not a XID_Start).
    c == '_' || unicode_xid::UnicodeXID::is_xid_start(c)
}

pub fn is_ident_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}

/// It doesn't contain information about data that has been parsed,
/// only the type of the token and its size.
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
}

impl Token {
    fn new(kind: TokenKind, len: usize) -> Token {
        Token { kind, len }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    // Utils
    Whitespace,

    // Single-char tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    Assign,

    NotEquals,

    Comment,
    Identifier,
    Literal(LiteralKind),
    Keyword(KeywordKind),
    Unknown,
    Eof,
}

#[derive(Debug, PartialEq)]
pub enum KeywordKind {
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

impl TryFrom<&str> for KeywordKind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "and" => Ok(And),
            "class" => Ok(Class),
            "else" => Ok(Else),
            "false" => Ok(False),
            "for" => Ok(For),
            "fun" => Ok(Fun),
            "if" => Ok(If),
            "nil" => Ok(Nil),
            "or" => Ok(Or),
            "print" => Ok(Print),
            "return" => Ok(Return),
            "super" => Ok(Super),
            "this" => Ok(This),
            "true" => Ok(True),
            "var" => Ok(Var),
            "while" => Ok(While),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LiteralKind {
    Str { terminated: bool },
    Number,
}
