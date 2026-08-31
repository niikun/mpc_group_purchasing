use reqwest;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

use std::collections::HashMap;
const API_URL:&str = "https://api.anthropic.com/v1/messages";
const MODEL:&str = "claude-haiku-4-5";

pub struct BuyerProfile{
    current_stock:u64,
    use_per_day:u64,
    reorder_point:u64,
    max_stock:u64,
    order_history:Vec<OrderRecord>,
    price_ceiling:u64, // 社内方針: この価格までなら買っていい、という上限
    notes:String,      // 特記事項(自由記述)
}

pub struct OrderRecord{
    date:String,
    order_amount:u64, 
}

pub struct SupplierProfile{
    current_stock:u64,
    supply_per_day:u64,
    reorder_point:u64,
    max_stock:u64,
    supply_history:Vec<SupplyRecord>,
    price_floor:u64, // 社内方針: この価格を下回っては売らない、という下限
    notes:String,    // 特記事項(自由記述)
}

pub struct SupplyRecord{
    date:String,
    supply_amount:u64, 
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