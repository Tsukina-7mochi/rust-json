use std::collections::HashMap;

use super::util::signed_num_64::SignedNum64;

#[derive(Debug, PartialEq, Clone)]
pub enum JSONValue {
    True,
    False,
    Null,
    Object(HashMap<String, JSONValue>),
    Array(Vec<JSONValue>),
    Number(SignedNum64),
    String(String),
}

impl JSONValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JSONValue::True => Some(true),
            JSONValue::False => Some(false),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        *self == JSONValue::Null
    }

    pub fn as_number(&self) -> Option<SignedNum64> {
        match self {
            JSONValue::Number(num) => Some(num.to_owned()),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JSONValue::Number(SignedNum64::Integer(num)) => Some(*num),
            JSONValue::Number(SignedNum64::Float(num)) => Some(*num as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JSONValue::Number(SignedNum64::Integer(num)) => Some(*num as f64),
            JSONValue::Number(SignedNum64::Float(num)) => Some(*num),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            JSONValue::String(val) => Some(val.to_owned()),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            JSONValue::Array(_) => true,
            _ => false,
        }
    }

    pub fn get_as_array(&self, index: usize) -> Option<&JSONValue> {
        (match self {
            JSONValue::Array(arr) => Some(arr),
            _ => None,
        })
        .and_then(|arr| arr.get(index))
    }

    pub fn is_object(&self) -> bool {
        match self {
            JSONValue::Object(_) => true,
            _ => false,
        }
    }

    pub fn get_as_object(&self, key: &str) -> Option<&JSONValue> {
        (match self {
            JSONValue::Object(obj) => Some(obj),
            _ => None,
        })
        .and_then(|obj| obj.get(key))
    }
}
