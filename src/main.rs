mod json;

use json::parser::Parser;

fn main() {
    let json_data = "
[
    { \"name\": \"Alice\", \"age\": 15 },
    { \"name\": \"Bob\", \"age\": 20 },
    { \"name\": \"Charlie\", \"age\": 25 }
]
    ";

    let obj = Parser::parse(json_data).unwrap();
    for i in 0..3 {
        let map = obj.get_as_array(i).unwrap();
        let name = map.get_as_object("name").unwrap().as_string().unwrap();
        let age = map.get_as_object("age").unwrap().as_i64().unwrap();

        println!("{} name: {}, age: {}", i, name, age);
    }
}
