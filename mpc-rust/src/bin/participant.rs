use std::env;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

use mpc_rust::auction::{set_share_for_send};
use mpc_rust::secret_sharing::Fp;
use mpc_rust::node::Branch;

#[tokio::main]
pub async  fn main()->std::io::Result<()>{
    let args:Vec<String> = env::args().collect();
    let is_buyer:bool = args[1].parse().unwrap();
    let threshold :u64 = args[2].parse().unwrap();
    let quantity :u64 = args[3].parse().unwrap();
    println!("{} : threshold: {}, quantity : {}",is_buyer,threshold, quantity);

    let (shares, branch) = set_share_for_send(threshold, quantity, is_buyer);

    // nodeのアドレス設定
    let addr1 = "127.0.0.1:8000";
    let addr2 = "127.0.0.1:8001";
    let addr3 = "127.0.0.1:8002";
    let addresses:Vec::<&str> = Vec::from([addr1, addr2, addr3]);

    for (share, addr) in shares.iter().zip(addresses.iter()){
        let mut stream = TcpStream::connect(addr).await?;
        let values:([Fp;9],Branch) = (*share, branch);
        let values_serialized = serde_json::to_string(&values).unwrap();
        let msg = format!("{}\n",values_serialized);
        stream.write_all(&msg.into_bytes()).await?;
    }   

Ok(())
}