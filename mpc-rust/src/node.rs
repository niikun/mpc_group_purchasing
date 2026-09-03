use crate::secret_sharing::Fp;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Branch {
    Seller,
    Buyer
}

#[derive(Debug, Clone,PartialEq, Serialize, Deserialize)]
pub struct Node{
    pub seller_quantities:[Fp;9],
    pub buyer_quantities:[Fp;9]
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
        // let mut node5 = Node::new();    

        let q1 = [Fp::new(10);9];
        let q2 = [Fp::new(20);9];
        node1 = node1.add_share(q1,Branch::Buyer);
        node2 = node2.add_share(q1,Branch::Buyer);
        node3 = node3.add_share(q2,Branch::Buyer);
        let mut node5 = Node::add_node(node1.clone(),node2.clone(),node3.clone());

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