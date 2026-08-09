// 加法的秘密分散(spec_v0.4.md §8.1 / DESIGN.md §5)
//
// 値 v を、法M上で3つのシェアに分割する。
//   r1, r2 <- [0, M) から一様ランダム
//   r3 = (v - r1 - r2) mod M
//   (r1 + r2 + r3) mod M == v が常に成立する

use rand::Rng;

/// 法M。実際に扱う需要合計・供給合計の最大値より十分大きい値にする。
pub const M: u64 = (1u64 << 31) - 1; // 2^31 - 1 (メルセンヌ素数)

pub type Share = u64;

/// 値 v を3つのシェアに分割する。
///
/// TODO:
/// 1. r1, r2 を rand::rng().random_range(0..M) で生成する
/// 2. r3 = (v - r1 - r2) mod M を計算する
///
/// ヒント: u64 のまま `v - r1 - r2` を計算すると、
/// 結果が負になるケースでアンダーフロー(パニック)する。
/// v, r1, r2 はすべて [0, M) の範囲にあることを踏まえて、
/// 「M表現内での引き算」をどう安全に行うか考えること。
/// (i128 に一度持ち上げる / M を足してからmodを取る、などいくつかやり方がある)
pub fn split_into_shares(v: u64) -> [Share; 3] {
    todo!("v を3シェアに分割する")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_sum_to_original_value_mod_m() {
        let v = 350u64;
        let shares = split_into_shares(v);
        let sum: u64 = ((shares[0] as u128 + shares[1] as u128 + shares[2] as u128) % M as u128) as u64;
        assert_eq!(sum, v);
    }
}
