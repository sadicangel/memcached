use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Error, Lines, Read, Write},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

#[derive(Debug)]
struct CacheEntry {
    data: String,
    flags: i32,
    ttl: u32,
}

static CACHE: Lazy<Mutex<HashMap<String, CacheEntry>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    Mutex::new(map)
});

fn main() {
    let listener = TcpListener::bind("127.0.0.1:11211").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => handle_connection(s),
            Err(e) => println!("Connection failed: {e}\r\n"),
        };
    }
}

fn handle_connection(mut stream: TcpStream) {
    let reader = BufReader::new(&mut stream);
    let mut lines = reader.lines();

    let response = match lines.next() {
        Some(result) => match result {
            Ok(command) => handle_command(&mut lines, command),
            Err(error) => {
                stream.write_all(format!("{error}\r\n").as_bytes()).unwrap();
                String::new()
            }
        },
        None => {
            stream.write_all(b"ERROR\r\n").unwrap();
            String::new()
        }
    };

    println!("Response: {response}");

    if response.len() > 0 {
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn handle_command(lines: &mut Lines<BufReader<&mut TcpStream>>, command: String) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.len() == 0 {
        return String::from("ERROR\r\n");
    }

    match parts[0] {
        "set" => match lines.next() {
            Some(result) => match result {
                Ok(data) => handle_set(&parts[1..], data),

                Err(error) => format!("{error}\r\n"),
            },
            None => String::from("CLIENT_ERROR bad data chunk\r\n"),
        },
        "get" => handle_get(&parts[1..]),
        _ => String::new(),
    }
}

fn handle_set(args: &[&str], data: String) -> String {
    if args.len() < 4 && args.len() > 5 {
        return String::from("ERROR invalid set arguments\r\n");
    }

    let key = args[0].to_string();
    let flags = args[1].parse::<i32>().unwrap();
    let ttl = args[2].parse::<u32>().unwrap();
    let length = args[3].parse::<usize>().unwrap();

    if data.len() != length {
        return String::from("CLIENT_ERROR bad data chunk\r\n");
    }

    let entry = CacheEntry { data, flags, ttl };

    CACHE.lock().unwrap().insert(key, entry);

    if args.len() == 4 || args[4] != "noreply" {
        String::from("STORED\r\n")
    } else {
        String::new()
    }
}

fn handle_get(args: &[&str]) -> String {
    let mut result = String::new();
    let cache = CACHE.lock().unwrap();
    for arg in args {
        match cache.get(*arg) {
            Some(entry) => {
                result += &format!("VALUE {} {} {}\r\n", arg, entry.flags, entry.data.len());
                result += &entry.data;
                result += "\r\nEND\r\n"
            }
            None => {}
        }
    }
    result
}
