use anyhow::Result;
use prost::Message;

mod protos;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");
    // let mut shirt = items::Shirt {
    //     color: "red".to_string(),
    //     ..Default::default()
    // };
    // shirt.set_size(Size::Large);

    // let mut buf = Vec::with_capacity(shirt.encoded_len());
    // shirt.encode(&mut buf)?;

    // println!("{:?}", shirt);
    // println!("{:?}", buf);
    protos::Task::default();
    Ok(())
}
