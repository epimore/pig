use pig::net;
use std::net::SocketAddr;
use std::str::FromStr;
use bytes::Bytes;
use log::error;
use pig::err::TransError;

//cmd: cargo run --example single_udp --features net
#[tokio::main]
async fn main() {
    let _tripe = pig::init();
    let (tx, mut rx) = net::init_net(net::shard::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let mut i = 0;
    while let Some(mut package) = rx.recv().await {
        println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
        i += 1;
        let tmp = package.bill.to;
        package.bill.to=package.bill.from;
        package.bill.from=tmp;
        package.data = Bytes::from(format!("abc - {}", i));
        println!("{:?}",&package);
        let _ = tx.clone().send(package).await.hand_err(|msg|error!("{msg}"));
    }
}