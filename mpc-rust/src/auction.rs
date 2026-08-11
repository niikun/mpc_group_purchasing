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

// struct Sellers {
//     // 各価格での希望販売数量を保持。デモでは2社
//     seller:[PriceQuantity;2]
// }

// struct Buyers {
//     // 各価格での希望購入数量を保持
//     buyer:[Quantity;3]
// }

// struct Trades {
//     // 取引の価格と取引の量
//     price:u64,
//     trade:u64
// }

// pub fn make_trade(buyers:Buyers, sellers:Sellers){

// }

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

}


