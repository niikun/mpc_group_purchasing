use tfhe::{ClientKey, ConfigBuilder, FheUint16, generate_keys, set_server_key};
use tfhe::prelude::*;

pub const PRICES: [u64; 9] = [95,100,105,110,115,120,125,130,135];

pub fn main(){
    let config = ConfigBuilder::default().build();
    let  (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

}

pub fn cordinator(threshold:u16, quantity:u16, client_key:&ClientKey)->(FheUint16,FheUint16){
    let threshold_encrypted = FheUint16::encrypt(threshold, client_key);
    let quantity_encrypted = FheUint16::encrypt(quantity, client_key);
    (threshold_encrypted, quantity_encrypted)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_cordinate(){
        let config = ConfigBuilder::default().build();
        let  (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);
        let threshold = 120u16;
        let quantity:u16 = 100u16;
        let (crypted_threshold, crypted_quantity) = cordinator(threshold, quantity, &client_key);
        let clear_threshold:u16 = crypted_threshold.decrypt(&client_key);
        let clear_quantity:u16 = crypted_quantity.decrypt(&client_key);
        assert_eq!(clear_threshold, threshold);
        assert_eq!(clear_quantity, quantity);
    }
}
