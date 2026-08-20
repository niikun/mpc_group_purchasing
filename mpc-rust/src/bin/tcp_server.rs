use serde_json::Deserializer;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use mpc_rust::secret_sharing::Fp;
use mpc_rust::node::Node;

#[tokio::main]
async fn main()->std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("listening...");
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("connected: {addr}");
        let handle = tokio::spawn(async move {
            let mut reader = BufReader::new(socket);
            let mut line = String::new();
            reader.read_line(&mut line).await;
            let msg:Node = serde_json::from_str(&line).unwrap();
            println!("{:?}",msg);
        });
    }

    Ok(())
}