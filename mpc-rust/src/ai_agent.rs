use reqwest;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auction::{PRICES, PriceQuantity, clearing_price, set_share_for_send, allocate};
use crate::node::Node;

const API_URL:&str = "https://api.anthropic.com/v1/messages";
const MODEL:&str = "claude-haiku-4-5";

pub enum Profile{
    Buyer(BuyerProfile),
    Supplier(SupplierProfile),
}

#[derive(Debug, PartialEq)]
pub enum BidError{
    NotOnGrid(u64),
    QuantityExceedsMax(u64,u64),
    PolicyViolation,
} 

#[derive(Debug, Clone)]
pub struct BuyerProfile{
    pub strategy:String,
    pub current_stock:u64,
    pub use_per_day:u64,
    pub reorder_point:u64,
    pub max_stock:u64,
    pub order_history:Vec<OrderRecord>,
    price_ceiling:u64, // 社内方針: この価格までなら買っていい、という上限
    pub notes:String,      // 特記事項(自由記述)
}

impl BuyerProfile{
    pub fn new(strategy:String, current_stock:u64, use_per_day:u64, reorder_point:u64, max_stock:u64, order_history:Vec<OrderRecord>, price_ceiling:u64, notes:String) -> Self {
        BuyerProfile {
            strategy,
            current_stock,
            use_per_day,
            reorder_point,
            max_stock,
            order_history,
            price_ceiling,
            notes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderRecord{
    pub date:String,
    pub order_amount:u64, 
}


#[derive(Debug, Clone)]
pub struct SupplierProfile{
    pub strategy:String,
    pub current_stock:u64,
    pub supply_per_day:u64,
    pub safety_stock:u64,
    pub max_stock:u64,
    pub supply_history:Vec<SupplyRecord>,
    pub price_floor:u64, // 社内方針: この価格を下回っては売らない、という下限
    pub notes:String,    // 特記事項(自由記述)
}

#[derive(Debug, Clone)]
pub struct SupplyRecord{
    pub date:String,
    pub supply_amount:u64, 
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Bid{
    pub threshold: u64,
    pub quantity:u64,
    pub reason:String,
}

pub struct RoundResult{
    pub round: usize,
    pub price: u64,
    pub volume: u64,
    pub allocation: Vec<u64>,
    pub thresholds: Vec<u64>
}

fn format_history(results:&[RoundResult], my_index:usize) -> String{
    if results.is_empty() {
        return "(まだ取引履歴はありません。今回が初回の入札です。)".to_string();
    }
    let mut lines = Vec::new();
    for r in results{
        let prompt = format!(
            "{}ラウンド目: 清算価格: {}円, 成立総量: {}l, あなたの入札threshold: {}円, 約定量: {}l", 
            r.round+1, r.price, r.volume, r.thresholds[my_index], r.allocation[my_index]);
        lines.push(prompt);        
    }
    lines.join("\n")
}

fn build_buyer_prompt(profile:&BuyerProfile, history:&str)->String{
    let prices = PRICES;
    let strategy:&String = &profile.strategy;
    let current_stock:u64 = profile.current_stock;
    let use_per_day:u64 = profile.use_per_day;
    let reorder_point:u64 = profile.reorder_point;
    let max_stock:u64 = profile.max_stock;
    let order_history:String = profile.order_history.iter()
            .map(|o| format!("{}:{}l",o.date,o.order_amount))
            .collect::<Vec<_>>()
            .join(",");
    let price_ceiling:u64 = profile.price_ceiling;
    let notes:&String = &profile.notes;
    let prompt = format!(r#"
        あなたは外食チェーンの食用油のbuyerです。
        目的は、{strategy}ことです。 
        購入する価格は{prices:?}グリッドで決まっています。
        それ以外の価格での購入はできません。
        ### 会社の状況
        現在のストック量(単位:l): {current_stock}
        1日の消費量(単位:l): {use_per_day},
        再発注点(単位:l):{reorder_point},
        最大在庫量(単位:l):{max_stock},
        過去の購入履歴:{order_history},
        購入価格上限(単位:円):{price_ceiling}
        特記事項:{notes}
        ### 前ラウンドまでの公開結果
        {history}
        これを踏まえて今回の入札を決めて下さい。
        ### 出力形式
        以下の例のようなjsonのみ。余計な文章は加えないでください。
        {{"threshold":100, "quantity":200, "reason":"在庫に対して、使用量のため..."}}
        thresholdはこの価格までなら買うという上限金額です。
        必ず {price_ceiling} 円以下で、かつ pricesの中の数字にしてください。
        quantityは購入する数量です。max_stock - current_stock を超えない。
        reasonは判断した理由を50字以内で説明してください
        "#);
    prompt
}

fn build_supplier_prompt(profile:&SupplierProfile, history:&str)->String{
    let prices = PRICES;
    let strategy:&String = &profile.strategy;
    let current_stock:u64 = profile.current_stock;
    let supply_per_day:u64 = profile.supply_per_day;
    let safety_stock:u64 = profile.safety_stock;
    let max_stock:u64 = profile.max_stock;
    let supply_history:String = profile.supply_history.iter()
            .map(|o| format!("{}:{}l",o.date,o.supply_amount))
            .collect::<Vec<_>>()
            .join(",");
    let price_floor:u64 = profile.price_floor;
    let notes:&String = &profile.notes;
    let prompt = format!(r#"
        あなたは外食チェーンの食用油の供給者です。
        目的は、{strategy}ことです。 
        販売する価格は{prices:?}グリッドで決まっています。
        それ以外の価格での販売はできません。
        ### 会社の状況
        現在のストック量(単位:l): {current_stock}
        1日の供給量(単位:l): {supply_per_day},
        安全在庫(単位:l):{safety_stock},
        最大在庫量(単位:l):{max_stock},
        過去の販売履歴:{supply_history},
        販売価格下限(単位:円):{price_floor}
        特記事項:{notes}
        ### 前ラウンドまでの公開結果
        {history}
        これを踏まえて今回の入札を決めて下さい
        ### 出力形式
        以下の例のようなjsonのみ。余計な文章は加えないでください。
        {{"threshold":100, "quantity":200, "reason":"在庫に対して、使用量のため..."}}
        thresholdは「この価格を下回っては売らない」という下限価格です。
        必ず{price_floor} 円以上で、かつ pricesの中の数字にしてください。
        quantityは販売する数量です。current_stock-safty_stock以下で設定してください。
        reasonは判断した理由を50字以内で説明してください。
        "#);
    prompt
}

pub async fn call_claude(prompt:&str) -> Result<String, Box<dyn std::error::Error>>{
    dotenvy::dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    let client = reqwest::Client::new();
    let message = json!({
        "model":MODEL,
        "max_tokens":256,
        "messages": [{ "role": "user", "content": prompt }] 
    });
    let response :Value = client
        .post(API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&message)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let text = response["content"][0]["text"]
                .as_str()
                .ok_or("responseにtextブロックがない")?
                .to_string();
    Ok(text)
}

pub fn parse_bid(text:&str)->Result<Bid, Box<dyn std::error::Error>>{
        let start = text.find('{').unwrap();
        let end = text.rfind('}').unwrap();
        let target_text = &text[start..=end];
        let bid:Bid = serde_json::from_str(target_text)?;
        Ok(bid)
}

pub fn validate_bid(bid:&Bid, profile:&Profile) -> Result<Bid, BidError>{
    match profile {
        Profile::Buyer(buyer_profile)=>{
            if !PRICES.contains(&bid.threshold){
                Err(BidError::NotOnGrid(bid.threshold))
            } else if bid.threshold > buyer_profile.price_ceiling{
                Err(BidError::PolicyViolation)
            } else if bid.quantity > buyer_profile.max_stock - buyer_profile.current_stock{
                Err(BidError::QuantityExceedsMax(bid.quantity, buyer_profile.max_stock - buyer_profile.current_stock))
            } else {
                Ok(bid.clone())
            }
        },
        Profile::Supplier(supplier_profile ) =>{
            if !PRICES.contains(&bid.threshold){
                Err(BidError::NotOnGrid(bid.threshold))
            } else if bid.threshold < supplier_profile.price_floor{
                Err(BidError::PolicyViolation)
            } else if bid.quantity > (supplier_profile.current_stock - supplier_profile.safety_stock){
                Err(BidError::QuantityExceedsMax(bid.quantity, supplier_profile.current_stock - supplier_profile.safety_stock))
            } else {
                Ok(bid.clone())
            }
        }
    }
}

pub async fn propose_bid(
    profile:&Profile, 
    history: &str,
    prev:Option<&Bid>,
    max_retries:u32
) -> Bid {
    let prompt = match profile {
        Profile::Buyer(p)    => build_buyer_prompt(p, history),
        Profile::Supplier(p) => build_supplier_prompt(p, history),
    };
    for _ in 0..max_retries {
        let text = match call_claude(&prompt).await{
            Ok(t) => t,
            Err(e) =>{eprintln!("call_claude 失敗:{e:?}"); continue;}
        };
        let bid = parse_bid(&text);
        match bid {
            Ok(bid) => match validate_bid(&bid,profile){
                Ok(valid) => return valid,
                Err(e) => {eprintln!("validate 失敗:{e:?}"); continue;}
            },
            Err(e) => {
                eprintln!("parse 失敗:{e:?}");
                continue;
            }
        }
    }    
    match prev {
        Some(bid) => {return bid.clone();},
        None => panic!("value error"),
    }
}

pub fn aggregate_bids(bids:&[(Bid, bool)]) -> Option<(u64, u64, u64, u64)>{
    let mut node_a = Node::new();
    let mut node_b = Node::new();
    let mut node_c = Node::new();
    for (bid, is_buyer) in bids.iter(){
        let (shares, branch) = set_share_for_send(bid.threshold,bid.quantity, *is_buyer);
        node_a = node_a.add_share(shares[0],branch);
        node_b = node_b.add_share(shares[1], branch);
        node_c = node_c.add_share(shares[2], branch);
    }
    let total_node = Node::add_node(node_a, node_b, node_c);
    let demand_quantity = PriceQuantity{quantities:total_node.buyer_quantities};
    let supply_quantity = PriceQuantity{quantities:total_node.seller_quantities};
    match clearing_price(demand_quantity, supply_quantity){
        Some((price, quantity)) =>{
            let position = PRICES.iter().position(|p| *p == price).unwrap();
            let total_demand = demand_quantity.quantities[position].value();
            let total_supply = supply_quantity.quantities[position].value();
            return Some((price, quantity, total_demand, total_supply));
        },
        None => {return None;}
    }
}

pub async fn run_round(profiles:&[Profile], n_rounds: usize) ->Vec<RoundResult>{
    let mut results = Vec::new();
    let mut last_bids = Vec::new();
    for round_num in 0..n_rounds{
        let mut trade_quantities = Vec::new();
        let mut bids = Vec::new();
        let mut thresholds = Vec::new();
        for (i, profile) in profiles.iter().enumerate(){
            let history = format_history(&results, i);
            let bid = propose_bid(profile, &history,last_bids.get(i),3).await;
            match profile {
                Profile::Buyer(_) => bids.push((bid, true)),
                Profile::Supplier(_) => bids.push((bid, false))
            }
        }
        let result = match aggregate_bids(&bids){
            Some((price, quantity, total_demand, total_supply)) =>{
                for (i,(profile,bid)) in profiles.iter().zip(bids.iter()).enumerate(){
                    let mut trade_quantity = 0u64;
                    let mut is_trade = false;
                    match profile {
                        Profile::Buyer(_) => {
                            if price <= bid.0.threshold{
                                trade_quantity = allocate(bid.0.quantity, total_demand, quantity);
                        }
                        },
                        Profile::Supplier(_) => {
                                if price >= bid.0.threshold{
                                trade_quantity = allocate(bid.0.quantity, total_supply, quantity);
                            }
                        }
                    }
                    if trade_quantity > 0 {
                        is_trade = true;
                    }
                    println!("{},{},{},{},{}",round_num, i+1, bid.0.threshold, price, is_trade);
                    trade_quantities.push(trade_quantity);
                    thresholds.push(bid.0.threshold);
                }
                RoundResult{
                    round:round_num,
                    price:price,
                    volume:quantity,
                    allocation:trade_quantities,
                    thresholds:thresholds,
                }
            },
            None => {
                for (i,(_,bid)) in profiles.iter().zip(bids.iter()).enumerate(){
                    println!("{},{},{},{},{}",round_num, i+1, bid.0.threshold, 0, false);
                }
                RoundResult {
                    round: round_num,
                    price: 0, volume: 0,
                    allocation: vec![0; profiles.len()],
                    thresholds: bids.iter().map(|(b, _)| b.threshold).collect(),
                }
            }
        };
        last_bids = bids.iter().map(|(b, _)| b.clone()).collect();
        results.push(result);
    }
    return results;
} 

#[cfg(test)]
mod tests{
    use crate::node::Branch::Buyer;

use super::*;
    #[test]
    fn test_validate_bid(){
        let bid1 = Bid{threshold:100, quantity:200, reason:"在庫が薄いため".to_string()};
        let bid2 = Bid{threshold:101, quantity:200, reason:"在庫が薄いため".to_string()};
        let bid3 = Bid{threshold:100, quantity:3000, reason:"在庫が薄いため".to_string()};
        let bid4 = Bid{threshold:100, quantity:0, reason:"在庫が薄いため".to_string()};
        let buyer_profile = BuyerProfile::new(
            "在庫を切らさず、なるべく安く仕入れる".to_string(),
            800,    // current_stock: 現在庫 800L
            120,    // use_per_day: 1日120L消費
            600,    // reorder_point: 600Lを切ったら発注
            2000,   // max_stock: タンク上限 2000L
            vec![
                OrderRecord { date: "2026-07-15".to_string(), order_amount: 1000 },
                OrderRecord { date: "2026-08-12".to_string(), order_amount: 800 },
            ],
            120,    // price_ceiling: 社内方針で120まで
            "梅雨明けで揚げ物需要が増加見込み".to_string(),
        );
        let profile = Profile::Buyer(buyer_profile);
        let result1: Result<Bid, BidError> = validate_bid(&bid1, &profile);
        let result2 = validate_bid(&bid2, &profile);
        let result3 = validate_bid(&bid3, &profile);
        let result4 = validate_bid(&bid4, & profile);
        assert!(result1.is_ok());
        assert_eq!(result2, Err(BidError::NotOnGrid(101)));
        assert_eq!(result3, Err(BidError::QuantityExceedsMax(3000, 1200)));
        assert!(result4.is_ok());
    }


    #[test]
    fn test_parse_bid(){
        let text = r#"{"threshold":100,"quantity":200,"reason":"在庫が薄いため"}"#;
        let bid = parse_bid(text).unwrap();
        assert_eq!(bid.threshold, 100);
    }

  
    #[tokio::test]
    #[ignore]
    async fn test_build_call_parse(){
        let p = BuyerProfile::new(
        "在庫を切らさず、なるべく安く仕入れる".to_string(),
        800,    // current_stock: 現在庫 800L
        120,    // use_per_day: 1日120L消費
        600,    // reorder_point: 600Lを切ったら発注
        2000,   // max_stock: タンク上限 2000L
        vec![
            OrderRecord { date: "2026-07-15".to_string(), order_amount: 1000 },
            OrderRecord { date: "2026-08-12".to_string(), order_amount: 800 },
        ],
        120,    // price_ceiling: 社内方針で120まで
        "梅雨明けで揚げ物需要が増加見込み".to_string(),
        );
        let results = [];
        let history = format_history(&results, 0);
        
        let prompt = build_buyer_prompt(&p,&history);
        let text = call_claude(&prompt).await.unwrap();
        let start = text.find('{').unwrap();
        let end = text.rfind('}').unwrap();
        let target_text = &text[start..=end];
        println!("raw: {:?}", target_text);
        let bid = parse_bid(&target_text).unwrap();
        println!("{:?}", bid);
    }
    #[test]
fn test_aggregate_bids() {
    // b1: th=110 q=200, b2: th=120 q=100, b3: th=95 q=100
    // s1: th=110 q=200, s2: th=100 q=200
    let bids = vec![
        (Bid { threshold: 110, quantity: 200, reason: String::new() }, true),
        (Bid { threshold: 120, quantity: 100, reason: String::new() }, true),
        (Bid { threshold: 95,  quantity: 100, reason: String::new() }, true),
        (Bid { threshold: 110, quantity: 200, reason: String::new() }, false),
        (Bid { threshold: 100, quantity: 200, reason: String::new() }, false),
    ];
    let (price, volume, total_demand, total_supply) =
        aggregate_bids(&bids).unwrap();
    assert_eq!(price, 110);
    // total_demand = p<=110 の買い手 = b1(200)+b3(100) = 300
    // total_supply = p>=110 の売り手 = s1(200) のみ = 200
    // volume = min(300, 200) だが clearing_price は d<=s の最初の価格の d.value() を返す
    assert_eq!(volume, 300);
    assert_eq!(total_demand, 300);
    assert_eq!(total_supply, 400);
    }

    #[test]
    fn test_format_history(){
        let empty = vec![];
        assert!(format_history(&empty, 0).contains("初回"));

        let results = vec![RoundResult {
            round: 0, price: 110, volume: 1800,
            allocation: vec![0,1800,0,1800,0],
            thresholds: vec![95, 110, 105, 95, 100]
        }];
        let s = format_history(&results, 2);
        assert!(s.contains("110"));
        assert!(s.contains("105"));
    }

}
