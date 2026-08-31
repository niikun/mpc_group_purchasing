use serde_json::{json, Value};

const API_URL:&str = "https://api.anthropic.com/v1/messages";
const MODEL:&str = "claude-haiku-4-5";

#[tokio::main]
pub async fn main() ->Result<(), Box<dyn std::error::Error>>{
    let out = mpc_rust::ai_agent::call_claude(
        "Reply with the single word: pong"
    ).await?;
    println!("{}",out);
    Ok(())
}