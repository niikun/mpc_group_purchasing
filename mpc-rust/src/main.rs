use tokio::sync::mpsc;
use mpc_rust::secret_sharing::Fp;
use mpc_rust::node::{Node, Branch};

use mpc_rust::auction::{PriceQuantity, clearing_price, allocate, derive, set_share_for_send};

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

    // TODO 7: threshold/quantityを変数に出す(既存の数値をそのまま使えばOK)
    let (threshold_b0, quantity_b0) = (100, 300);
    let (threshold_b1, quantity_b1) = (110, 200);
    let (threshold_b2, quantity_b2) = (120, 500);
    let (threshold_s0, quantity_s0) = (105, 200);
    let (threshold_s1, quantity_s1) = (100, 500);

    // TODO 8: set_share_for_send呼び出しを、上の変数を使う形に書き換える

    // TODO 9: clearing_priceの結果がSomeのとき、aggrigate_marketの
    //   match Some((price, total_quantity)) ブロックと同じロジック
    //   (price <= threshold で参加判定 → allocate)を5人分書く

    
    let (shares_b0, branch_b0) = set_share_for_send(threshold_b0, quantity_b0, true);
    let (shares_b1, branch_b1) = set_share_for_send(threshold_b1, quantity_b1, true);
    let (shares_b2, branch_b2) = set_share_for_send(threshold_b2, quantity_b2, true);

    let (shares_s0, branch_s0) = set_share_for_send(threshold_s0, quantity_s0, false);
    let (shares_s1, branch_s1) = set_share_for_send(threshold_s1, quantity_s1, false);
    let senders = [sender0, sender1,sender2];
    let shares_total = [shares_b0,shares_b1, shares_b2, shares_s0, shares_s1];
    let branches = [branch_b0, branch_b1, branch_b2, branch_s0, branch_s1];
    for (shares, branch) in shares_total.iter().zip(branches.iter()){
        for (share, sender ) in shares.iter().zip(senders.iter()){
        sender.send((*share, *branch)).await;
        }
    }
    // TODO 4: sender0, sender1, sender2 をそれぞれ drop してチャネルを閉じる
    drop(senders);
  
    // TODO 5: handle0.await.unwrap() などで node0, node1, node2 を回収する
    let node0:Node = handle0.await.unwrap();
    let node1:Node = handle1.await.unwrap();    
    let node2:Node = handle2.await.unwrap();
    // TODO 6: Node::add_node(node0, node1, node2) で合算し、結果を println! で確認する
    let node_total = Node::add_node(node0,node1,node2);
    let demand = node_total.buyer_quantities;
    let supply = node_total.seller_quantities;
    let pq_demand = PriceQuantity::new(demand);
    let pq_supply = PriceQuantity::new(supply);
    let trades = clearing_price(pq_demand, pq_supply);
    match trades {
        Some((price,quentities)) => {
            println!("取引成立 price: {}, quntity: {}",price,quentities);
            let threshold_buyers = [threshold_b0,threshold_b1,threshold_b2];
            let threshold_sellers = [threshold_s0, threshold_s1];
            let quantities_buyers = [quantity_b0, quantity_b1, quantity_b2];
            let quantities_sellers = [quantity_s0, quantity_s1];
            let mut total_demand = 0;
            let mut total_supply = 0;
            for (th, q) in threshold_buyers.iter().zip(quantities_buyers.iter()){
                if price <= *th {
                    total_demand += q;
                }
            }
            for (th, q) in threshold_sellers.iter().zip(quantities_sellers.iter()){
                if price >= *th {
                    total_supply += q
                }
            }
            let mut buyer_allocated = [0u64;3];
            let mut seller_allocated = [0u64;2];
            for (i,(th, q)) in threshold_buyers.iter().zip(quantities_buyers.iter()).enumerate(){
                if price <= *th {
                    let allocated = allocate(*q, total_demand, quentities);
                    println!("b{} : {}", i, allocated);
                    buyer_allocated[i] = allocated;
                }
            }
            for (i,  (th, q)) in threshold_sellers.iter().zip(quantities_sellers.iter()).enumerate(){
                if price >= *th {
                    let allocated = allocate(*q, total_supply, quentities);
                    println!("s{} : {}", i, allocated);
                    seller_allocated[i] = allocated;
                }
            }
            println!("buyer: {:?}, seller: {:?}", buyer_allocated, seller_allocated);
        },
        None => {
            println!("取引不成立")
        }
    }
}

