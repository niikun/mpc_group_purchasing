
use crate::secret_sharing::Fp;
use crate::auction::{PriceQuantity, derive, set_share_for_send};
use crate::node::{Branch};

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

    pub fn trade(self){
        let (share, branch) = set_share_for_send(self.threshold,self.quantity, self.is_buyer);

    
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
}
