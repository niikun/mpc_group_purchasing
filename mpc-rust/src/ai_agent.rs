use reqwest;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auction::PRICES;

const API_URL:&str = "https://api.anthropic.com/v1/messages";
const MODEL:&str = "claude-haiku-4-5";

pub enum Profile{
    Buyer(BuyerProfile),
    Supplier(SupplierProfile),
}

#[derive(Debug)]
pub enum BidError{
    NotOnGrid(u64),
    QuantityExceedsMax(u64,u64),
    QuantityZero,
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

#[derive(Debug, Clone, Deserialize)]
pub struct Bid{
    pub threshold: u64,
    pub quantity:u64,
    pub reason:String,
}

fn build_buyer_prompt(profile:&BuyerProfile)->String{
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
        ### 出力形式
        以下の例のようなjsonのみ。余計な文章は加えないでください。
        {{"threshold":100, "quantity":200, "reason":"在庫に対して、使用量のため..."}}
        thresholdは購入上限の価格です。pricesの中の数字を必ず設定してください。
        quantityは購入する数量です。max_stock - current_stock を超えない。
        reasonは判断した理由を50字以内で説明してください
        "#);
    prompt
}

fn build_supplier_prompt(profile:&SupplierProfile)->String{
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
        ### 出力形式
        以下の例のようなjsonのみ。余計な文章は加えないでください。
        {{"threshold":100, "quantity":200, "reason":"在庫に対して、使用量のため..."}}
        thresholdは販売下限の価格です。pricesの中の数字を必ず設定してください。
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
            } else if bid.quantity == 0{
                Err(BidError::QuantityZero)
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
            } else if bid.quantity == 0{
                Err(BidError::QuantityZero)
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

pub async fn propose_bid(profile:&Profile) -> Result<Bid, Box<dyn std::error::Error>>{
    match profile {
        Profile::Buyer(buyer_profile) =>{
            let prompt = build_buyer_prompt(&buyer_profile);
            let text = call_claude(&prompt).await.unwrap();
            let bid = parse_bid(&text)?;
            Ok(bid)
        },
        Profile::Supplier(supplier_profile) =>{
            let prompt = build_supplier_prompt(&supplier_profile);
            let text = call_claude(&prompt).await.unwrap();
            let bid = parse_bid(&text)?;
            Ok(bid)
        }        
    }
}

#[cfg(test)]
mod tests{
    use crate::node::Branch::Buyer;

use super::*;

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
        
        let prompt = build_buyer_prompt(&p);
        let text = call_claude(&prompt).await.unwrap();
        let start = text.find('{').unwrap();
        let end = text.rfind('}').unwrap();
        let target_text = &text[start..=end];
        println!("raw: {:?}", target_text);
        let bid = parse_bid(&target_text).unwrap();
        println!("{:?}", bid);
    }
}
