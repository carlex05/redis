use std::{fmt::format, io::Read, net::TcpListener, str, thread};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Servidor escuchando en el puerto 6379");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    println!("Nueva conexión: {}", stream.peer_addr().unwrap());
                    let mut buffer = [0; 1024];

                    match stream.read(&mut buffer) {
                        Ok(size) => {
                            // Convertir los bytes leídos en una cadena
                            let received_data = match str::from_utf8(&buffer[..size]) {
                                Ok(v) => v,
                                Err(e) => {
                                    println!("Error al convertir los datos en cadena: {}", e);
                                    return;
                                }
                            };

                            println!("Datos recibidos: {}", received_data);
                            
                        }
                        Err(e) => {
                            println!("Error al leer: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                println!("Error de conexión: {}", e);
            }
        }
    }
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

fn deserialize(param: &String) -> Resp {
    if "$-1\r\n" == param {
        return Resp::BulkString(None);
    }
    let characters: Vec<char> = param.chars().collect();
    let datatype: char = characters[0];
    if datatype == '$' {
        return deserialize_bulk(&characters);
    }else if param.starts_with("*") {
        return deserialize_array(characters, param);
    }
    Resp::BulkString(Some("Hello world!".to_string()))
}

fn deserialize_array(characters: Vec<char>, resp: &String) -> Resp {
    let amount: usize = get_resp_array_size(characters);
    let tokens: Vec<&str> = resp.split("\r\n").collect();
    let mut resps: Vec<String> = Vec::new();
    for index in 1..amount + 1 {
        let resp_type: &str = tokens[index*2-1];
        let resp_data: &str = tokens[index*2];
        resps.push(format!("{resp_type}\r\n{resp_data}"));
    }
    let content: Vec<Resp> = resps.iter().map(|respL| deserialize(respL)).collect();
    Resp::Array(content)
}

fn get_resp_array_size(resp_array: Vec<char>) -> usize {
    let mut amount_str = String::new();
    let mut char_index = 1;
    while resp_array[char_index] <= '9' && resp_array[char_index] >= '0' {
        amount_str.push(resp_array[char_index]);
        char_index += 1;
    }
    amount_str.parse().expect("Problem getting the size of the Vector")
}

fn deserialize_bulk(characters: &Vec<char>) -> Resp {
    let mut amount_str: String = String::new();
    let mut char_index = 1;
    while characters[char_index] <= '9' && characters[char_index] >= '0' {
        amount_str.push(characters[char_index]);
        char_index += 1;
    }
    let amount: usize = amount_str.parse().expect("Problem getting the size of the Bulk");
    let content: String = (&characters[char_index + 2..char_index + 2 + amount]).iter().collect();
    return Resp::BulkString(Some(content));
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
    fn deserialize_bulk_string_null() {
        let result = deserialize(&"$-1\r\n".to_string());
        assert_eq!(result, Resp::BulkString(None));
    }

    #[test]
    fn deserialize_bulk_string_helloworld() {
        let result = deserialize(&"$12\r\nHello world!\r\n".to_string());
        assert_eq!(result, Resp::BulkString(Some("Hello world!".to_string())));
    }

    #[test]
    fn deserialize_bulk_string_empty() {
        let result = deserialize(&"$0\r\n\r\n".to_string());
        assert_eq!(result, Resp::BulkString(Some("".to_string())));
    }

    #[test]
    fn deserialize_array_2bulks_string_not_empty() {
        let result = deserialize(&"*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n".to_string());
        assert_eq!(result, Resp::Array(vec![
            Resp::BulkString(Some("Hello".to_string())),
            Resp::BulkString(Some("World".to_string())),
        ]));
    }

    #[test]
    fn get_resp_array_size_2bulks(){
        let resp: &String = &"*21\r\n$5\r\nHello\r\n$5\r\nWorld\r\n".to_string();
        let characters: Vec<char> = resp.chars().collect();
        let result: usize = get_resp_array_size(characters);
        assert_eq!(21, result);
    }

}
