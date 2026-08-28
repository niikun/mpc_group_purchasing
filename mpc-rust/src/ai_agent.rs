use reqwest;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

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

#[tokio::main]
pub async fn send_for_agent() -> std::io::Result<()>{

    Ok(())
}