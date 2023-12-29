use super::util::signed_num_64::SignedNum64;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    BeginArray,
    EndArray,
    BeginObject,
    EndObject,
    NameSeparator,
    ValueSeparator,
    True,
    False,
    Null,
    Number(SignedNum64),
    String(String),
}
