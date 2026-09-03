use sha2::{Sha256, Digest};

fn main(){
    let mut h = Sha256::new();
    let text1 = b"hello";
    let text2 = b" world";

    println!("{:?}", text1);
    println!("{:?}", text2);

    h.update(text1);
    h.update(text2);
    let out:[u8;32] = h.finalize().into();

    let mut h = Sha256::new();
    h.update(b"hello world");
    let out2:[u8;32] = h.finalize().into();
    println!("{:?}",out);
    println!("{:?}",out2);
    assert_eq!(out,out2);
}