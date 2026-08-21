use std::env;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use mpc_rust::secret_sharing::Fp;
use mpc_rust::node::{Node, Branch};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args:Vec<String> = env::args().collect();
    if args.len() <= 1{
        eprintln!("Usage: {} <port> <port>...", args[0]);
        std::process::exit(1);
    }
    let addresses = args.iter().skip(1)
        .map(|s| format!("127.0.0.1:{}", s.parse::<u16>().expect("invalid port number")))
        .collect::<Vec<String>>(); 

    let node: Arc<Mutex<Node>> = Arc::new(Mutex::new(Node::new()));

    let listener = TcpListener::bind(&addresses[0]).await?;
    println!("listening...");

    loop {
        let _node: Arc<Mutex<Node>> = Arc::clone(&node);
        let (socket, addr) = listener.accept().await?;
        println!("connected: {addr}");

        tokio::spawn(async move {
            let mut reader = BufReader::new(socket);

            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        // EOF: クライアントが切断
                        println!("disconnected: {addr}");
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("read error from {addr}: {e}");
                        break;
                    }
                }

                let messages: ([Fp; 9], Branch) = match serde_json::from_str(line.trim()) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("parse error from {addr}: {e}");
                        continue; // このメッセージだけスキップして接続は継続
                    }
                };

                let (share, branch) = messages;

                let mut guard = match _node.lock() {
                    Ok(g) => g,
                    Err(poisoned) => {
                        eprintln!("mutex poisoned, recovering");
                        poisoned.into_inner()
                    }
                };

                *guard = guard.add_share(share, branch);
                println!("{:?}", *guard);
            }
        });
    }
}