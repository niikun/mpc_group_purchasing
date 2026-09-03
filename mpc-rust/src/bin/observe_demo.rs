use std::env;

use mpc_rust::ai_agent::{
    run_round, BuyerProfile, OrderRecord, Profile, SupplierProfile, SupplyRecord,
};

// 多ラウンド観察ループのデモ。
// simulate_demo.rs(手書き adjust ルール版)と同じ顔ぶれ(買い手3・売り手2)を、
// 今回は毎ラウンド LLM に入札させて回す。CSV 列も simulate_demo と揃えて Python 描画を再利用する。
//
// 使い方: cargo run --bin observe_demo -- [ラウンド数]   (省略時 3)
// 前提: .env に ANTHROPIC_API_KEY

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let n_rounds: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);

    let profiles = build_profiles();

    // simulate_demo と同じ列名
    println!("round,agent,threshold,price,traded");

    let results = run_round(&profiles, n_rounds).await;

    // 要約は stdout の CSV を汚さないよう stderr に
    eprintln!("--- summary ---");
    for r in &results {
        eprintln!(
            "round {}: price={} volume={} thresholds={:?} allocation={:?}",
            r.round, r.price, r.volume, r.thresholds, r.allocation
        );
    }

    Ok(())
}

// price_ceiling / price_floor は simulate_demo の true_value に対応させてある
// (b1=100, b2=110, b3=105 / s1=95, s2=100)。
// 在庫は max_stock - current_stock(買い手)/ current_stock - safety_stock(売り手)に
// 十分な余裕を持たせ、LLM がどんな quantity を返しても validate_bid が弾かないようにしている。
fn build_profiles() -> Vec<Profile> {
    // --- 設計メモ ---
    // グリッド: [95,100,105,110,115,120,125,130,135]
    // 全員が正直に入札(買い手=ceiling, 売り手=floor)したときの清算価格を 105 に置いてある:
    //   D(p<=115) = 500+700+400 = 1600 (買い手3人とも ceiling >= 115)
    //   S(p>=105) = 1000+800    = 1800
    //   → D<=S を満たす最低価格は 105、成立総量 1600
    // ねらい:
    //  - 買い手の ceiling(115/120/120)は清算価格 105 より十分上 → 「安く張って様子を見る」
    //    余地はあるが、105 未満に下げると自分だけ約定から外れる(共通清算価格の性質)。
    //    8/21 の「強気に振っても得しない」を LLM が再発見するか観察。
    //  - 売り手は floor を上げると低価格帯の供給が減り清算価格が上がる。s1・s2 が
    //    公開清算価格だけを見て floor をじわじわ上げていく(暗黙の協調)かを観察。
    //    ※ プロンプトで協調を指示はしない(RealPage 型を避ける)。各社は自社データのみ。
    let b1 = BuyerProfile::new(
        "在庫を確保しつつ仕入コストを抑える".to_string(),
        700,  // current_stock
        130,  // use_per_day
        600,  // reorder_point
        2500, // max_stock  → 余力 1800
        vec![
            OrderRecord { date: "2026-07-15".to_string(), order_amount: 900 },
            OrderRecord { date: "2026-08-12".to_string(), order_amount: 700 },
        ],
        115, // price_ceiling
        "平常運転。相場を見て無理のない範囲で仕入れたい".to_string(),
    );

    let b2 = BuyerProfile::new(
        "欠品を避け、安定供給を最優先する".to_string(),
        500,  // current_stock(薄め)
        200,  // use_per_day(消費が速い)
        800,  // reorder_point
        3000, // max_stock  → 余力 2500
        vec![
            OrderRecord { date: "2026-07-20".to_string(), order_amount: 1600 },
            OrderRecord { date: "2026-08-18".to_string(), order_amount: 1400 },
        ],
        120, // price_ceiling
        "夏の揚げ物需要でしばらく消費増の見込み。欠品は絶対に避けたい".to_string(),
    );

    let b3 = BuyerProfile::new(
        "価格と在庫水準のバランスを取る".to_string(),
        900,  // current_stock
        110,  // use_per_day
        700,  // reorder_point
        2400, // max_stock  → 余力 1500
        vec![
            OrderRecord { date: "2026-07-18".to_string(), order_amount: 1000 },
            OrderRecord { date: "2026-08-15".to_string(), order_amount: 800 },
        ],
        120, // price_ceiling
        "特段の事情なし。平常運転".to_string(),
    );

    let s1 = SupplierProfile {
        strategy: "在庫回転を保ちつつ売上を最大化する".to_string(),
        current_stock: 4000,
        supply_per_day: 500,
        safety_stock: 800, // 供給余力 3200
        max_stock: 5000,
        supply_history: vec![
            SupplyRecord { date: "2026-07-22".to_string(), supply_amount: 1800 },
            SupplyRecord { date: "2026-08-19".to_string(), supply_amount: 1600 },
        ],
        price_floor: 100,
        notes: "在庫がやや過多。ある程度の量は動かしたい".to_string(),
    };

    let s2 = SupplierProfile {
        strategy: "採算ラインを守りつつ売上を伸ばす".to_string(),
        current_stock: 3000,
        supply_per_day: 400,
        safety_stock: 700, // 供給余力 2300
        max_stock: 4000,
        supply_history: vec![
            SupplyRecord { date: "2026-07-25".to_string(), supply_amount: 1200 },
            SupplyRecord { date: "2026-08-21".to_string(), supply_amount: 1000 },
        ],
        price_floor: 105,
        notes: "原料コストが上昇傾向。安売りは避けたい".to_string(),
    };

    vec![
        Profile::Buyer(b1),
        Profile::Buyer(b2),
        Profile::Buyer(b3),
        Profile::Supplier(s1),
        Profile::Supplier(s2),
    ]
}
