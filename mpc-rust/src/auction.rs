use crate::secret_sharing::{Fp, Share, split_into_shares, reconstruct};

const PRICES: [u64; 5] = [100, 200, 300, 400, 500];

#[derive(Debug, Clone, Copy)]
struct Quantity {
    quantity: [Fp; 5],
}
impl Quantity {
    fn new(quantity: [Fp; 5]) -> Self {
        Quantity { quantity }
    }

    fn from_quantities(quantities: [u64; 5]) -> Self {
        let quantity = quantities.map(|q| Fp::new(q));
        Quantity::new(quantity)
    }

    fn to_quantities(&self) -> [u64; 5] {
        self.quantity.map(|bit| bit.value())
    }
}

pub fn set_quantities

