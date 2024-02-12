use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:11211").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        _ = stream;
        println!("Connection established!");
    }
}
