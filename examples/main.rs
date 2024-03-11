use verust::runtime::{executor, timerfuture};

use tokio::io::{self, AsyncWriteExt};
use tokio::fs::File;

async fn say_hello() {
    // println!("hello");
    // timerfuture::TimerFuture::new(Duration::new(2, 0)).await;
    // println!("world");

    // executor::spawn(async {
    //     println!("I can spawn like this too");
    // })
    // let listner = TcpListener::bind("127.0.0.1:4000").await.unwrap();
    // loop {
    //     let (mut stream, sockAddr) = listner.accept().unwrap();
    //     println!("[1] : Connection from {}", sockAddr);
    //     let _ = stream.shutdown(std::net::Shutdown::Both);
    //     // break;
    // }
    // executor::spawn(async {
    //     println!("I can spawn like this too");
    // })
    let mut file = File::create("foo.txt").await.unwrap();
    file.write_all(b"some bytes").await.unwrap();
    file.flush().await.unwrap();
}

async fn say_hello2() {
    // println!("hello2");
    // timerfuture::TimerFuture::new(Duration::new(2, 0)).await;
    // println!("world2");
    // let listner = TcpListener::bind("127.0.0.1:5000").unwrap();
    // loop {
    //     let (mut stream, sockAddr) = listner.accept().unwrap();
    //     println!("[2] : Connection from {}", sockAddr);
    //     let _ = stream.shutdown(std::net::Shutdown::Both);
    //     // break;
    // }
}

fn main() {
    let exec = executor::Executor::new();

    exec.spawn(say_hello());
    // exec.spawn(say_hello2());

    exec.run();
}
