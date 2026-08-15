mod secret_sharing;
mod auction;
mod node;

use tokio::sync::mpsc;
use secret_sharing::Fp;
use node::{Node, Branch};

#[tokio::main]
async fn main() {
    // TODO 1: (sender0, receiver0), (sender1, receiver1), (sender2, receiver2) の
    //   3組のchannelを作る(型は前回と同じ ([Fp;9], Branch))
    let (sender0, mut receiver0) = mpsc::channel::<([Fp;9], Branch)>(32);
    let (sender1, mut receiver1) = mpsc::channel::<([Fp;9], Branch)>(32);
    let (sender2, mut receiver2) = mpsc::channel::<([Fp;9], Branch)>(32);

    // TODO 2: 3つの受信タスクをspawnする(中身は前回と全く同じロジックを3回)
    //   → handle0, handle1, handle2 
    let handle0 = tokio::spawn(async move{
        let mut node = Node::new();
        while let Some((share, branch)) = receiver0.recv().await {
            node = node.add_share(share, branch);
        }
        node
    });
    let handle1 = tokio::spawn(async move {
        let mut node = Node::new();
        while let Some((share, branch)) = receiver1.recv().await {
            node = node.add_share(share, branch);
        }
        node
    });
    let handle2 = tokio::spawn(async move{
        let mut node = Node::new();
        while let Some((share, branch)) = receiver2.recv().await{
            node = node.add_share(share, branch);
        }
        node
    });

    // TODO 3: 各senderに適当なシェアを1〜2回ずつ送る
    //   (例: sender0.send(([Fp::new(10);9], Branch::Buyer)).await; など。値は何でもいい)
    sender0.send(([Fp::new(10);9],Branch::Buyer)).await;
    sender0.send(([Fp::new(20);9],Branch::Seller)).await;
    sender0.send(([Fp::new(30);9],Branch::Buyer)).await;    

    sender1.send(([Fp::new(30);9],Branch::Seller)).await;    
    sender1.send(([Fp::new(30);9],Branch::Seller)).await;    
    sender0.send(([Fp::new(30);9],Branch::Buyer)).await;

    sender2.send(([Fp::new(10);9],Branch::Buyer)).await;
    sender2.send(([Fp::new(20);9],Branch::Seller)).await;
    sender2.send(([Fp::new(30);9],Branch::Buyer)).await;    
    // TODO 4: sender0, sender1, sender2 をそれぞれ drop してチャネルを閉じる
    drop(sender0);
    drop(sender1); 
    drop(sender2);   
    // TODO 5: handle0.await.unwrap() などで node0, node1, node2 を回収する
    let node0:Node = handle0.await.unwrap();
    let node1:Node = handle1.await.unwrap();    
    let node2:Node = handle2.await.unwrap();
    // TODO 6: Node::add_node(node0, node1, node2) で合算し、結果を println! で確認する
    let node_total = Node::add_node(node0,node1,node2);
    println!("{:?}",node_total);
}

