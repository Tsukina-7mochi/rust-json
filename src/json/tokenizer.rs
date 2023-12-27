use regex::bytes::Regex;

use super::token::IntOrFloatNumber;
use super::token::Token;

pub struct Tokenizer<'a> {
    index: usize,
    text: &'a [u8],
}

impl<'a> Tokenizer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            index: 0,
            text: text.as_bytes(),
        }
    }

    pub fn tokenize(text: &'a str) -> Vec<Token> {
        let tokenizer = Self::new(text);
        let iter = TokenizerIterator { tokenizer };
        iter.collect()
    }

    fn consume_whitespaces(&mut self) -> Option<()> {
        loop {
            let head_char = self.text.get(self.index)?;

            if *head_char == 0x20 || *head_char == 0x09 || *head_char == 0x0a || *head_char == 0x0d
            {
                self.index += 1;
            } else {
                return Some(());
            }
        }
    }

    fn consume_char(&mut self) -> Option<Token> {
        let head_char = self.text.get(self.index)?;
        let token = match *head_char {
            b'[' => Some(Token::BeginArray),
            b']' => Some(Token::EndArray),
            b'{' => Some(Token::BeginObject),
            b'}' => Some(Token::EndObject),
            b':' => Some(Token::NameSeparator),
            b',' => Some(Token::ValueSeparator),
            _ => None,
        };

        if token.is_some() {
            self.index += 1;
        }

        token
    }

    fn consume_bool_and_null(&mut self) -> Option<Token> {
        let sub4 = self.text.get((self.index)..(self.index + 4))?;
        let token = if sub4[0] == b't' && sub4[1] == b'r' && sub4[2] == b'u' && sub4[3] == b'e' {
            Some(Token::True)
        } else if sub4[0] == b'n' && sub4[1] == b'u' && sub4[2] == b'l' && sub4[3] == b'l' {
            Some(Token::Null)
        } else {
            None
        };

        if token.is_some() {
            self.index += 4;
            return token;
        }

        let sub5 = self.text.get((self.index)..(self.index + 5))?;
        return if sub5[0] == b'f'
            && sub5[1] == b'a'
            && sub5[2] == b'l'
            && sub5[3] == b's'
            && sub5[4] == b'e'
        {
            self.index += 5;
            Some(Token::False)
        } else {
            None
        };
    }

    fn consume_string(&mut self) -> Option<Token> {
        let head_char = self.text.get(self.index)?;
        if *head_char != b'"' {
            return None;
        }

        self.index += 1;
        let start = self.index;

        loop {
            let next_char = self.text.get(self.index)?;
            if *next_char == b'"' {
                break;
            } else if *next_char == b'\\' {
                self.index += 1;
            } else if *next_char & 0b11110000 == 0b11110000 {
                // 4 byte UTF-8 chars
                self.index += 3;
            } else if *next_char & 0b11100000 == 0b11100000 {
                // 3 byte UTF-8 chars
                self.index += 2;
            } else if *next_char & 0b11000000 == 0b11000000 {
                // 2 byte UTF-8 chars
                self.index += 1;
            }

            self.index += 1;
        }

        let end = self.index;
        self.index += 1;

        let sub = self.text[start..end].to_owned();
        let value = String::from_utf8(sub).unwrap();
        let value = super::string::unescape(&value);
        Some(Token::String(value))
    }

    fn consume_int_number(&mut self) -> Option<Token> {
        let regex = Regex::new(r"-?(0|[1-9]\d*)").unwrap();
        let match_len = regex
            .captures_at(self.text, self.index)?
            .get(0)
            .and_then(|m| {
                if m.start() == self.index {
                    Some(m.as_bytes().len())
                } else {
                    None
                }
            })?;
        let sub = self.text[self.index..(self.index + match_len)].to_owned();

        let value: i64 = String::from_utf8(sub).unwrap().parse().unwrap();
        let token = Token::Number(IntOrFloatNumber::Integer(value));
        self.index += match_len;

        Some(token)
    }

    fn consume_float_number(&mut self) -> Option<Token> {
        let regex =
            Regex::new(r"-?(0|[1-9]\d*)((\.\d+)([eE][+\-]?\d+)?|(\.\d+)?([eE][+\-]?\d+))").unwrap();
        let match_len = regex
            .captures_at(self.text, self.index)?
            .get(0)
            .and_then(|m| {
                if m.start() == self.index {
                    Some(m.as_bytes().len())
                } else {
                    None
                }
            })?;
        let sub = self.text[self.index..(self.index + match_len)].to_owned();

        let value: f64 = String::from_utf8(sub).unwrap().parse().unwrap();
        let token = Token::Number(IntOrFloatNumber::Float(value));
        self.index += match_len;

        Some(token)
    }

    fn consume_number(&mut self) -> Option<Token> {
        self.consume_float_number()
            .or_else(|| self.consume_int_number())
    }

    fn consume(&mut self) -> Option<Token> {
        self.consume_whitespaces();

        if self.index >= self.text.len() {
            return None;
        }

        self.consume_char()
            .or_else(|| self.consume_bool_and_null())
            .or_else(|| self.consume_string())
            .or_else(|| self.consume_number())
    }
}

