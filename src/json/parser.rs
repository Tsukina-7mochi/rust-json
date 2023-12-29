use std::collections::HashMap;
use std::iter::Peekable;

use super::json_value::JSONValue;
use super::parser_error::{ParserError, ParserErrorKind};
use super::token::Token;
use super::tokenizer::Tokenizer;

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    iter: Peekable<std::slice::Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            iter: tokens.iter().peekable(),
        }
        .to_owned()
    }

    pub fn parse(text: &str) -> Result<JSONValue, ParserError> {
        let tokens = Tokenizer::tokenize(text);
        let mut parser = Parser::new(&tokens);

        parser.parse_value().and_then(|token| {
            if parser.iter.peek().is_none() {
                Ok(token)
            } else {
                Err(ParserError::new(ParserErrorKind::UnexpectedToken))
            }
        })
    }

    fn consume_token(&mut self, token: Token) -> Result<&Token, ParserError> {
        (self.iter.next())
            .filter(|v| **v == token)
            .ok_or(ParserError::new(ParserErrorKind::UnexpectedToken))
    }

    fn parse_key_value_pair(&mut self) -> Result<(String, JSONValue), ParserError> {
        let key = (self.iter.next())
            .and_then(|v| match v {
                Token::String(val) => Some(val.to_owned()),
                _ => None,
            })
            .ok_or(ParserError::new(ParserErrorKind::UnexpectedToken))?;

        self.consume_token(Token::NameSeparator)?;

        let value = self.parse_value()?;

        Ok((key, value))
    }

    fn parse_object(&mut self) -> Result<JSONValue, ParserError> {
        let mut contents: HashMap<String, JSONValue> = HashMap::new();

        self.consume_token(Token::BeginObject)?;

        if let Some(next) = self.iter.peek() {
            if **next != Token::EndObject {
                let next_entry = self.parse_key_value_pair()?;
                contents.insert(next_entry.0, next_entry.1);

                loop {
                    if let Some(Token::ValueSeparator) = self.iter.peek() {
                        self.iter.next();
                    } else {
                        break;
                    }

                    let next_entry = self.parse_key_value_pair()?;
                    contents.insert(next_entry.0, next_entry.1);
                }
            }
        }

        self.consume_token(Token::EndObject)?;

        Ok(JSONValue::Object(contents))
    }

    fn parse_array(&mut self) -> Result<JSONValue, ParserError> {
        let mut contents: Vec<JSONValue> = Vec::new();

        self.consume_token(Token::BeginArray)?;

        if let Some(next) = self.iter.peek() {
            if **next != Token::EndArray {
                let next_val = self.parse_value()?;
                contents.push(next_val);

                loop {
                    if let Some(Token::ValueSeparator) = self.iter.peek() {
                        self.iter.next();
                    } else {
                        break;
                    }

                    let next_val = self.parse_value()?;
                    contents.push(next_val);
                }
            }
        }

        self.consume_token(Token::EndArray)?;

        Ok(JSONValue::Array(contents))
    }

    fn parse_value(&mut self) -> Result<JSONValue, ParserError> {
        if let Some(next) = self.iter.peek() {
            match next {
                Token::True => {
                    self.iter.next();
                    Ok(JSONValue::True)
                }
                Token::False => {
                    self.iter.next();
                    Ok(JSONValue::False)
                }
                Token::Null => {
                    self.iter.next();
                    Ok(JSONValue::Null)
                }
                Token::Number(val) => {
                    self.iter.next();
                    Ok(JSONValue::Number(val.clone()))
                }
                Token::String(val) => {
                    self.iter.next();
                    Ok(JSONValue::String(val.clone()))
                }
                Token::BeginArray => self.parse_array(),
                Token::BeginObject => self.parse_object(),
                _ => Err(ParserError::new(ParserErrorKind::UnexpectedToken)),
            }
        } else {
            Err(ParserError::new(ParserErrorKind::UnexpectedEOF))
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::util::signed_num_64::SignedNum64;
    use super::*;

    #[test]
    fn value_true() {
        assert_eq!(Ok(JSONValue::True), Parser::parse("true"));
    }

    #[test]
    fn value_false() {
        assert_eq!(Ok(JSONValue::False), Parser::parse("false"));
    }

    #[test]
    fn value_null() {
        assert_eq!(Ok(JSONValue::Null), Parser::parse("null"));
    }

    #[test]
    fn value_number() {
        assert_eq!(
            Ok(JSONValue::Number(SignedNum64::Integer(123))),
            Parser::parse("123")
        );
        assert_eq!(
            Ok(JSONValue::Number(SignedNum64::Integer(-123))),
            Parser::parse("-123")
        );
        assert_eq!(
            Ok(JSONValue::Number(SignedNum64::Float(123.456))),
            Parser::parse("123.456")
        );
        assert_eq!(
            Ok(JSONValue::Number(SignedNum64::Float(-123.456))),
            Parser::parse("-123.456")
        );
    }

    #[test]
    fn value_string() {
        assert_eq!(
            Ok(JSONValue::String("hello".to_string())),
            Parser::parse("\"hello\"")
        );
    }

    #[test]
    fn value_array() {
        assert_eq!(
            Ok(JSONValue::Array(vec![
                JSONValue::Number(SignedNum64::Integer(1)),
                JSONValue::Number(SignedNum64::Integer(2)),
                JSONValue::Number(SignedNum64::Integer(3)),
            ])),
            Parser::parse("[1, 2, 3]")
        );
    }

    #[test]
    fn value_array_mixed() {
        assert_eq!(
            Ok(JSONValue::Array(vec![
                JSONValue::Number(SignedNum64::Integer(1)),
                JSONValue::String("abc".to_string()),
                JSONValue::True,
            ])),
            Parser::parse("[1, \"abc\", true]")
        );
    }

    #[test]
    fn value_array_empty() {
        assert_eq!(Ok(JSONValue::Array(vec![])), Parser::parse("[]"));
    }

    #[test]
    fn value_array_nested() {
        assert_eq!(
            Ok(JSONValue::Array(vec![
                JSONValue::Array(vec![JSONValue::Number(SignedNum64::Integer(1))]),
                JSONValue::Array(vec![
                    JSONValue::Number(SignedNum64::Integer(2)),
                    JSONValue::Number(SignedNum64::Integer(3))
                ])
            ])),
            Parser::parse("[[1], [2, 3]]")
        );
    }

    #[test]
    fn value_object() {
        let mut map: HashMap<String, JSONValue> = HashMap::new();
        map.insert("a".to_string(), JSONValue::Number(SignedNum64::Integer(0)));
        map.insert("b".to_string(), JSONValue::True);
        map.insert("c".to_string(), JSONValue::Null);

        assert_eq!(
            Ok(JSONValue::Object(map)),
            Parser::parse("{\"a\": 0, \"b\": true, \"c\": null}")
        );
    }

    #[test]
    fn value_object_empty() {
        assert_eq!(Ok(JSONValue::Object(HashMap::new())), Parser::parse("{}"));
    }

    #[test]
    fn value_object_nested() {
        let mut inner_map: HashMap<String, JSONValue> = HashMap::new();
        inner_map.insert("b".to_string(), JSONValue::Number(SignedNum64::Integer(0)));

        let mut map: HashMap<String, JSONValue> = HashMap::new();
        map.insert("a".to_string(), JSONValue::Object(inner_map));

        assert_eq!(
            Ok(JSONValue::Object(map)),
            Parser::parse("{\"a\": {\"b\": 0}}")
        );
    }
}
