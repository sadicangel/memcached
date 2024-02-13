use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Lines, Write},
    net::{TcpListener, TcpStream},
    ops::Add,
    sync::Mutex,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
struct CacheEntry {
    data: String,
    flags: i32,
    expire: u64,
}

static CACHE: Lazy<Mutex<HashMap<String, CacheEntry>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn main() {
    let listener = TcpListener::bind("127.0.0.1:11211").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => _ = thread::spawn(|| handle_connection(s)),
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
        "set" | "SET" => match lines.next() {
            Some(result) => match result {
                Ok(data) => handle_set(&parts[1..], data),

                Err(error) => format!("{error}\r\n"),
            },
            None => String::from("CLIENT_ERROR bad data chunk\r\n"),
        },
        "get" | "GET" => handle_get(&parts[1..]),
        _ => String::from("ERROR invalid command"),
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

    let expire = if ttl > 0 {
        SystemTime::now()
            .add(Duration::from_secs(ttl.into()))
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    } else {
        0
    };

    let entry = CacheEntry {
        data,
        flags,
        expire,
    };

    CACHE.lock().unwrap().insert(key, entry);

    if args.len() == 4 || args[4] != "noreply" {
        String::from("STORED\r\n")
    } else {
        String::new()
    }
}

fn handle_get(args: &[&str]) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut result = String::new();
    let cache = CACHE.lock().unwrap();

    for arg in args {
        match cache.get(*arg) {
            Some(entry) => {
                if entry.expire == 0 || now <= entry.expire {
                    result += &format!("VALUE {} {} {}\r\n", arg, entry.flags, entry.data.len());
                    result += &entry.data;
                    result += "\r\nEND\r\n"
                }
            }
            None => {}
        }
    }
    result
}
