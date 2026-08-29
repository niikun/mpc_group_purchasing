use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint16, FheBool};
use tfhe::prelude::*;

fn main(){
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    let demand = FheUint16::encrypt(700u16, &client_key);
    let supply = FheUint16::encrypt(500u16, &client_key);

    let cond:FheBool = demand.le(&supply);
    let min = demand.min(&supply);

    let less_side = cond.if_then_else(&demand,  &supply);


    let clear_less_side:u16 = less_side.decrypt( &client_key);
    let clear_min:u16 = min.decrypt(&client_key);
    println!("less_side :{}",clear_less_side);
    println!("min:{}",clear_min);
}