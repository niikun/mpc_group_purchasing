use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct User{
    name: String,
    age: u32,
}

fn main(){
    let user = User{
        name:"Taro".to_string(),
        age:59,
    };

    let json = serde_json::to_string(&user).unwrap();
    println!("{}",json);

    let user2:User = serde_json::from_str(&json).unwrap();
    println!("{:?}",user2);
}