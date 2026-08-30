use std::time::Instant;

use tfhe::{ClientKey, ConfigBuilder, FheUint16, FheUint32, generate_keys, set_server_key, FheBool};
use tfhe::prelude::*;

pub const PRICES: [u16; 9] = [95u16,100u16,105u16,110u16,115u16,120u16,125u16,130u16,135u16];

// FHE ベンチハーネス
//   buyer3 + seller2 の「1取引」= fhe_aggregate → fhe_clearing_price → fhe_allocate x5社
//   を RUNS 回まわし、先頭 1 回(ウォームアップ)を捨てて中央値/最小/最大を出す。
//   フェーズ別(aggregate / clearing / allocate)も個別に計測する。
//
//   実行: RUSTFLAGS="-C target-cpu=native" cargo run --release --example derive
//   鍵生成・入力暗号化・復号はタイミング区間の外(別枠)。
pub fn main() {
    use std::time::Duration;

    let config = ConfigBuilder::default().build();
    let t = Instant::now();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
    println!("keygen: {:?}", t.elapsed());

    // 定数(区間外)
    let zero = FheUint16::encrypt(0u16, &client_key);
    let zero32 = FheUint32::encrypt(0u32, &client_key);
    let enc_false = FheBool::encrypt(false, &client_key);
    let prices_enc: [FheUint16; 9] =
        core::array::from_fn(|i| FheUint16::encrypt(PRICES[i], &client_key));

    // 固定シナリオ = mpc-rust の test_aggrigate_market3 と同じ (threshold, quantity, is_buyer)
    // 期待: 清算価格 110
    let scenario: [(u16, u16, bool); 5] = [
        (110, 200, true),
        (120, 100, true),
        (95, 100, true),
        (110, 200, false),
        (100, 200, false),
    ];
    let t = Instant::now();
    let participants: Vec<(FheUint16, FheUint16, bool)> = scenario
        .iter()
        .map(|&(th, q, b)| {
            (
                FheUint16::encrypt(th, &client_key),
                FheUint16::encrypt(q, &client_key),
                b,
            )
        })
        .collect();
    println!("encrypt inputs (5社 x 2値): {:?}", t.elapsed());

    const RUNS: usize = 5; // 先頭を捨てる → 有効サンプル 4

    let (mut whole, mut agg, mut clr, mut alo) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for i in 0..RUNS {
        let t_whole = Instant::now();

        let t = Instant::now();
        let (demand, supply) = fhe_aggregate(&participants, &zero);
        let dur_agg = t.elapsed();

        let t = Instant::now();
        let (is_trade, price, dem_clr, sup_clr) =
            fhe_clearing_price(&demand, &supply, &prices_enc, &zero, &enc_false);
        let dur_clr = t.elapsed();

        let t = Instant::now();
        let mut allocs = Vec::with_capacity(5);
        for p in &participants {
            let total = if p.2 { &dem_clr } else { &sup_clr };
            allocs.push(fhe_allocate(&p.1, total, &dem_clr, &zero32));
        }
        let dur_alo = t.elapsed();

        let dur_whole = t_whole.elapsed();
        eprintln!(
            "run {i}: whole={dur_whole:?}  (aggregate={dur_agg:?}  clearing={dur_clr:?}  allocate x5={dur_alo:?})"
        );

        if i == 0 {
            // 正しさの目視確認(清算価格が MPC と一致するか)
            let it: bool = is_trade.decrypt(&client_key);
            let pr: u16 = price.decrypt(&client_key);
            let dc: u16 = dem_clr.decrypt(&client_key);
            let sc: u16 = sup_clr.decrypt(&client_key);
            println!(
                "sanity: is_trade={it}  price={pr}  demand@clearing={dc}  supply@clearing={sc}  (MPC 期待: price=110)"
            );
            let a: Vec<u32> = allocs.iter().map(|x| x.decrypt(&client_key)).collect();
            println!("  raw fhe_allocate (参加ゲート未実装, b3 は本来 0): {a:?}");
        } else {
            whole.push(dur_whole);
            agg.push(dur_agg);
            clr.push(dur_clr);
            alo.push(dur_alo);
        }
    }

    fn stats(label: &str, mut v: Vec<Duration>) {
        v.sort();
        let n = v.len();
        println!(
            "{label}: median={:?}  min={:?}  max={:?}  (n={n})",
            v[n / 2],
            v[0],
            v[n - 1]
        );
    }

    println!("--- FHE 1取引 (buyer3 + seller2) ---");
    println!("threads available: {:?}", std::thread::available_parallelism());
    stats("aggregate (D/S 構築, fhe_derive 込み)", agg);
    stats("clearing scan (9 バンド)", clr);
    stats("allocate x5 社", alo);
    stats("whole trade", whole);
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

fn fhe_allocate(
    desired:&FheUint16, //自社の希望量
    total: & FheUint16,  //buyerならD sellerならS
    traded: &FheUint16,  //成立総量 D
    zero32: &FheUint32,  //32bit の暗号ゼロ
) -> FheUint32 {
    let desired32 = FheUint32::cast_from(desired.clone());
    let total32  = FheUint32::cast_from(total.clone());
    let traded32 = FheUint32::cast_from(traded.clone());
    let numerator = &traded32 * &desired32;
    let quantity = numerator / total32;
    let cond = total.ne(0);
    let result = cond.if_then_else(&quantity, zero32);
    result
}

fn fhe_clearing_price(
    demands: &[FheUint16; 9],
    supplys: &[FheUint16; 9],
    prices_enc: &[FheUint16; 9],  // PRICES を暗号化したもの
    zero: &FheUint16,
    enc_false: &FheBool,          // 暗号化した false
) -> (FheBool, FheUint16, FheUint16, FheUint16) {     // (トレードしたか,清算価格, 約定時のデマンド量, 約定時のサプライ量) どちらも暗号のまま
    let mut price    = zero.clone();
    let mut demand_at_clearing = zero.clone();
    let mut supply_at_clearing = zero.clone();
    let mut found    = enc_false.clone();
    let mut total_demand = zero.clone();
    let mut total_supply = zero.clone();
    for i in 0..9 {
        // このバンドで D(p) <= S(p) か
        let le_i: FheBool = demands[i].le(&supplys[i]);
        total_demand += &demands[i];
        total_supply += &supplys[i];
        // 「条件成立」かつ「まだ見つけてない」= ここが最初の成立バンド
        let not_found: FheBool = !&found;
        let pick_i: FheBool = &le_i & &not_found;

        // pick_i が true のバンドだけ反映(他は zero を足す = 何もしない)
        price    += pick_i.if_then_else(&prices_enc[i], zero);
        demand_at_clearing += pick_i.if_then_else(&demands[i], zero);
        supply_at_clearing += pick_i.if_then_else(&supplys[i], zero);

        // ラッチ更新:一度 true になったら戻らない
        found = &found | &le_i;
    }
    let has_demand = total_demand.ne(0);
    let has_supply = total_supply.ne(0);
    let is_trade = has_demand & has_supply & found;
    (is_trade, price, demand_at_clearing, supply_at_clearing)
}


#[cfg(test)]
mod tests{
use tfhe::array::ClearSliceMut;

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
        let (is_trade, p, q,s) = fhe_clearing_price(&d_enc, &s1_enc, &prices_enc, &zero, &enc_false);
        println!("fhe_clearing_price: {:?}", t.elapsed());
        let p: u16 = p.decrypt(&client_key);
        let q: u16 = q.decrypt(&client_key);
        let is_trade = is_trade.decrypt(&client_key);
        assert_eq!((p, q), (120, 50));
        assert_eq!(is_trade, true);
    }

    #[test]
    fn test_fhe_clearing_price_zero_demand() {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);

        let zero = FheUint16::encrypt(0u16, &client_key);
        let enc_false = FheBool::encrypt(false, &client_key);
        let prices_enc: [FheUint16; 9] = core::array::from_fn(|i| FheUint16::encrypt(PRICES[i], &client_key));

        let d  = [0u16,0,0,0,0,0,0,0,0];
        let s1 = [10u16,20,30,40,50,60,70,80,90];   // i=5 で初成立 → price 120, qty 50
        let d_enc:  [FheUint16; 9] = core::array::from_fn(|i| FheUint16::encrypt(d[i],  &client_key));
        let s1_enc: [FheUint16; 9] = core::array::from_fn(|i| FheUint16::encrypt(s1[i], &client_key));

        let t = Instant::now();
        let (is_trade, p, q,s) = fhe_clearing_price(&d_enc, &s1_enc, &prices_enc, &zero, &enc_false);
        println!("fhe_clearing_price: {:?}", t.elapsed());
        let clear_is_trade: bool = is_trade.decrypt(&client_key);
        assert_eq!(clear_is_trade,false);
    }

    #[test]
    fn test_allocate(){
        let desired = 50u16;
        let total = 100u16;
        let traded = 20u16;
        let qty = 20 * 50 / 100;
        let t = Instant::now();
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);
        println!("set up:{:?}", t.elapsed());
        let t = Instant::now();
        let desired_fhe16 = FheUint16::encrypt(desired, &client_key);
        let total_fhe16 = FheUint16::encrypt(total, &client_key);
        let traded_fhe16 = FheUint16::encrypt(traded, &client_key);
        let zero32 = FheUint32::encrypt(0u32, &client_key);
        println!("encrypt:{:?}", t.elapsed());
        let t = Instant::now();
        let quantity = fhe_allocate(&desired_fhe16, &total_fhe16, &traded_fhe16, &zero32);
        println!("fhe allocate: {:?}", t.elapsed());
        let clear_quantity:u32 = quantity.decrypt(&client_key);
        assert_eq!(clear_quantity, qty);
    }

}
