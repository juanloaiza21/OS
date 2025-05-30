use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;

fn main() {
    let mut stream =
        TcpStream::connect("25.49.153.184:8080").expect("Couldnt connect to TcpServer"); //In connect change the ip
    loop {
        let mut input = String::new();
        input = "Hello there from client:)".to_string();
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read from server");
        stream
            .write(input.as_bytes())
            .expect("Failed to write to server");
        let mut reader = BufReader::new(&stream);
        reader
            .read_until(b'\n', &mut buffer)
            .expect("could not read into buffer");
        println!(
            "{}",
            str::from_utf8(&buffer).expect("Couldnt save buffer as string")
        )
    }
}
