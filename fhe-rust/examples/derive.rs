use std::time::Instant;

use tfhe::{ClientKey, ConfigBuilder, FheUint16, generate_keys, set_server_key, FheBool};
use tfhe::prelude::*;

pub const PRICES: [u16; 9] = [95u16,100u16,105u16,110u16,115u16,120u16,125u16,130u16,135u16];

pub fn main(){
    let config = ConfigBuilder::default().build();
    let  (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
}

pub fn cordinator(threshold:u16, quantity:u16, client_key:&ClientKey)->(FheUint16,FheUint16){
    let threshold_enc = FheUint16::encrypt(threshold, client_key);
    let quantity_enc = FheUint16::encrypt(quantity, client_key);
    (threshold_enc, quantity_enc)
}

pub fn fhe_derive(
    threshold:&FheUint16, 
    quantity:&FheUint16,
    is_buyer:bool,
    zero:&FheUint16
) ->[FheUint16;9]{
    let price_quantity:[FheUint16;9] = core::array::from_fn(|i|{
        if is_buyer{
            let cond:FheBool = threshold.ge(PRICES[i]);
            let slot:FheUint16 = cond.if_then_else(quantity, zero);
            slot
        } else {
            let cond:FheBool =  threshold.le(PRICES[i]);
            let slot:FheUint16 = cond.if_then_else(quantity,zero);
            slot
        }
    }); 
    price_quantity
}

fn fhe_aggregate(
    participants:&[(FheUint16, FheUint16, bool)], zero: &FheUint16
) -> ([FheUint16;9], [FheUint16;9]){
    let mut demands:[FheUint16;9] = core::array::from_fn(|_| zero.clone());
    let mut supplys:[FheUint16;9] = core::array::from_fn(|_| zero.clone());

    for participant in participants{
        let (threshold, quantity, is_buyer) = participant;
        let price_quantity = fhe_derive(threshold,quantity,*is_buyer,zero);
        if *is_buyer {
            for i in 0..9 {demands[i] += &price_quantity[i];}
        } else {
            for i in 0..9 { supplys[i] += &price_quantity[i];}
        }
    }
    (demands, supplys)
}

fn fhe_clearing_price(
    demands: &[FheUint16; 9],
    supplys: &[FheUint16; 9],
    prices_enc: &[FheUint16; 9],  // PRICES を暗号化したもの
    zero: &FheUint16,
    enc_false: &FheBool,          // 暗号化した false
) -> (FheUint16, FheUint16) {     // (清算価格, 約定量) どちらも暗号のまま
    let mut price    = zero.clone();
    let mut quantity = zero.clone();
    let mut found    = enc_false.clone();

    for i in 0..9 {
        // このバンドで D(p) <= S(p) か
        let le_i: FheBool = demands[i].le(&supplys[i]);

        // 「条件成立」かつ「まだ見つけてない」= ここが最初の成立バンド
        let not_found: FheBool = !&found;
        let pick_i: FheBool = &le_i & &not_found;

        // pick_i が true のバンドだけ反映(他は zero を足す = 何もしない)
        price    += pick_i.if_then_else(&prices_enc[i], zero);
        quantity += pick_i.if_then_else(&demands[i],    zero);

        // ラッチ更新:一度 true になったら戻らない
        found = &found | &le_i;
    }

    (price, quantity)
}


#[cfg(test)]
mod tests{
use super::*;
    #[test]
    fn test_aggrigate(){
        let config = ConfigBuilder::default().build();
        let t  =Instant::now();
        let (client_key, server_key) = generate_keys(config);
        println!("key_generate: {:?}", t.elapsed());
        set_server_key(server_key);
        let t = Instant::now();
        let b1_th_enc = FheUint16::encrypt(110u16,&client_key);
        let b1_qty_enc = FheUint16::encrypt(100u16, &client_key);
        let b2_th_enc = FheUint16::encrypt(120u16, &client_key);
        let b2_qty_enc = FheUint16::encrypt(200u16, &client_key);
        let s1_th_enc = FheUint16::encrypt(105u16,&client_key);
        let s1_qty_enc = FheUint16::encrypt(150u16, &client_key);
        let buyer1:(FheUint16,FheUint16,bool) = (b1_th_enc, b1_qty_enc, true);
        let buyer2:(FheUint16,FheUint16,bool) = (b2_th_enc, b2_qty_enc, true); 
        let seller1:(FheUint16,FheUint16,bool) = (s1_th_enc, s1_qty_enc, false);
        println!("for encrypt threshold and quantity 3company: {:?}", t.elapsed());
        let d:[u16;9] = [300,300,300,300,200,200,0,0,0];
        let s:[u16;9] = [0,0,150,150,150,150,150,150,150];
        let zero = FheUint16::encrypt(0u16, &client_key);

        let participants = [buyer1, buyer2, seller1];
        let t = Instant::now();
        let (demand, supply) = fhe_aggregate(&participants, &zero);
        println!("for fhe_aggrefate {:?}", t.elapsed());
        let clear_demand:Vec<u16> = demand.iter().map(|d| d.decrypt(&client_key)).collect();
        let clear_supply:Vec<u16> = supply.iter().map(|s| s.decrypt(&client_key)).collect();
        assert_eq!(clear_demand, d);
        assert_eq!(clear_supply, s);

    }


    #[test]
    fn test_fhe_derive(){
        let config = ConfigBuilder::default().build();
        let  (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);
        let threshold = 120u16;
        let quantity:u16 = 100u16;
        let zero = FheUint16::encrypt(0u16, &client_key);
        let (threshold_enc, quantity_enc) = cordinator(threshold, quantity, &client_key);
        let price_quantity1 = fhe_derive(&threshold_enc, &quantity_enc,true,&zero);
        let price_quantity2 = fhe_derive(&threshold_enc, &quantity_enc,false,&zero);
        let clear_pq1:Vec<u16> = price_quantity1.iter().map(|v| v.decrypt(&client_key)).collect();
        let clear_pq2:Vec<u16> = price_quantity2.iter().map(|v| v.decrypt(&client_key)).collect();
        assert_eq!(clear_pq1, [quantity, quantity,quantity, quantity,quantity, quantity, 0, 0, 0]);
        assert_eq!(clear_pq2, [0, 0, 0, 0, 0, quantity, quantity,quantity, quantity]);
    }
    #[test]
    fn test_cordinate(){
        let config = ConfigBuilder::default().build();
        let  (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);
        let threshold = 120u16;
        let quantity:u16 = 100u16;
        let (crypted_threshold, crypted_quantity) = cordinator(threshold, quantity, &client_key);
        let clear_threshold:u16 = crypted_threshold.decrypt(&client_key);
        let clear_quantity:u16 = crypted_quantity.decrypt(&client_key);
        assert_eq!(clear_threshold, threshold);
        assert_eq!(clear_quantity, quantity);
    }

    #[test]
    fn test_fhe_clearing_price() {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);

        let zero = FheUint16::encrypt(0u16, &client_key);
        let enc_false = FheBool::encrypt(false, &client_key);
        let prices_enc: [FheUint16; 9] = core::array::from_fn(|i| FheUint16::encrypt(PRICES[i], &client_key));

        let d  = [100u16,90,80,70,60,50,40,30,20];
        let s1 = [10u16,20,30,40,50,60,70,80,90];   // i=5 で初成立 → price 120, qty 50
        let d_enc:  [FheUint16; 9] = core::array::from_fn(|i| FheUint16::encrypt(d[i],  &client_key));
        let s1_enc: [FheUint16; 9] = core::array::from_fn(|i| FheUint16::encrypt(s1[i], &client_key));

        let t = Instant::now();
        let (p, q) = fhe_clearing_price(&d_enc, &s1_enc, &prices_enc, &zero, &enc_false);
        println!("fhe_clearing_price: {:?}", t.elapsed());
        let p: u16 = p.decrypt(&client_key);
        let q: u16 = q.decrypt(&client_key);
        assert_eq!((p, q), (120, 50));
    }
}
