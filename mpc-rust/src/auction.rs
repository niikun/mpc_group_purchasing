use std::intrinsics::discriminant_value;

use crate::secret_sharing::{Fp, Share, split_into_shares};
use crate::node::{Node, Branch};

pub const PRICES: [u64; 9] = [95,100,105,110,115,120,125,130,135];

#[derive(PartialEq, Eq,Debug,Clone, Copy)]
pub struct PriceQuantity{
    quantities:[Fp;9]
}

impl PriceQuantity {
    pub fn new(values: [Fp; 9]) -> Self {
        PriceQuantity { quantities:values }
    }

    pub fn from_quantities(values: &[u64; 9]) -> Self {
        let quantities:[Fp;9] = values.map(|v| Fp::new(v));
        PriceQuantity::new(quantities)
    }

    fn to_quantities(&self) -> [u64; 9] {
        self.quantities.map(|bit:Fp| bit.value())
    }

    pub fn quantities_share(self:PriceQuantity)->([Fp;9],[Fp;9],[Fp;9]){
        let quantities_shares = self.quantities.map(|q:Fp| split_into_shares(q));
        let (mut share_1, mut share_2, mut share_3) = ([Fp::zero();9], [Fp::zero();9], [Fp::zero();9]);
        for (i, share) in quantities_shares.iter().enumerate(){
            share_1[i] = share[0];
            share_2[i] = share[1];
            share_3[i] = share[2];
        }
        (share_1, share_2, share_3)
    }

    pub fn quantities_join(share_1:&[Fp;9], share_2:&[Fp;9], share_3:&[Fp;9])->PriceQuantity{
        let mut values = [Fp::zero();9];
        for (i,((s_1, s_2), s_3)) in share_1.iter().zip(share_2.iter()).zip(share_3.iter()).enumerate(){
            values[i] = *s_1 + *s_2 + *s_3;
        }
        PriceQuantity{quantities:values}
    }
}

pub fn derive(threshhold:u64,quantity:u64,is_buyer:bool) -> PriceQuantity{
    let mut price_quantities = PriceQuantity::new([Fp::zero();9]);
    match is_buyer{
        true => {
            for (i,price) in PRICES.iter().enumerate(){
                if price <= &threshhold{
                    price_quantities.quantities[i] = Fp::new(quantity);
                }
            }
        }
        false => {
            for (i,price) in PRICES.iter().enumerate(){
                if price >= &threshhold{
                    price_quantities.quantities[i] = Fp::new(quantity);
                }
            }
        }
    }
    price_quantities
}

fn clearing_price(demand:PriceQuantity, supply:PriceQuantity)->Option<(u64,u64)>{
    for ((d,s),p) in demand.quantities.iter().zip(supply.quantities.iter()).zip(PRICES.iter()){
        if d <= s{
            return Some((*p, d.value()));
        }
    }    None
}
pub fn allocate(desired: u64, total: u64, traded: u64) -> u64 {
    traded * desired / total 
}

