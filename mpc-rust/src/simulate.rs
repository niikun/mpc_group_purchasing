
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
    pub fn new(is_buyer:bool, true_value:u64, threshold:u64,quantity:u64)->Trader{
        Trader{
            is_buyer:is_buyer,
            true_value: true_value,
            threshold: threshold,
            quantity:quantity
        }
    }
    // ３nodeへのshareの分割とBranchの作成
    pub fn trade(self)-> ([[Fp;9];3], Branch){
        set_share_for_send(self.threshold,self.quantity, self.is_buyer)
    }
    // 利益を計算する
    pub fn profit(self, price:u64, quantity:u64)->i64{
        if self.is_buyer{
            (self.true_value as i64 - price as i64) * quantity as i64
        } else {
            (price as i64 - self.true_value as i64) * quantity as i64
        }
    }
    // 適応ルール1　成立したら、変更なし。不成立だったら、成立するためにtrue valueにthresholdを近づける
    fn adjust(&mut self,traded:bool){
        if self.is_buyer{
            if !traded {
                let price_delta = self.true_value - self.threshold;
                self.threshold += price_delta/2;
            } 
        } else {
            if !traded {
                let price_delta = self.threshold - self.true_value;
                self.threshold -= price_delta/2
            }
        }
    }
    // 適応ルール2　成立したら、条件を5円修正。不成立だったら、成立するためにtrue valueにthresholdを近づける
    fn adjust_aggressive(&mut self,traded:bool){
        if self.is_buyer{
            if !traded {
                let price_delta = self.true_value - self.threshold;
                self.threshold += price_delta/2;
            } else {
                self.threshold -= 5;
            }
        } else {
            if !traded {
                let price_delta = self.threshold - self.true_value;
                self.threshold -= price_delta/2
            } else {
                self.threshold += 5;
            }
        }
    }
}

pub fn round(traders:&[Trader])->Option<(u64,u64,u64,u64)>{
     //3nodeの立ち上げ 
    let mut node_a = Node::new();
    let mut node_b = Node::new();
    let mut node_c = Node::new();
    // shareを各nodeに追加
    for trader in traders{
        let (shares, branch) = trader.trade();
        node_a = node_a.add_share(shares[0], branch);
        node_b = node_b.add_share(shares[1], branch);
        node_c = node_c.add_share(shares[2], branch);
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

pub fn run_round(traders: &mut Vec<Trader>, n_rounds:usize,aggressive:bool){
    for round_num in 0..n_rounds {
        let mut trade_quantities = Vec::new();
        let mut is_trades = Vec::new();
        if let Some((price, quantity, total_demand, total_supply)) = round(traders){
            println!("# price: {}, quantity: {}",price,quantity);
            for (i, trader) in traders.iter_mut().enumerate(){
                let mut is_trade = false;
                let mut trade_quantity = 0;
                if trader.is_buyer {
                    if price <= trader.threshold{
                        trade_quantity =  allocate(trader.quantity, total_demand, quantity);
                    }
                } else { 
                    if price >= trader.threshold{
                        trade_quantity = allocate(trader.quantity, total_supply, quantity);
                    }
                } 
                if trade_quantity > 0{
                        is_trade = true;
                }
                trade_quantities.push(trade_quantity);
                is_trades.push(is_trade);
                if aggressive{
                    trader.adjust_aggressive(is_trade);
                } else {
                    trader.adjust(is_trade);
                }
                println!("{}, {}, {}, {}, {}",round_num, i+1, trader.threshold, price, is_trade);
            }
        }
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

        let traders = Vec::from([trader_b1, trader_b2, trader_b3, trader_s1, trader_s2]);
         
        let (price, quantity, total_demmand, total_supply) = round(&traders).unwrap();

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

    #[test]
    fn test_profit(){
        let trader1 = Trader::new(true,100,100,100);
        let trader2 = Trader::new(false,100,100,100);

        let price1 = 200u64;
        let price2 = 90u64;
        let quantity1 = 1000u64;
        let quantity2 = 0u64;

        assert_eq!(trader1.profit(price2, quantity1), 10_000i64);
        assert_eq!(trader2.profit(price1, quantity1), 100_000i64);
        assert_eq!(trader2.profit(price2, quantity1), -10_000i64);
        assert_eq!(trader1.profit(price1, quantity2), 0i64);
    }
    #[test]
    fn test_adjust_aggressive(){
        let mut trader1 = Trader::new(true,100,80,100);
        let mut trader2 = Trader::new(false,100,120,100);
        let mut trader3 = Trader::new(true,100,80,100);
        let mut trader4 = Trader::new(false,100,120,100);

        trader1.adjust_aggressive(true);
        assert_eq!(trader1.threshold,75);
        trader2.adjust_aggressive(true);
        assert_eq!(trader2.threshold,125);
        trader3.adjust_aggressive(false);
        assert_eq!(trader3.threshold,90);
        trader4.adjust_aggressive(false);
        assert_eq!(trader4.threshold,110);
    }

    #[test]
    fn test_adjust(){
        let mut trader1 = Trader::new(true,100,80,100);
        let mut trader2 = Trader::new(false,100,120,100);
        let mut trader3 = Trader::new(true,100,80,100);
        let mut trader4 = Trader::new(false,100,120,100);

        trader1.adjust(true);
        assert_eq!(trader1.threshold,80);
        trader2.adjust(true);
        assert_eq!(trader2.threshold,120);
        trader3.adjust(false);
        assert_eq!(trader3.threshold,90);
        trader4.adjust(false);
        assert_eq!(trader4.threshold,110);
    }
}
