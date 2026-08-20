use std::sync::{Arc, Mutex};
use serde_json::Deserializer;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use mpc_rust::secret_sharing::Fp;
use mpc_rust::node::{Node, Branch};

#[tokio::main]
async fn main()->std::io::Result<()>{
    let node:Arc<Mutex<Node>> = Arc::new(Mutex::new(Node::new()));
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("listening...");
    loop {
        let _node:Arc<Mutex<Node>> = Arc::clone(&node);
        let (socket, addr) = listener.accept().await?;
        println!("connected: {addr}");
        let handle = tokio::spawn(async move {
            let mut reader = BufReader::new(socket);
            let mut line = String::new();
            reader.read_line(&mut line).await;
            let messages:([Fp;9],Branch) = serde_json::from_str(&line).unwrap();
            let mut guard = _node.lock().unwrap();
            let (share, branch) = messages;
            *guard = guard.add_share(share, branch);
            println!("{:?}",guard);
        });
    }

    Ok(())
}