struct TokenizerIterator<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Iterator for TokenizerIterator<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokenizer.consume()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_begin_array() {
        let mut tokenizer = Tokenizer::new("[");
        assert_eq!(Some(Token::BeginArray), tokenizer.consume_char());

        let mut tokenizer = Tokenizer::new("[");
        assert_eq!(Some(Token::BeginArray), tokenizer.consume());
    }

    #[test]
    fn consume_end_array() {
        let mut tokenizer = Tokenizer::new("]");
        assert_eq!(Some(Token::EndArray), tokenizer.consume_char());

        let mut tokenizer = Tokenizer::new("]");
        assert_eq!(Some(Token::EndArray), tokenizer.consume());
    }

    #[test]
    fn consume_begin_object() {
        let mut tokenizer = Tokenizer::new("{");
        assert_eq!(Some(Token::BeginObject), tokenizer.consume_char());
        let mut tokenizer = Tokenizer::new("{");
        assert_eq!(Some(Token::BeginObject), tokenizer.consume());
    }

    #[test]
    fn consume_end_object() {
        let mut tokenizer = Tokenizer::new("}");
        assert_eq!(Some(Token::EndObject), tokenizer.consume_char());

        let mut tokenizer = Tokenizer::new("}");
        assert_eq!(Some(Token::EndObject), tokenizer.consume());
    }

    #[test]
    fn consume_name_separator() {
        let mut tokenizer = Tokenizer::new(":");
        assert_eq!(Some(Token::NameSeparator), tokenizer.consume_char());

        let mut tokenizer = Tokenizer::new(":");
        assert_eq!(Some(Token::NameSeparator), tokenizer.consume());
    }

    #[test]
    fn consume_value_separator() {
        let mut tokenizer = Tokenizer::new(",");
        assert_eq!(Some(Token::ValueSeparator), tokenizer.consume_char());

        let mut tokenizer = Tokenizer::new(",");
        assert_eq!(Some(Token::ValueSeparator), tokenizer.consume());
    }

    #[test]
    fn consume_true() {
        let mut tokenizer = Tokenizer::new("true");
        assert_eq!(Some(Token::True), tokenizer.consume_bool_and_null());

        let mut tokenizer = Tokenizer::new("true");
        assert_eq!(Some(Token::True), tokenizer.consume());
    }

    #[test]
    fn consume_false() {
        let mut tokenizer = Tokenizer::new("false");
        assert_eq!(Some(Token::False), tokenizer.consume_bool_and_null());

        let mut tokenizer = Tokenizer::new("false");
        assert_eq!(Some(Token::False), tokenizer.consume());
    }

    #[test]
    fn consume_null() {
        let mut tokenizer = Tokenizer::new("null");
        assert_eq!(Some(Token::Null), tokenizer.consume_bool_and_null());

        let mut tokenizer = Tokenizer::new("null");
        assert_eq!(Some(Token::Null), tokenizer.consume());
    }

    #[cfg(test)]
    mod consume_string {
        use super::*;

        #[test]
        fn string() {
            let mut tokenizer = Tokenizer::new("\"hello\"");
            assert_eq!(
                Some(Token::String(String::from("hello"))),
                tokenizer.consume_string()
            );

            let mut tokenizer = Tokenizer::new("\"hello\"");
            assert_eq!(
                Some(Token::String(String::from("hello"))),
                tokenizer.consume()
            );
        }

        #[test]
        fn string_with_escape() {
            let mut tokenizer = Tokenizer::new("\"hello\\\"\"");
            assert_eq!(
                Some(Token::String(String::from("hello\""))),
                tokenizer.consume_string()
            );

            let mut tokenizer = Tokenizer::new("\"hello\\\"\"");
            assert_eq!(
                Some(Token::String(String::from("hello\""))),
                tokenizer.consume()
            );
        }

        #[test]
        fn string_with_unicode() {
            let mut tokenizer = Tokenizer::new("\"\\u3042\"");
            assert_eq!(
                Some(Token::String(String::from("あ"))),
                tokenizer.consume_string()
            );

            let mut tokenizer = Tokenizer::new("\"\\u3042\"");
            assert_eq!(Some(Token::String(String::from("あ"))), tokenizer.consume());
        }
    }

    #[cfg(test)]
    mod consume_number {
        use super::*;

        #[test]
        fn positive_int_number() {
            let mut tokenizer = Tokenizer::new("123");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Integer(123))),
                tokenizer.consume_int_number()
            );

            let mut tokenizer = Tokenizer::new("123");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Integer(123))),
                tokenizer.consume()
            );
        }

        #[test]
        fn negative_int_number() {
            let mut tokenizer = Tokenizer::new("-123");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Integer(-123))),
                tokenizer.consume_int_number()
            );

            let mut tokenizer = Tokenizer::new("-123");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Integer(-123))),
                tokenizer.consume()
            );
        }

        #[test]
        fn positive_float_number() {
            let mut tokenizer = Tokenizer::new("123.456");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(123.456))),
                tokenizer.consume_float_number()
            );

            let mut tokenizer = Tokenizer::new("123.456");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(123.456))),
                tokenizer.consume()
            );
        }

        #[test]
        fn positive_float_number_starts_with_0() {
            let mut tokenizer = Tokenizer::new("0.456");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(0.456))),
                tokenizer.consume_float_number()
            );

            let mut tokenizer = Tokenizer::new("0.456");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(0.456))),
                tokenizer.consume()
            );
        }

        #[test]
        fn negative_float_number() {
            let mut tokenizer = Tokenizer::new("-123.456");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(-123.456))),
                tokenizer.consume_float_number()
            );

            let mut tokenizer = Tokenizer::new("-123.456");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(-123.456))),
                tokenizer.consume()
            );
        }

        #[test]
        fn positive_float_number_with_exponent() {
            let mut tokenizer = Tokenizer::new("123.456e+10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(123.456e+10))),
                tokenizer.consume_float_number()
            );

            let mut tokenizer = Tokenizer::new("123.456e+10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(123.456e+10))),
                tokenizer.consume()
            );
        }

        #[test]
        fn negative_float_number_with_exponent() {
            let mut tokenizer = Tokenizer::new("-123.456e-10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(-123.456e-10))),
                tokenizer.consume_float_number()
            );

            let mut tokenizer = Tokenizer::new("-123.456e-10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(-123.456e-10))),
                tokenizer.consume()
            );
        }

        #[test]
        fn positive_float_number_with_exponent_without_fractional_part() {
            let mut tokenizer = Tokenizer::new("123e+10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(123e+10))),
                tokenizer.consume_float_number()
            );

            let mut tokenizer = Tokenizer::new("123e+10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(123e+10))),
                tokenizer.consume()
            );
        }

        #[test]
        fn negative_float_number_with_exponent_without_fractional_part() {
            let mut tokenizer = Tokenizer::new("-123e-10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(-123e-10))),
                tokenizer.consume_float_number()
            );

            let mut tokenizer = Tokenizer::new("-123e-10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(-123e-10))),
                tokenizer.consume()
            );
        }

        #[test]
        fn positive_float_number_with_exponent_without_fractional_part_and_plus() {
            let mut tokenizer = Tokenizer::new("123e10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(123e10))),
                tokenizer.consume_float_number()
            );

            let mut tokenizer = Tokenizer::new("123e10");
            assert_eq!(
                Some(Token::Number(IntOrFloatNumber::Float(123e10))),
                tokenizer.consume()
            );
        }
    }

    #[test]
    fn random_sequence() {
        let mut tokenizer =
            Tokenizer::new("  [  ]  {  }  :  ,  true  false  null  123  123.456  \"hello\"  ");
        assert_eq!(Some(Token::BeginArray), tokenizer.consume());
        assert_eq!(Some(Token::EndArray), tokenizer.consume());
        assert_eq!(Some(Token::BeginObject), tokenizer.consume());
        assert_eq!(Some(Token::EndObject), tokenizer.consume());
        assert_eq!(Some(Token::NameSeparator), tokenizer.consume());
        assert_eq!(Some(Token::ValueSeparator), tokenizer.consume());
        assert_eq!(Some(Token::True), tokenizer.consume());
        assert_eq!(Some(Token::False), tokenizer.consume());
        assert_eq!(Some(Token::Null), tokenizer.consume());
        assert_eq!(
            Some(Token::Number(IntOrFloatNumber::Integer(123))),
            tokenizer.consume()
        );
        assert_eq!(
            Some(Token::Number(IntOrFloatNumber::Float(123.456))),
            tokenizer.consume()
        );
        assert_eq!(
            Some(Token::String(String::from("hello"))),
            tokenizer.consume()
        );
    }
}
