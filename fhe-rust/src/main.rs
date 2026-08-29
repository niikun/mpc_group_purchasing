use std::time::Instant;

use tfhe::{ConfigBuilder, FheUint8, PublicKey, generate_keys, set_server_key};
use tfhe::prelude::*;

fn main() {
    let t = Instant::now();
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    println!("genarate key:{:?}",t.elapsed());
    let t = Instant::now();
    let public_key = PublicKey::new(&client_key);
    println!("genarate public key:{:?}",t.elapsed());
    let clear_a = 27u8;
    let clear_b = 128u8;
    let t = Instant::now();
    let a = FheUint8::encrypt(clear_a, &public_key);
    let b = FheUint8::encrypt(clear_b, &public_key);
    println!("encrypt:{:?}",t.elapsed());

    set_server_key(server_key);
    let t = Instant::now();
    let result = a + b;
    println!("clculate:{:?}",t.elapsed());    

    let t = Instant::now();
    let decrypted_result:u8 = result.decrypt(&client_key);
    println!("decrypt:{:?}",t.elapsed());
    println!("result:{}", decrypted_result);

    let clear_result = clear_a + clear_b;

    assert_eq!(decrypted_result, clear_result);
}
