use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};

use mpc_rust::secret_sharing::Fp;
use mpc_rust::node::{Node, Branch};


#[tokio::main]
async fn main() -> std::io::Result<()> {
    // TODO: tcp_server.rs が bind しているアドレス(127.0.0.1:8000)に
    //       TcpStream::connect で接続する
    let mut stream = TcpStream::connect("127.0.0.1:8000").await?;
    let share = [Fp::one();9];
    let branch = Branch::Buyer;
    let test_value = (share, branch);
    let test_json = serde_json::to_string(&test_value)?;
    let msg = format!("{}\n",test_json);

    // TODO: AsyncWriteExt が提供する write_all で stream に書き込む
    stream.write_all(msg.as_bytes()).await?;

    Ok(())
}
