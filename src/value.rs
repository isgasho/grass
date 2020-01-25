#![allow(dead_code, unused_variables)]
use std::fmt::{self, Display};
use std::iter::{Iterator, Peekable};
use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::color::Color;
use crate::common::{Keyword, Op, Scope, Symbol};
use crate::units::Unit;
use crate::utils::{deref_variable, devour_whitespace_or_comment, eat_interpolation};
use crate::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Dimension {
    val: u64,
}

impl Add for Dimension {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Dimension {
            val: self.val + other.val,
        }
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListSeparator {
    Space,
    Comma,
}

impl ListSeparator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Space => " ",
            Self::Comma => ", ",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Space => "space",
            Self::Comma => "comma",
        }
    }
}

impl Display for ListSeparator {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Space => write!(f, " "),
            Self::Comma => write!(f, ", "),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum QuoteKind {
    Single,
    Double,
    None,
}

impl QuoteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "'",
            Self::Double => "\"",
            Self::None => "",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Value {
    Important,
    True,
    False,
    Null,
    Dimension(Dimension, Unit),
    List(Vec<Value>, ListSeparator),
    Color(Color),
    Paren(Box<Value>),
    Ident(String, QuoteKind),
}

impl Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Self::Important => todo!(),
            Self::True => todo!(),
            Self::False => todo!(),
            Self::Null => todo!(),
            Self::Dimension(num, unit) => match other {
                Self::Dimension(num2, unit2) => Value::Dimension(num + num2, unit),
                _ => todo!(),
            },
            Self::List(..) => todo!(),
            Self::Color(..) => todo!(),
            Self::Paren(..) => todo!(),
            Self::Ident(..) => todo!(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Important => write!(f, "!important"),
            Self::Dimension(num, unit) => write!(f, "{}{}", num, unit),
            Self::List(vals, sep) => write!(
                f,
                "{}",
                vals.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(sep.as_str())
            ),
            Self::Color(c) => write!(f, "{}", c),
            Self::Paren(val) => write!(f, "({})", val),
            Self::Ident(val, kind) => write!(f, "{}{}{}", kind.as_str(), val, kind.as_str()),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Null => write!(f, "null"),
        }
    }
}

impl Value {
    pub fn is_true(&self) -> bool {
        todo!()
    }

    pub fn unquote(&mut self) -> &mut Self {
        todo!()
    }

    pub fn from_tokens<I: Iterator<Item = Token>>(
        toks: &mut Peekable<I>,
        scope: &Scope,
    ) -> Option<Self> {
        let left = Self::_from_tokens(toks, scope)?;
        let whitespace = devour_whitespace_or_comment(toks);
        let next = match toks.peek() {
            Some(x) => x,
            None => return Some(left),
        };
        match next.kind {
            TokenKind::Symbol(Symbol::Comma) => {
                toks.next();
                devour_whitespace_or_comment(toks);
                let right = match Self::from_tokens(toks, scope) {
                    Some(x) => x,
                    None => return Some(left),
                };
                Some(Value::List(vec![left, right], ListSeparator::Comma))
            }
            TokenKind::Symbol(Symbol::Plus) => {
                toks.next();
                devour_whitespace_or_comment(toks);
                let right = match Self::from_tokens(toks, scope) {
                    Some(x) => x,
                    None => return Some(left),
                };
                Some(left + right)
            }
            _ if whitespace => {
                devour_whitespace_or_comment(toks);
                let right = match Self::from_tokens(toks, scope) {
                    Some(x) => x,
                    None => return Some(left),
                };
                Some(Value::List(vec![left, right], ListSeparator::Space))
            }
            _ => {
                dbg!(&next.kind);
                todo!("unimplemented token in value")
            }
        }
    }

    fn _from_tokens<I: Iterator<Item = Token>>(
        toks: &mut Peekable<I>,
        scope: &Scope,
    ) -> Option<Self> {
        let kind = if let Some(tok) = toks.next() {
            tok.kind
        } else {
            return None;
        };
        match kind {
            TokenKind::Number(val) => {
                let unit = if let Some(tok) = toks.peek() {
                    match tok.kind.clone() {
                        TokenKind::Ident(i) => {
                            toks.next();
                            Unit::from(&i)
                        }
                        TokenKind::Symbol(Symbol::Percent) => {
                            toks.next();
                            Unit::Percent
                        }
                        _ => Unit::None,
                    }
                } else {
                    Unit::None
                };
                Some(Value::Dimension(
                    Dimension {
                        val: val.parse().unwrap(),
                    },
                    unit,
                ))
            }
            TokenKind::Ident(mut s) => {
                while let Some(tok) = toks.peek() {
                    match tok.kind.clone() {
                        TokenKind::Interpolation => {
                            toks.next();
                            s.push_str(
                                &eat_interpolation(toks, scope)
                                    .iter()
                                    .map(|x| x.kind.to_string())
                                    .collect::<String>(),
                            )
                        }
                        TokenKind::Ident(ref i) => {
                            toks.next();
                            s.push_str(i)
                        }
                        _ => break,
                    }
                }
                Some(Value::Ident(s, QuoteKind::None))
            }
            TokenKind::Symbol(Symbol::DoubleQuote) => {
                let mut s = String::new();
                while let Some(tok) = toks.next() {
                    match tok.kind {
                        TokenKind::Symbol(Symbol::DoubleQuote) => break,
                        _ => {}
                    }
                    s.push_str(&tok.kind.to_string());
                }
                Some(Value::Ident(s, QuoteKind::Double))
            }
            TokenKind::Symbol(Symbol::SingleQuote) => {
                let mut s = String::new();
                while let Some(tok) = toks.next() {
                    match tok.kind {
                        TokenKind::Symbol(Symbol::SingleQuote) => break,
                        _ => {}
                    }
                    s.push_str(&tok.kind.to_string());
                }
                Some(Value::Ident(s, QuoteKind::Single))
            }
            TokenKind::Variable(ref v) => {
                Value::from_tokens(&mut deref_variable(v, scope).into_iter().peekable(), scope)
            }
            TokenKind::Interpolation => {
                let mut s = eat_interpolation(toks, scope)
                    .iter()
                    .map(|x| x.kind.to_string())
                    .collect::<String>();
                while let Some(tok) = toks.peek() {
                    match tok.kind.clone() {
                        TokenKind::Interpolation => {
                            toks.next();
                            s.push_str(
                                &eat_interpolation(toks, scope)
                                    .iter()
                                    .map(|x| x.kind.to_string())
                                    .collect::<String>(),
                            )
                        }
                        TokenKind::Ident(ref i) => {
                            toks.next();
                            s.push_str(i)
                        }
                        _ => break,
                    }
                }
                Some(Value::Ident(s, QuoteKind::None))
            }
            TokenKind::Keyword(Keyword::Important) => Some(Value::Important),
            _ => None,
        }
    }
}