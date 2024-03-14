use common::net;
use std::net::SocketAddr;
use std::str::FromStr;
use common::net::shard::Zip;

//cmd: cargo run --example single_tcp --features net
#[tokio::main]
async fn main() {
    let _tripe = common::init();
    let (tx, mut rx) = net::init_net(net::shard::Protocol::TCP, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    while let Some(zip) = rx.recv().await {
        match zip {
            Zip::Data(ref package) => {
                println!("bill = {:?} - data_size: {}", package.get_bill(), package.get_data().len());
                let _ = tx.clone().send(zip).await;
            }
            Zip::Event(ref event) => {
                println!("event type code = {}",event.get_type_code());
            }
        }
    }
}