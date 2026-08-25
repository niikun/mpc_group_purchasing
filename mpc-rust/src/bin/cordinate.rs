// use std::env;
use tokio::io::BufReader;

use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
// use serde::{Serialize, Deserialize};

// use mpc_rust::secret_sharing::Fp;
use mpc_rust::node::Node;
use mpc_rust::auction::{PriceQuantity, clearing_price};
// use tokio::net::tcp::OwnedReadHalf;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    
    // ３nodeのアドレス設定
    let addr1 = "127.0.0.1:8000";
    let addr2 = "127.0.0.1:8001";
    let addr3 = "127.0.0.1:8002";
    let addresses:Vec::<&str> = Vec::from([addr1, addr2, addr3]);

    let mut total_node_vec = Vec::new();
    for addr in addresses{
        let mut stream = TcpStream::connect(addr).await?;
        let values = "GET";
        let values_serialized = serde_json::to_string(&values).unwrap();
        let msg = format!("{}\n",values_serialized);
        stream.write_all(msg.as_bytes()).await?;
        
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let msg:Node = serde_json::from_str(&line.trim()).unwrap();
        total_node_vec.push(msg);     
    }
    let total_node = Node::add_node(total_node_vec[0].clone(), total_node_vec[1].clone(),total_node_vec[2].clone());
    // println!("Total_node : {:?}", total_node);
    // Nodeを需要側と供給側に分割
    let demand_quantity:PriceQuantity = PriceQuantity { quantities:total_node.buyer_quantities};
    let supply_quantity:PriceQuantity = PriceQuantity { quantities: total_node.seller_quantities};
    // 需要と供給を計算
    match clearing_price(demand_quantity, supply_quantity){
        Some((price, total_quantity)) =>{
            
            println!("{:?},{:?}",price,total_quantity);
        },
        None => {println!("((0,(0, 0, 0, 0, 0))");}
    }
    Ok(())
}