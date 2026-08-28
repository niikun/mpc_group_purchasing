use std::env;
use mpc_rust::simulate::{Trader, run_round};

fn main() {
    let args: Vec<String> = env::args().collect();
    // 使い方: cargo run --bin simulate_demo -- [aggressive|passive] [ラウンド数]
    let aggressive = args.get(1).map(|s| s == "aggressive").unwrap_or(false);
    let n_rounds: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(15);

    // println!("mode: {}, rounds: {}", if aggressive { "aggressive" } else { "passive" }, n_rounds);

    // Buyer: is_buyer, true_value(本当の評価額), threshold(最初の入札), quantity
    // Seller: is_buyer=false, true_value(本当の原価), threshold(最初の入札), quantity
    let mut traders = vec![
        Trader::new(true, 100, 80, 100),
        Trader::new(true, 110, 90, 200),
        Trader::new(true, 105, 95, 300),
        Trader::new(false, 90, 110, 400),
        Trader::new(false, 100, 115, 500),
    ];
    println!("round,trader,threshold,price,traded");

    run_round(&mut traders, n_rounds, aggressive);
}
