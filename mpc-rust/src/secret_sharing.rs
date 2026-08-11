// 加法的秘密分散(spec_v0.4.md §8.1 / DESIGN.md §5)
//
// 値 v を、法M上で3つのシェアに分割する。
//   r1, r2 <- [0, M) から一様ランダム
//   r3 = (v - r1 - r2) mod M
//   (r1 + r2 + r3) mod M == v が常に成立する

use rand::RngExt;
use std::ops::{Add, Sub, Mul};

/// 法M。実際に扱う需要合計・供給合計の最大値より十分大きい値にする。
pub const M: u64 = (1u64 << 31) - 1; // 2^31 - 1 (メルセンヌ素数)

#[derive(Debug, Clone, Copy, PartialEq, Eq,PartialOrd)]
pub struct Fp{
    value:u64,
}
    

impl Fp {
    pub fn new(value: u64) -> Self {
        Fp{
            value: value % M,
        }
    }

    pub fn zero() -> Fp {
        Fp::new(0)
    }

    pub fn one() -> Fp {
        Fp::new(1)
    }
    pub fn value(&self) -> u64 {
        self.value
    }

}

impl Add for Fp {
    type Output = Fp;
    fn add(self,other:Fp) -> Fp{
        let new_value = (self.value + other.value + M) % M;
        Fp::new(new_value)
    }
} 

impl Sub for Fp {
    type Output = Fp;
    fn sub(self,other:Fp) -> Fp{
        let new_value = (self.value + M - other.value) % M;
        Fp::new(new_value)
    }
}

impl Mul for Fp {
    type Output = Fp;
    fn mul(self, other: Fp) -> Fp {
        let new_value = (self.value * other.value) % M;
        Fp::new(new_value)
    }
}

impl Fp {
    pub fn pow(&self, exp: u64) -> Fp {
        let mut result = Fp::one();
        let mut base = *self;
        let mut exp = exp;

        while exp > 0 {
            if exp&1 == 1 {
                result = result.mul(base);
            }
            base = base.mul(base);
            exp >>= 1;
        }
        result
    }

    pub fn inverse(&self) -> Option<Fp> {
        match self.value {
            0 => None,
            _ => Some(self.pow(M - 2)),
        }
    }
}

pub type Share = Fp;

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
pub fn split_into_shares(v:Fp) -> [Share; 3] {
    let mut rng = rand::rng();
    let r1 = Fp::new(rng.random_range(0..M));
    let r2 = Fp::new(rng.random_range(0..M));
    let r3 = v - r1 - r2;
    [r1,r2,r3]
}

pub fn reconstruct(shares:&[Fp;3]) -> Fp {
    shares[0] + shares[1] + shares[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_sum_to_original_value_mod_m() {
        let v = Fp::new(350u64);
        let shares = split_into_shares(v);
        let sum: Fp = shares[0]  + shares[1] + shares[2] ;
        assert_eq!(sum, v);
    }

    #[test]
    fn fp_addition_and_subtraction() {
        let a = Fp::new(10);
        let b = Fp::new(20);
        let c = Fp::new(M - 5);

        assert_eq!((a + b).value(), 30);
        assert_eq!((a - b).value(), (M + 10 - 20) % M);
        assert_eq!((b - a).value(), 10);
        assert_eq!((c + a).value(), (M - 5 + 10) % M);
    }   

    #[test]
    fn fp_multiplication() {
        let a = Fp::new(10);
        let b = Fp::new(20);
        assert_eq!((a * b).value(), 200);
    }

    #[test]
    fn fp_inverse() {
        let a = Fp::new(3);
        let inv_a = a.inverse().unwrap();
        assert_eq!((a * inv_a).value(), 1); 
    }

    #[test]
    fn test_reconstruct(){
        let v = Fp::new(24);
        let shares = split_into_shares(v);
        let reconstructed = reconstruct(&shares);
        assert_eq!(v, reconstructed);
    }


}