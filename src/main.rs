use std::fmt::format;

fn main() {
    println!("Hello, world!");
}

#[derive(Debug, PartialEq)]
enum Resp {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<Resp>),
}


fn serialize(resp: &Resp) -> String {
    match resp {
        Resp::BulkString(None) => "$-1\r\n".to_string(),
        Resp::BulkString(Some(contents)) => format!("${}\r\n{}\r\n", contents.len(), contents),
        Resp::Array(elements) => serialize_array(elements),
        Resp::SimpleString(content) => format!("+{}\r\n", content),
        Resp::Error(content) => format!("-{}\r\n", content),
        _ => String::new()
    }
    
}

fn serialize_array(elements: &Vec<Resp>) -> String {
    if elements.is_empty(){
        return "*0\r\n".to_string();
    }
    let serialized_elements = elements.iter() 
        .map(serialize) 
        .collect::<Vec<String>>()
        .join("");
    format!("*{}\r\n{}", elements.len(), serialized_elements)
}

fn deserialize(param: String) -> Resp {
    Resp::BulkString(None)
}

#[cfg(test)]
mod tests {

    use super::*; 

    #[test]
    fn serialize_bulk_string_null() {
        let result = serialize(&Resp::BulkString(None));
        assert_eq!(result, "$-1\r\n");
    }

    #[test]
    fn serialize_bulk_string_helloworld() {
        let result = serialize(&Resp::BulkString(Some("Hello world!".to_string())));
        assert_eq!(result, "$12\r\nHello world!\r\n");
    }

    #[test]
    fn serialize_bulk_string_empty() {
        let result = serialize(&Resp::BulkString(Some("".to_string())));
        assert_eq!(result, "$0\r\n\r\n");
    }

    #[test]
    fn serialize_array_2bulks_strings_notempty() {
        let result = serialize(&Resp::Array(vec![
            Resp::BulkString(Some("Hello".to_string())),
            Resp::BulkString(Some("World".to_string())),
        ]));
        assert_eq!(result, "*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n");
    }

    #[test]
    fn serialize_array_1bulk_string_empty() {
        let result = serialize(&Resp::Array(vec![
            Resp::BulkString(Some("".to_string())),
        ]));
        assert_eq!(result, "*1\r\n$0\r\n\r\n");
    }

    #[test]
    fn serialize_array_empty() {
        let result = serialize(&Resp::Array(vec![]));
        assert_eq!(result, "*0\r\n");
    }

    #[test]
    fn serialize_string_ok() {
        let result = serialize(&&Resp::SimpleString("OK".to_string()));
        assert_eq!(result, "+OK\r\n");
    }

    #[test]
    fn serialize_string_empty() {
        let result = serialize(&&Resp::SimpleString("".to_string()));
        assert_eq!(result, "+\r\n");
    }

    #[test]
    fn serialize_error_empty() {
        let result = serialize(&Resp::Error("".to_string()));
        assert_eq!(result, "-\r\n");
    }

    #[test]
    fn serialize_error_notempty() {
        let result = serialize(&Resp::Error("Error message".to_string()));
        assert_eq!(result, "-Error message\r\n");
    }

    #[test]
    fn deserialize_bulk_string() {
        let result = deserialize("$-1\r\n".to_string());
        assert_eq!(result, Resp::BulkString(None));
    }

    #[test]
    fn deserialize_bulk_string_helloworld() {
        let result = deserialize("$12\r\nHello world!\r\n".to_string());
        assert_eq!(result, Resp::BulkString(Some("Hello world!".to_string())));
    }

}
