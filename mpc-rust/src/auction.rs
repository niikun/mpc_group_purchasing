use crate::secret_sharing::{Fp, Share, split_into_shares, reconstruct};

const PRICES: [u64; 9] = [95,100,105,110,115,120,125,130,135];

#[derive(PartialEq, Eq,Debug,Clone, Copy)]
struct Quantity{
    quantity:[Fp;9]
}

impl Quantity {
    fn new(quantity: [Fp; 9]) -> Self {
        Quantity { quantity:quantity }
    }

    fn from_quantities(quantities: &[u64; 9]) -> Self {
        let quantity:[Fp;9] = quantities.map(|q| Fp::new(q));
        Quantity::new(quantity)
    }

    fn to_quantities(&self) -> [u64; 9] {
        self.quantity.map(|bit:Fp| bit.value())
    }

    pub fn quantities_share(&self:Quantity)->[[Fp;3];9]{
    let quantities_shares = self.quantity.map(|q:Fp| split_into_shares(q));
    quantities_shares
    }
}



struct Seller {

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
        let q1 = Quantity::new(quantities);
        let q2 = Quantity::from_quantities(
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
        let q = Quantity::new(quantities);
        let shares = quantities_share(q);
        let mut count = 0;
        for share in shares{
            for s in share{
                count += 1;
            }
        }
        assert_eq!(27, count);
    }

}