fn aggrigate_market(
    threshold_b1:u64,
    threshold_b2:u64,
    threshold_b3:u64,
    threshold_s1:u64,
    threshold_s2:u64,
    quantity_b1:u64,
    quantity_b2:u64,
    quantity_b3:u64,
    quantity_s1:u64,
    quantity_s2:u64
)->Option<(u64,u64,u64)>{
    // 3社　B1,B2,B3
    let b1_pq = derive(threshold_b1,quantity_b1,true);
    let b2_pq = derive(threshold_b2,quantity_b2,true);
    let b3_pq = derive(threshold_b3,quantity_b3,true);
    //  2社 S1,S2
    let s1_pq = derive(threshold_s1,quantity_s1,false);
    let s2_pq = derive(threshold_s2,quantity_s2,false);
    // 3つのNode
    let mut node_a = Node::new();
    let mut node_b = Node::new();
    let mut node_c = Node::new();
    //それぞれのpqを分割
    let (b1_a,b1_b,b1_c) = b1_pq.quantities_share();
    let (b2_a,b2_b,b2_c) = b2_pq.quantities_share();
    let (b3_a,b3_b,b3_c) = b3_pq.quantities_share();
    let (s1_a,s1_b,s1_c) = s1_pq.quantities_share();
    let (s2_a,s2_b,s2_c) = s2_pq.quantities_share();
    //それぞれのnodeに足し上げる
    //node_a
    node_a.add_share(b1_a, Branch::Buyer);
    node_a.add_share(b2_a, Branch::Buyer);
    node_a.add_share(b3_a, Branch::Buyer);
    node_a.add_share(s1_a, Branch::Seller);
    node_a.add_share(s2_a, Branch::Seller);
    //node_b
    node_b.add_share(b1_b, Branch::Buyer);
    node_b.add_share(b2_b, Branch::Buyer);
    node_b.add_share(b3_b, Branch::Buyer);
    node_b.add_share(s1_b, Branch::Seller);
    node_b.add_share(s2_b, Branch::Seller);
    //node_c
    node_c.add_share(b1_c, Branch::Buyer);
    node_c.add_share(b2_c, Branch::Buyer);
    node_c.add_share(b3_c, Branch::Buyer);
    node_c.add_share(s1_c, Branch::Seller);
    node_c.add_share(s2_c, Branch::Seller);

    // node_a,node_b,node_cを合体
    let total_node = Node::add_node(node_a, node_b, node_c);
    let demand_quantity:PriceQuantity = PriceQuantity { quantities:total_node.buyer_quantities};
    let supply_quantity:PriceQuantity = PriceQuantity { quantities: total_node.seller_quantities};
    for demand_quantity
    // 需要と供給を計算
    if let Some((price, total_quantity)) = clearing_price(demand_quantity, supply_quantity){
        let b1_trade = allocate(quantity_b1,total_node.buyer_quantities.iter().max(),total_quantity);
        let b2_trade = allocate(quantity_b2,total_node.buyer_quantities.iter().max(),total_quantity);
        let b3_trade = allocate(quantity_b3,total_node.buyer_quantities.iter().max(),total_quantity);
        Some((b1_trade, b2_trade, b3_trade))
    }else{
        None
    }


}

#[cfg(test)]
mod tests{
    use super::*;
    use rand::Rng;

    #[test]
    fn test_derive(){
        // pub const PRICES: [u64; 9] = [95,100,105,110,115,120,125,130,135];
        let pq1 = derive(120,100,true);
        let pq1_test = PriceQuantity::new([Fp::new(100),Fp::new(100),Fp::new(100),Fp::new(100),Fp::new(100),Fp::new(100),Fp::zero(),Fp::zero(),Fp::zero()]);
        assert_eq!(pq1,pq1_test);
        
        let pq2 = derive(100,100,false);
        let pq2_test = PriceQuantity::new([Fp::zero(),Fp::new(100),Fp::new(100),Fp::new(100),Fp::new(100),Fp::new(100),Fp::new(100),Fp::new(100),Fp::new(100)]);
        assert_eq!(pq2,pq2_test);    
    }

    #[test]
    fn test_quantity(){
        let quantities = [
            Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10)
            ];
        let q1 = PriceQuantity::new(quantities);
        let q2 = PriceQuantity::from_quantities(
            &[10,10,10,10,10,10,10,10,10]
        );
        let quantity2 = q2.to_quantities();
        assert_eq!(q1,q2);
        assert_eq!(quantity2,[10,10,10,10,10,10,10,10,10]);
    }

    #[test]
    fn test_quantities_share(){
        let quantities = [
            Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10),Fp::new(10)
            ];
        let q = PriceQuantity::new(quantities);
        let (share_1,share_2,share_3) = q.quantities_share();
        let new_q = PriceQuantity::quantities_join(&share_1, &share_2, &share_3);
        assert_eq!(q, new_q);
    }
    #[test]
    fn test_clearing_price(){
        let dq = [
            Fp::new(100),Fp::new(90),Fp::new(80),Fp::new(70),Fp::new(60),Fp::new(50),Fp::new(40),Fp::new(30),Fp::new(20)
        ];
        let sq1 = [
            Fp::new(10),Fp::new(20),Fp::new(30),Fp::new(40),Fp::new(50),Fp::new(60),Fp::new(70),Fp::new(80),Fp::new(90)
        ];
        let sq2 = [
            Fp::new(1),Fp::new(2),Fp::new(3),Fp::new(4),Fp::new(5),Fp::new(6),Fp::new(7),Fp::new(8),Fp::new(9)
        ];
        let dpq = PriceQuantity::new(dq);
        let spq1 = PriceQuantity::new(sq1);
        let spq2 = PriceQuantity::new(sq2);
        let res1 = (120u64,50u64);
        assert_eq!(res1,clearing_price(dpq, spq1).unwrap().clone());
        assert_eq!(None,clearing_price(dpq, spq2));
    }

    #[test]
    fn test_allocate(){
        let (d1,total,traded) = (50,100,20);
        let a1 = allocate(d1,total,traded);
        assert_eq!(10u64,a1);
    }
}

