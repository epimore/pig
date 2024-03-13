use common::net;
use std::net::SocketAddr;
use std::str::FromStr;
use bytes::Bytes;

//cmd: cargo run --example single_tcp --features net
#[tokio::main]
async fn main() {
    let _tripe = common::init();
    let (tx, mut rx) = net::init_net(net::shard::Protocol::TCP, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let mut i = 0;
    while let Some(mut package) = rx.recv().await {
        println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
        i += 1;
        package.data = Bytes::from(format!("abc - {}", i));
        let _ = tx.clone().send(package).await;
    }
}