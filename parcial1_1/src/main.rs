//Import module from Rust libraries
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .expect("Failed reading from client");
    let request = String::from_utf8_lossy(&buffer[..]); //Convert data on buffer into utf-8
    println!("Received request: {}", request);
    let response = "Hello, Client!".as_bytes();
    stream
        .write_all(&response)
        .expect("Failed to write response");
}

//Entry point
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
}
