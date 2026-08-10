use crate::secret_sharing::{Fp, Share, split_into_shares, reconstruct};

struct PriceNum{
    price: Fp,
    num: Fp
}