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

