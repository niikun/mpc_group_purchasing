use crate::secret_sharing::{Fp, Share, split_into_shares, reconstruct};

const PRICES: [u64; 9] = [95,100,105,110,115,120,125,130,135];

#[derive(PartialEq, Eq,Debug,Clone, Copy)]
struct PriceQuantity{
    quantities:[Fp;9]
}

impl PriceQuantity {
    fn new(values: [Fp; 9]) -> Self {
        PriceQuantity { quantities:values }
    }

    fn from_quantities(values: &[u64; 9]) -> Self {
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

pub enum Branch {
    Seller,
    Buyer
}

#[derive(Debug, Clone,PartialEq)]
pub struct Node{
    seller_quantities:[Fp;9],
    buyer_quantities:[Fp;9]
}


impl Node {
    pub fn new()->Node{
        Node{
            seller_quantities:[Fp::zero();9],
            buyer_quantities:[Fp::zero();9]
        }
    } 
pub fn add_share(&self,share:[Fp;9],branch:Branch)->Node{
    match branch {
        Branch::Seller => {
            let mut updates = self.seller_quantities;
            updates = update_share(updates,share);
            Node{buyer_quantities:self.buyer_quantities,
                seller_quantities:updates    
            }
        },
        Branch::Buyer => {
            let mut updates = self.buyer_quantities;
            updates = update_share(updates,share);
            Node{seller_quantities:self.seller_quantities,
                buyer_quantities:updates    
            }
        }
    }
}

pub fn add_node(node_1:Node,node_2:Node,node_3:Node)->Node{
    let mut results:Node = node_1.clone();
    let node_2_buyer:[Fp;9] = node_2.buyer_quantities;
    let node_3_buyer:[Fp;9] = node_3.buyer_quantities;
    let node_2_seller:[Fp;9] = node_2.seller_quantities;
    let node_3_seller:[Fp;9] = node_3.seller_quantities;
    results = results.add_share(node_2_buyer,Branch::Buyer);
    results = results.add_share(node_3_buyer,Branch::Buyer);
    results = results.add_share(node_2_seller,Branch::Seller);
    results = results.add_share(node_3_seller,Branch::Seller);
    results
}
}
pub fn update_share(updates:[Fp;9],share:[Fp;9])->[Fp;9]{
    let mut results = [Fp::zero();9];
    for (i, (u, s)) in updates.iter().zip(share.iter()).enumerate(){
        results[i] = *u + *s;
    }
    results
}


#[cfg(test)]
mod tests{
    use super::*;
    use rand::Rng;

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
    fn test_add_share(){
        let mut node1 = Node::new();
        let mut node2 = Node::new();
        let q1 = [Fp::new(10);9];
        let q2 = [Fp::new(20);9];
        node1 = node1.add_share(q1,Branch::Buyer);
        node1 = node1.add_share(q1,Branch::Buyer);
        node2 = node2.add_share(q2,Branch::Buyer);

        assert_eq!(node1,node2);

        node1 = node1.add_share(q1,Branch::Seller);
        node1 = node1.add_share(q1,Branch::Seller);
        node2 = node2.add_share(q2,Branch::Seller);

        assert_eq!(node1,node2);
    }
    #[test]
    fn test_add_node(){
        let mut node1 = Node::new();
        let mut node2 = Node::new();  
        let mut node3 = Node::new();      
        let mut node4 = Node::new();    
        let mut node5 = Node::new();    

        let q1 = [Fp::new(10);9];
        let q2 = [Fp::new(20);9];
        node1 = node1.add_share(q1,Branch::Buyer);
        node2 = node2.add_share(q1,Branch::Buyer);
        node3 = node3.add_share(q2,Branch::Buyer);
        node5 = Node::add_node(node1.clone(),node2.clone(),node3.clone());

        node4 = node4.add_share(q1,Branch::Buyer);
        node4 = node4.add_share(q1,Branch::Buyer);
        node4 = node4.add_share(q2,Branch::Buyer);

        assert_eq!(node4, node5);

        node1 = node1.add_share(q1,Branch::Seller);
        node2 = node2.add_share(q1,Branch::Seller);
        node3 = node3.add_share(q2,Branch::Seller);
        node5 = Node::add_node(node1.clone(),node2.clone(),node3.clone());

        node4 = node4.add_share(q1,Branch::Seller);
        node4 = node4.add_share(q1,Branch::Seller);
        node4 = node4.add_share(q2,Branch::Seller);
        assert_eq!(node4, node5);

    }

}

