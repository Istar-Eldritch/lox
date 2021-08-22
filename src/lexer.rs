use std::{convert::TryFrom, iter::Peekable};

use self::KeywordKind::*;
use self::LiteralKind::*;
use self::TokenKind::*;

pub fn tokenize(mut code: &str) -> impl Iterator<Item = Token> + Clone + '_ {
    let mut index = 0;
    std::iter::from_fn(move || {
        let token = next_token(code, index);
        index += token.len;
        if token.kind == Eof {
            None
        } else {
            code = &code[token.len..];
            Some(token)
        }
    })
}

fn next_token(code: &str, index: usize) -> Token {
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
        Some(':') => Colon,
        Some('?') => Interrogation,
        Some('/') => {
            if let Some('/') = chars.peek() {
                let (c, _, _) = consume_while(&mut chars, |c| c != '\u{000A}');
                consumed += c;
                Comment
            } else {
                Slash
            }
        }
        Some('*') => Star,
        Some('"') => {
            let (c, terminated, value) = consume_while(&mut chars, |c| c != '"');
            consumed += c;
            // Consume while does not consume the ending character '"', so we have do do it here
            if terminated {
                chars.next();
                consumed += 1;
            }
            Literal(Str { terminated, value })
        }
        Some('!') => {
            if let Some('=') = chars.peek() {
                chars.next();
                consumed += 1;
                NotEquals
            } else {
                Bang
            }
        }
        Some('=') => {
            if let Some(c) = chars.peek() {
                if *c == '=' {
                    chars.next();
                    consumed += 1;
                    Equals
                } else {
                    Assign
                }
            } else {
                Assign
            }
        }
        Some('<') => {
            if let Some(c) = chars.peek() {
                if *c == '=' {
                    chars.next();
                    consumed += 1;
                    LessThanEquals
                } else {
                    LessThan
                }
            } else {
                LessThan
            }
        }
        Some('>') => {
            if let Some(c) = chars.peek() {
                if *c == '=' {
                    chars.next();
                    consumed += 1;
                    GreaterThanEquals
                } else {
                    GreaterThan
                }
            } else {
                GreaterThan
            }
        }
        Some(c) if is_digit(c) => {
            let (s, _, value) = consume_while(&mut chars, is_digit);
            let mut str_number = String::with_capacity(s + 1);
            str_number.push(c);
            str_number.push_str(&value);
            consumed += s;
            let mut foreview = chars.clone();
            if let Some(c) = chars.peek() {
                if *c == '.' {
                    foreview.next();
                    if let Some(c) = foreview.peek() {
                        if is_digit(*c) {
                            chars.next();
                            let (c, _, value) = consume_while(&mut chars, is_digit);
                            str_number.push('.');
                            str_number.push_str(&value);
                            consumed += c + 1;
                        }
                    }
                }
            }
            Literal(Number(str_number.parse().unwrap()))
        }
        Some(c) if is_whitespace(c) => {
            let (c, _, _) = consume_while(&mut chars, is_whitespace);
            consumed += c;
            Whitespace
        }
        Some(c) if is_ident_start(c) => {
            let (c, _, _) = consume_while(&mut chars, is_ident_continue);
            consumed += c;
            let s = &code[..consumed];
            if let Ok(k) = KeywordKind::try_from(s) {
                Keyword(k)
            } else {
                Identifier(s.into())
            }
        }
        Some(_) => Unknown,
        _ => Eof,
    };

    Token::new(token_kind, index, consumed)
}

fn consume_while(
    chars: &mut Peekable<impl Iterator<Item = char>>,
    f: impl Fn(char) -> bool,
) -> (usize, bool, String) {
    let mut consumed = 0;
    let mut terminated = false;
    let mut value = String::with_capacity(8);
    while let Some(c) = chars.peek() {
        if f(*c) {
            let c = chars.next().unwrap();
            value.push(c);
            consumed += 1;
        } else {
            terminated = true;
            break;
        }
    }
    (consumed, terminated, value)
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
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub index: usize,
    pub len: usize,
}

impl Token {
    fn new(kind: TokenKind, index: usize, len: usize) -> Token {
        Token { kind, index, len }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Single-char tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Semicolon,
    Colon,
    Interrogation,

    Minus,
    Plus,
    Slash,
    Star,

    Bang,
    Assign,

    // Equality operators
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,

    // Lexemes
    Comment,
    Identifier(String),
    Literal(LiteralKind),
    Keyword(KeywordKind),

    // Other
    Whitespace,
    Unknown,
    Eof,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralKind {
    Str { terminated: bool, value: String },
    Number(f64),
}
