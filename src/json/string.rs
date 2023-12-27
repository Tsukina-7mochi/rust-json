pub fn escape(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\x08' => result.push_str("\\b"),
            '\x0c' => result.push_str("\\f"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c => result.push(c),
        }
    }
    result
}

pub fn unescape(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('/') => result.push('/'),
                Some('\\') => result.push('\\'),
                Some('b') => result.push('\x08'),
                Some('f') => result.push('\x0c'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('u') => {
                    let mut hex = String::new();
                    for _ in 0..4 {
                        match chars.next() {
                            Some(c) => hex.push(c),
                            None => {
                                result.push('?');
                                return result;
                            }
                        }
                    }
                    match u32::from_str_radix(&hex, 16) {
                        Ok(n) => match char::from_u32(n) {
                            Some(c) => result.push(c),
                            None => result.push('?'),
                        },
                        Err(_) => result.push('?'),
                    }
                }
                Some(c) => result.push(c),
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}
