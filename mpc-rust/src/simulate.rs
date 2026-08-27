
use crate::secret_sharing::Fp;
use crate::auction::{PRICES, PriceQuantity, set_share_for_send, clearing_price, allocate};
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

pub fn round(all_shares:Vec::<[[Fp;9];3]>, branches:Vec::<Branch>)->Option<(u64,u64,u64,u64)>{
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
    let demand_quantity = PriceQuantity{quantities:total_node.buyer_quantities};
    let supply_quantity = PriceQuantity{quantities:total_node.seller_quantities};
    // 需要と供給を計算
    match clearing_price(demand_quantity, supply_quantity){
        Some((price, quantity)) => {
            let position = PRICES.iter().position(|p| *p == price).unwrap();
            let total_demand = demand_quantity.quantities[position].value();
            let total_supply = supply_quantity.quantities[position].value();
            return Some((price, quantity, total_demand, total_supply));
        },
        None => {return None;}
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_trader(){
        let trader = Trader::new(true,100,100,100);
        assert_eq!(trader, Trader::new(true,100,100,100));
    }

    #[test]
    fn test_simulate(){
        let threshold_b1 = 100;
        let threshold_b2 = 110;
        let threshold_b3 = 105;
        let threshold_s1 = 90;
        let threshold_s2 = 100;

        let quantity_b1 = 100;
        let quantity_b2 = 200;
        let quantity_b3 = 300;
        let quantity_s1 = 400;
        let quantity_s2 = 500;     

        let trader_b1 = Trader::new(true, 100, threshold_b1,100);
        let trader_b2 = Trader::new(true, 110, threshold_b2,200);
        let trader_b3 = Trader::new(true, 105, threshold_b3,300);
        let trader_s1 = Trader::new(false, 90, threshold_s1,400);
        let trader_s2 = Trader::new(false, 100, threshold_s2,500);

        let (shares_b1, branch_b1) = trader_b1.trade();
        let (shares_b2, branch_b2) = trader_b2.trade();
        let (shares_b3, branch_b3) = trader_b3.trade();
        let (shares_s1, branch_s1) = trader_s1.trade();
        let (shares_s2, branch_s2) = trader_s2.trade();

        let all_shares = Vec::from([shares_b1,shares_b2,shares_b3,shares_s1,shares_s2]);
        let branches = Vec::from([branch_b1,branch_b2,branch_b3,branch_s1,branch_s2]);

 
        let (price, quantity, total_demmand, total_supply) = round(all_shares,branches).unwrap();

        // Buyer: 参加条件は price <= threshold、分母は total_demmand(需要側)
        let b1_trade = if price <= threshold_b1 { allocate(quantity_b1, total_demmand, quantity) } else { 0 };
        let b2_trade = if price <= threshold_b2 { allocate(quantity_b2, total_demmand, quantity) } else { 0 };
        let b3_trade = if price <= threshold_b3 { allocate(quantity_b3, total_demmand, quantity) } else { 0 };
        // Seller: 参加条件は price >= threshold、分母は total_supply(供給側)
        let s1_trade = if price >= threshold_s1 { allocate(quantity_s1, total_supply, quantity) } else { 0 };
        let s2_trade = if price >= threshold_s2 { allocate(quantity_s2, total_supply, quantity) } else { 0 };

        assert_eq!(price, 100);
        assert_eq!(quantity, 600);
        assert_eq!(b1_trade,100);
        assert_eq!(b2_trade,200);
        assert_eq!(b3_trade,300);
        assert_eq!(s1_trade,400*600/900);
        assert_eq!(s2_trade,500*600/900);
    }


}
