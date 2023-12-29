use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum ParserErrorKind {
    UnexpectedToken,
    UnexpectedEOF,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParserError {
    kind: ParserErrorKind,
}

impl ParserError {
    pub fn new(kind: ParserErrorKind) -> Self {
        ParserError { kind }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl std::error::Error for ParserError {}
