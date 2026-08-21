use std::env;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use serde::{Serialize, Deserialize};

use mpc_rust::secret_sharing::Fp;
use mpc_rust::node::{Node, Branch};
use mpc_rust::auction::{set_share_for_send};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //Buyer 3社の価格セッティング
    let threshold_b1:u64 = 100;
    let threshold_b2:u64 = 110;
    let threshold_b3:u64 = 105;
    //Seller 2社の価格セッティング
    let threshold_s1:u64 = 90;
    let threshold_s2:u64 = 100;
    //Buyer 3社の希望量セッティング    
    let quantity_b1:u64 = 100;
    let quantity_b2:u64 = 200;
    let quantity_b3:u64= 300;
    //Seller 2社の希望量セッティング
    let quantity_s1:u64 = 400;
    let quantity_s2:u64 = 500;

    // shareのセッティング
    let (shares_b1, branch_b1) = set_share_for_send(threshold_b1, quantity_b1, true);
    let (shares_b2, branch_b2) = set_share_for_send(threshold_b2, quantity_b2, true);    
    let (shares_b3, branch_b3) = set_share_for_send(threshold_b3, quantity_b3, true);
    let (shares_s1, branch_s1) = set_share_for_send(threshold_s1, quantity_s1, false);
    let (shares_s2, branch_s2) = set_share_for_send(threshold_s2, quantity_s2, false);
    let shares_all = Vec::from([shares_b1, shares_b2, shares_b3, shares_s1, shares_s2]);
    let branch_all = Vec::from([branch_b1, branch_b2, branch_b3, branch_s1, branch_s2]);
    // ３nodeのアドレス設定
    let addr1 = "127.0.0.1:8000";
    let addr2 = "127.0.0.1:8001";
    let addr3 = "127.0.0.1:8002";
    let addresses:Vec::<&str> = Vec::from([addr1, addr2, addr3]);

    // 各ノードにshareを送信
    for (shares, branch) in shares_all.iter().zip(branch_all.iter()){
        for (share, addr) in shares.iter().zip(addresses.iter()){
            let mut stream = TcpStream::connect(addr).await?;
            let values:([Fp;9], Branch) = (*share, *branch);
            let values_serialized =  serde_json::to_string(&values).unwrap();
            let msg = format!("{}\n",values_serialized);
            stream.write_all(msg.as_bytes()).await?;
        }
    }
    Ok(())
}