//! agent-provenance の共有クレート。
//!
//! `mpc-rust/src/ai_agent.rs` からプロンプト生成の純粋部分を **コピー** したもの。
//! ここのテンプレ本文(`build_buyer_prompt` / `build_supplier_prompt` / `format_history`)は
//! 実パスと 1 文字も違ってはいけない。ズレると「同じ関数で再生成した」という証明の主張が壊れる。
//! (PoC のため二重管理。本番化には単一の共有クレートにする。writeup §7 参照)

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ===== ai_agent.rs からのコピー(改変禁止ゾーン) =====

pub const PRICES: [u64; 9] = [95, 100, 105, 110, 115, 120, 125, 130, 135];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Profile {
    Buyer(BuyerProfile),
    Supplier(SupplierProfile),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerProfile {
    pub strategy: String,
    pub current_stock: u64,
    pub use_per_day: u64,
    pub reorder_point: u64,
    pub max_stock: u64,
    pub order_history: Vec<OrderRecord>,
    pub price_ceiling: u64, // 社内方針: この価格までなら買っていい、という上限
    pub notes: String,      // 特記事項(自由記述)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRecord {
    pub date: String,
    pub order_amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierProfile {
    pub strategy: String,
    pub current_stock: u64,
    pub supply_per_day: u64,
    pub safety_stock: u64,
    pub max_stock: u64,
    pub supply_history: Vec<SupplyRecord>,
    pub price_floor: u64, // 社内方針: この価格を下回っては売らない、という下限
    pub notes: String,    // 特記事項(自由記述)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyRecord {
    pub date: String,
    pub supply_amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResult {
    pub round: usize,
    pub price: u64,
    pub volume: u64,
    pub allocation: Vec<u64>,
    pub thresholds: Vec<u64>,
}

pub fn format_history(results: &[RoundResult], my_index: usize) -> String {
    if results.is_empty() {
        return "(まだ取引履歴はありません。今回が初回の入札です。)".to_string();
    }
    let mut lines = Vec::new();
    for r in results {
        let prompt = format!(
            "{}ラウンド目: 清算価格: {}円, 成立総量: {}l, あなたの入札threshold: {}円, 約定量: {}l",
            r.round + 1,
            r.price,
            r.volume,
            r.thresholds[my_index],
            r.allocation[my_index]
        );
        lines.push(prompt);
    }
    lines.join("\n")
}

pub fn build_buyer_prompt(profile: &BuyerProfile, history: &str) -> String {
    let prices = PRICES;
    let strategy: &String = &profile.strategy;
    let current_stock: u64 = profile.current_stock;
    let use_per_day: u64 = profile.use_per_day;
    let reorder_point: u64 = profile.reorder_point;
    let max_stock: u64 = profile.max_stock;
    let order_history: String = profile
        .order_history
        .iter()
        .map(|o| format!("{}:{}l", o.date, o.order_amount))
        .collect::<Vec<_>>()
        .join(",");
    let price_ceiling: u64 = profile.price_ceiling;
    let notes: &String = &profile.notes;
    let prompt = format!(
        r#"
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
        "#
    );
    prompt
}

pub fn build_supplier_prompt(profile: &SupplierProfile, history: &str) -> String {
    let prices = PRICES;
    let strategy: &String = &profile.strategy;
    let current_stock: u64 = profile.current_stock;
    let supply_per_day: u64 = profile.supply_per_day;
    let safety_stock: u64 = profile.safety_stock;
    let max_stock: u64 = profile.max_stock;
    let supply_history: String = profile
        .supply_history
        .iter()
        .map(|o| format!("{}:{}l", o.date, o.supply_amount))
        .collect::<Vec<_>>()
        .join(",");
    let price_floor: u64 = profile.price_floor;
    let notes: &String = &profile.notes;
    let prompt = format!(
        r#"
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
        "#
    );
    prompt
}

// ===== ここから provenance 用の新規ヘルパー(あなたが埋める) =====

/// プロフィールの決定的バイト列。host と guest で必ず同じ結果になる必要があるので bincode。
pub fn canonical_bytes(profile: &Profile) -> Vec<u8> {
    bincode::serialize(profile).expect("serialize profile")
}

/// コミットメント = sha256(canonical(profile) ++ salt)
pub fn commit_profile(profile: &Profile, salt: &[u8; 32]) -> [u8; 32] {
    // TODO:
    //   let mut h = Sha256::new();
    //   h.update(canonical_bytes(profile));
    //   h.update(salt);
    //   h.finalize().into()
    todo!()
}

/// propose_bid の match と同じ振り分けで prompt を 1 本化する。
pub fn render_prompt(profile: &Profile, history: &str) -> String {
    // TODO:
    //   match profile {
    //       Profile::Buyer(b) => build_buyer_prompt(b, history),
    //       Profile::Supplier(s) => build_supplier_prompt(s, history),
    //   }
    todo!()
}

/// sha256 ヘルパー(guest / host 共用)
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}
