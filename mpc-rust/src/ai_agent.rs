use reqwest;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

use std::collections::HashMap;
use crate::auction::PRICES;

const API_URL:&str = "https://api.anthropic.com/v1/messages";
const MODEL:&str = "claude-haiku-4-5";
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
    order_amount:u64, 
}

#[derive(Debug, Clone)]
pub struct SupplierProfile{
    pub strategy:String,
    pub current_stock:u64,
    pub supply_per_day:u64,
    pub reorder_point:u64,
    pub max_stock:u64,
    pub supply_history:Vec<SupplyRecord>,
    pub price_floor:u64, // 社内方針: この価格を下回っては売らない、という下限
    notes:String,    // 特記事項(自由記述)
}

#[derive(Debug, Clone)]
pub struct SupplyRecord{
    pub date:String,
    pub supply_amount:u64, 
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
        現在のストック量: {current_stock}
        1日の消費量: {use_per_day},
        再購入のポイント:{reorder_point},
        最大材御量:{max_stock},
        過去の購入履歴:{order_history},
        購入価格上限:{price_ceiling}
        特記事項:{notes}
        ### 出力形式
        以下の例のようなjsonのみ。余計な文章は加えないでください。
        {{"threshold":100, "quantity":200, "reason":"在庫に対して、使用量のため..."}}
        thresholdは購入上限の価格です
        quantityは購入する数量です
        reasonは判断した理由を50字以内で説明してください
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

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn print_build_buyer_prompt(){
        let p = BuyerProfile::new(
            "在庫を切らさないようにする".to_string(),
            100,
            10,
            20,
            200,
            vec![],
            150,
            "特記事項なし".to_string()
        );
        let prompt = build_buyer_prompt(&p);
        println!("{}",prompt);
    }
}
