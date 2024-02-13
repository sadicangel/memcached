use std::{
    io::{stdin, Read, Write},
    net::TcpStream,
};

fn main() {
    println!("Welcome to Memcached client");

    let mut buffer = String::with_capacity(1024);

    loop {
        buffer.clear();
        let read = stdin().read_line(&mut buffer).unwrap();
        if read > 0 {
            let args: Vec<_> = buffer[..read].split_whitespace().collect();

            if args[0] == "set" || args[0] == "SET" {
                send_set_command(&args[1..]);
            } else if args[0] == "get" || args[0] == "GET" {
                send_get_command(&args[1..]);
            } else {
                println!("Invalid command");
            }
        }
    }
}

fn send_set_command(args: &[&str]) {
    if args.len() != 4 && args.len() != 5 {
        println!("Invalid SET");
        return;
    }

    let key = args[0].to_string();
    let flags = args[1].parse::<i32>().unwrap();
    let ttl = args[2].parse::<u32>().unwrap();
    let length = args[3].parse::<usize>().unwrap();

    let mut buffer = String::with_capacity(1024);
    let read = stdin().read_line(&mut buffer).unwrap();
    if read != length + 2 {
        println!("Invalid data chunk. Expected data length {length}. Got {read}");
        return;
    }
    let content = &buffer[..read];
    send_message(format!("SET {key} {flags} {ttl} {length}\r\n{content}\r\n"));
}

fn send_get_command(args: &[&str]) {
    if args.len() == 0 {
        println!("Invalid GET");
        return;
    }

    let keys = args.join(" ");

    send_message(format!("GET {keys}\r\n"));
}

fn send_message(message: String) {
    let mut stream = TcpStream::connect("127.0.0.1:11211").unwrap();
    stream.write_all(message.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    println!("{response}");
}
