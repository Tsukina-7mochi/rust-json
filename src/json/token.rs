#[derive(Debug, PartialEq)]
pub enum IntOrFloatNumber {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
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
    Number(IntOrFloatNumber),
    String(String),
}
