use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // TODO: tcp_server.rs が bind しているアドレス(127.0.0.1:8000)に
    //       TcpStream::connect で接続する
    let mut stream = TcpStream::connect("127.0.0.1:8000").await?;

    // TODO: サーバーは read_line で1行読み取っているので、
    //       送るメッセージの末尾に "\n" を付ける
    let msg = "Hello world\n";

    // TODO: AsyncWriteExt が提供する write_all で stream に書き込む
    stream.write_all(msg.as_bytes()).await?;

    Ok(())
}
