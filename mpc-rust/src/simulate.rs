
use crate::secret_sharing::Fp;
use crate::auction::{PRICES, PriceQuantity, derive, set_share_for_send, clearing_price};
use crate::node::{Node, Branch};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trader{
    pub is_buyer:bool,
    true_value:u64,
    pub threshold: u64,
    pub quantity:u64
}

impl Trader{
    fn new(is_buyer:bool, true_value:u64, threshold:u64,quantity:u64)->Trader{
        Trader{
            is_buyer:is_buyer,
            true_value: true_value,
            threshold: threshold,
            quantity:quantity
        }
    }

    pub fn trade(self)-> ([[Fp;9];3], Branch){
        set_share_for_send(self.threshold,self.quantity, self.is_buyer)
    }
}

pub fn round(all_shares:Vec::<[[Fp;9];3]>, branches:Vec::<Branch>)->Option<(u64,u64)>{
     //3nodeの立ち上げ 
    let mut node_a = Node::new();
    let mut node_b = Node::new();
    let mut node_c = Node::new();
    // shareを各nodeに追加
    for (shares, branch) in all_shares.iter().zip(branches.iter()){
        node_a = node_a.add_share(shares[0], *branch);
        node_b = node_b.add_share(shares[1], *branch);
        node_c = node_c.add_share(shares[2], *branch);
    }
    // node_a,node_b,node_cを合体
    let total_node = Node::add_node(node_a, node_b, node_c);
    let demmand_quantity = PriceQuantity{quantities:total_node.buyer_quantities};
    let supply_quantity = PriceQuantity{quantities:total_node.seller_quantities};
    // 需要と供給を計算
    clearing_price(demmand_quantity, supply_quantity)    
}



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_trader(){
        let trader = Trader::new(true,100,100,100);
        assert_eq!(trader, Trader::new(true,100,100,100));
    }
}
