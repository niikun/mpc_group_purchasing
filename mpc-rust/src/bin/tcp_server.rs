use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main()->std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("listening...");
    let (socket, addr) = listener.accept().await?;
    println!("connected: {addr}");

    let mut reader = BufReader::new(socket);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    println!("receive: {line}");
    Ok(())
}