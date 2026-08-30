// MPC ベンチハーネス
//   buyer3 + seller2 の「1取引」= aggrigate_market を通しで 1000 回回し、
//   実行時間の中央値 / 最小 / p90 / 最大を出す。
//
//   実行: cargo run --release --bin bench
//   (必ず --release。debug ビルドの数値は無意味)

use std::time::{Duration, Instant};

use mpc_rust::auction::aggrigate_market;

// 固定シナリオ = auction.rs の test_aggrigate_market3 と同じ。
// 期待結果: 清算価格 110, 約定 (b1=200, b2=100, b3=0, s1=150, s2=150)
const TH_B1: u64 = 110;
const TH_B2: u64 = 120;
const TH_B3: u64 = 95;
const TH_S1: u64 = 110;
const TH_S2: u64 = 100;
const Q_B1: u64 = 200;
const Q_B2: u64 = 100;
const Q_B3: u64 = 100;
const Q_S1: u64 = 200;
const Q_S2: u64 = 200;

fn one_trade() -> Option<(u64, (u64, u64, u64, u64, u64))> {
    aggrigate_market(
        TH_B1, TH_B2, TH_B3, TH_S1, TH_S2, Q_B1, Q_B2, Q_B3, Q_S1, Q_S2,
    )
}

fn stats(label: &str, mut samples: Vec<Duration>) {
    samples.sort();
    let n = samples.len();
    let median = samples[n / 2];
    let p90 = samples[(n * 9) / 10];
    println!(
        "{label}: median={median:?}  min={:?}  p90={p90:?}  max={:?}  (n={n})",
        samples[0],
        samples[n - 1],
    );
}

fn main() {
    const WARMUP: usize = 20;
    const RUNS: usize = 1000;

    // 正しく計算できているか1回だけ確認(タイミング区間外)
    let sample = one_trade();
    println!("sanity: {sample:?}");
    assert_eq!(
        sample,
        Some((110, (200, 100, 0, 150, 150))),
        "aggrigate_market の結果が想定と違う"
    );

    for _ in 0..WARMUP {
        std::hint::black_box(one_trade());
    }

    let mut samples = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t = Instant::now();
        let out = one_trade();
        samples.push(t.elapsed());
        std::hint::black_box(out);
    }

    println!("--- MPC 1取引 (buyer3 + seller2, aggrigate_market 通し) ---");
    println!(
        "build: {}",
        if cfg!(debug_assertions) {
            "debug  (!!! --release で測り直すこと)"
        } else {
            "release"
        }
    );
    stats("full trade", samples);
}
