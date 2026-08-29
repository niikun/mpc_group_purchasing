use tfhe::{ClientKey, ConfigBuilder, FheUint16, generate_keys, set_server_key, FheBool};
use tfhe::prelude::*;

pub const PRICES: [u16; 9] = [95u16,100u16,105u16,110u16,115u16,120u16,125u16,130u16,135u16];

pub fn main(){
    let config = ConfigBuilder::default().build();
    let  (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);
}

pub fn cordinator(threshold:u16, quantity:u16, client_key:&ClientKey)->(FheUint16,FheUint16){
    let threshold_enc = FheUint16::encrypt(threshold, client_key);
    let quantity_enc = FheUint16::encrypt(quantity, client_key);
    (threshold_enc, quantity_enc)
}

pub fn fhe_derive(
    threshold:&FheUint16, 
    quantity:&FheUint16,
    is_buyer:bool,
    zero:&FheUint16
) ->[FheUint16;9]{
    
    let price_quantity:[FheUint16;9] = core::array::from_fn(|i|{
        if is_buyer{
            let cond:FheBool = threshold.ge(PRICES[i]);
            let slot:FheUint16 = cond.if_then_else(quantity, zero);
            slot
        } else {
            let cond:FheBool =  threshold.le(PRICES[i]);
            let slot:FheUint16 = cond.if_then_else(quantity,zero);
            slot
        }
    }); 
    price_quantity
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_fhe_derive(){
        let config = ConfigBuilder::default().build();
        let  (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);
        let threshold = 120u16;
        let quantity:u16 = 100u16;
        let zero = FheUint16::encrypt(0u16, &client_key);
        let (threshold_enc, quantity_enc) = cordinator(threshold, quantity, &client_key);
        let price_quantity1 = fhe_derive(&threshold_enc, &quantity_enc,true,&zero);
        let price_quantity2 = fhe_derive(&threshold_enc, &quantity_enc,false,&zero);
        let clear_pq1:Vec<u16> = price_quantity1.iter().map(|v| v.decrypt(&client_key)).collect();
        let clear_pq2:Vec<u16> = price_quantity2.iter().map(|v| v.decrypt(&client_key)).collect();
        assert_eq!(clear_pq1, [quantity, quantity,quantity, quantity,quantity, quantity, 0, 0, 0]);
        assert_eq!(clear_pq2, [0, 0, 0, 0, 0, quantity, quantity,quantity, quantity]);
    }
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
