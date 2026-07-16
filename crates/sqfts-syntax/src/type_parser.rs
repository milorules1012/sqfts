//! Recursive-descent parser for SQFts type expressions (SPEC §6).

use crate::typ::{Brand, Primitive, Type};
use float_ord::FloatOrd;
use thiserror::Error;

/// Error parsing a type expression.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TypeParseError {
    /// Unexpected token / end of input.
    #[error("type parse error at {at}: {message}")]
    Message {
        /// Byte offset into the type text.
        at: usize,
        /// Human-readable message.
        message: String,
    },
}

/// Parse a type expression from `input`. Returns the type and bytes consumed.
pub fn parse_type(input: &str) -> Result<(Type, usize), TypeParseError> {
    let mut p = Parser { src: input, pos: 0 };
    let ty = p.parse_union()?;
    Ok((ty.normalize(), p.pos))
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn remaining(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let mut chars = self.remaining().chars();
        let c = chars.next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.bump();
        }
    }

    fn err(&self, message: impl Into<String>) -> TypeParseError {
        TypeParseError::Message {
            at: self.pos,
            message: message.into(),
        }
    }

    fn parse_union(&mut self) -> Result<Type, TypeParseError> {
        let mut parts = vec![self.parse_array()?];
        loop {
            let saved = self.pos;
            self.skip_ws();
            if self.remaining().starts_with('|') {
                self.bump();
                self.skip_ws();
                parts.push(self.parse_array()?);
            } else {
                self.pos = saved;
                break;
            }
        }
        if parts.len() == 1 {
            Ok(parts.pop().unwrap())
        } else {
            Ok(Type::Union(parts))
        }
    }

    fn parse_array(&mut self) -> Result<Type, TypeParseError> {
        let mut ty = self.parse_atom()?;
        loop {
            // Peek past whitespace for `[]` without consuming trailing spaces
            // into the type span (callers need the space before `=` etc.).
            let saved = self.pos;
            self.skip_ws();
            if self.remaining().starts_with("[]") {
                self.pos += 2;
                ty = Type::ArrayOf(Box::new(ty));
            } else {
                self.pos = saved;
                break;
            }
        }
        Ok(ty)
    }

    fn parse_atom(&mut self) -> Result<Type, TypeParseError> {
        self.skip_ws();
        match self.peek() {
            Some('(') => {
                self.bump();
                let inner = self.parse_union()?;
                self.skip_ws();
                if self.bump() != Some(')') {
                    return Err(self.err("expected ')'"));
                }
                Ok(inner)
            }
            Some('[') => self.parse_tuple(),
            Some('"' | '\'') => self.parse_string_lit(),
            Some('-') | Some('0'..='9') => self.parse_number_lit(),
            Some(c) if is_ident_start(c) => {
                let name = self.parse_ident()?;
                if let Some(p) = Primitive::from_name(&name) {
                    Ok(Type::Primitive(p))
                } else if let Some(b) = Brand::from_name(&name) {
                    Ok(Type::Brand(b))
                } else {
                    Ok(Type::Named(name))
                }
            }
            _ => Err(self.err("expected type")),
        }
    }

    fn parse_string_lit(&mut self) -> Result<Type, TypeParseError> {
        let quote = self.bump().unwrap();
        let mut content = String::new();
        while let Some(c) = self.peek() {
            if c == quote {
                let mut chars = self.remaining().chars();
                chars.next();
                if chars.next() == Some(quote) {
                    content.push(quote);
                    self.bump();
                    self.bump();
                    continue;
                }
                self.bump();
                return Ok(Type::StringLit(content));
            }
            content.push(c);
            self.bump();
        }
        Err(self.err("unterminated string literal"))
    }

    fn parse_number_lit(&mut self) -> Result<Type, TypeParseError> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.bump();
            if !matches!(self.peek(), Some('0'..='9')) {
                return Err(self.err("expected digits after '-'"));
            }
        }
        while matches!(self.peek(), Some('0'..='9')) {
            self.bump();
        }
        if self.peek() == Some('.') {
            self.bump();
            if !matches!(self.peek(), Some('0'..='9')) {
                return Err(self.err("expected digits after '.'"));
            }
            while matches!(self.peek(), Some('0'..='9')) {
                self.bump();
            }
        }
        let text = &self.src[start..self.pos];
        let value: f32 = text
            .parse()
            .map_err(|_| self.err(format!("invalid number literal `{text}`")))?;
        Ok(Type::NumberLit(FloatOrd(value)))
    }

    fn parse_tuple(&mut self) -> Result<Type, TypeParseError> {
        assert_eq!(self.bump(), Some('['));
        self.skip_ws();
        if self.peek() == Some(']') {
            self.bump();
            return Ok(Type::Tuple(vec![]));
        }
        let mut elems = Vec::new();
        loop {
            let ty = self.parse_union()?;
            self.skip_ws();
            let optional = if self.peek() == Some('?') {
                self.bump();
                true
            } else {
                false
            };
            elems.push((ty, optional));
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                    self.skip_ws();
                    if self.peek() == Some(']') {
                        self.bump();
                        break;
                    }
                }
                Some(']') => {
                    self.bump();
                    break;
                }
                _ => return Err(self.err("expected ',' or ']' in tuple")),
            }
        }
        Ok(Type::Tuple(elems))
    }

    fn parse_ident(&mut self) -> Result<String, TypeParseError> {
        let start = self.pos;
        if !matches!(self.peek(), Some(c) if is_ident_start(c)) {
            return Err(self.err("expected identifier"));
        }
        while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
            self.bump();
        }
        Ok(self.src[start..self.pos].to_string())
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typ::{Brand, Primitive, Type};

    #[test]
    fn primitives_and_unions() {
        let (t, n) = parse_type("number | string").unwrap();
        assert_eq!(n, "number | string".len());
        assert_eq!(
            t,
            Type::Union(vec![
                Type::Primitive(Primitive::Number),
                Type::Primitive(Primitive::String)
            ])
        );
    }

    #[test]
    fn arrays_and_tuples() {
        assert_eq!(
            parse_type("string[]").unwrap().0,
            Type::ArrayOf(Box::new(Type::Primitive(Primitive::String)))
        );
        assert_eq!(
            parse_type("[number, number, boolean?]").unwrap().0,
            Type::Tuple(vec![
                (Type::Primitive(Primitive::Number), false),
                (Type::Primitive(Primitive::Number), false),
                (Type::Primitive(Primitive::Boolean), true),
            ])
        );
    }

    #[test]
    fn brands() {
        assert_eq!(
            parse_type("positionATL").unwrap().0,
            Type::Brand(Brand::PositionATL)
        );
    }

    #[test]
    fn string_literal_unions() {
        assert_eq!(
            parse_type("\"west\" | \"east\"").unwrap().0,
            Type::Union(vec![
                Type::StringLit("west".into()),
                Type::StringLit("east".into()),
            ])
        );
        assert_eq!(parse_type("'a'").unwrap().0, Type::StringLit("a".into()));
        assert_eq!(
            parse_type("\"say \"\"hi\"\"\"").unwrap().0,
            Type::StringLit("say \"hi\"".into())
        );
    }

    #[test]
    fn number_literal_unions() {
        assert_eq!(
            parse_type("0 | 1 | 2").unwrap().0,
            Type::Union(vec![
                Type::NumberLit(FloatOrd(0.0)),
                Type::NumberLit(FloatOrd(1.0)),
                Type::NumberLit(FloatOrd(2.0)),
            ])
        );
        assert_eq!(parse_type("-1").unwrap().0, Type::NumberLit(FloatOrd(-1.0)));
    }

    #[test]
    fn literals_in_tuples() {
        assert_eq!(
            parse_type("[\"west\", number]").unwrap().0,
            Type::Tuple(vec![
                (Type::StringLit("west".into()), false),
                (Type::Primitive(Primitive::Number), false),
            ])
        );
    }
}
