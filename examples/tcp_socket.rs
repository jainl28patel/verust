use std::net::{TcpListener,TcpStream};

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:4000").unwrap();
    loop {
        let (srtream, sockAddr) = listener.accept().unwrap();
        println!("Connection from {}", sockAddr);
        let _ = srtream.shutdown(std::net::Shutdown::Both);
    }
}