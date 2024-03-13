use common::net;
use std::net::SocketAddr;
use std::str::FromStr;
use bytes::Bytes;
use log::error;
use common::err::TransError;
use common::net::shard::Bill;

//cmd: cargo run --example single_all --features net
#[tokio::main]
async fn main() {
    let _tripe = common::init();
    let (tx, mut rx) = net::init_net(net::shard::Protocol::ALL, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let mut i = 0;
    while let Some(mut package) = rx.recv().await {
        println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
        if package.bill.protocol.eq(&net::shard::Protocol::UDP) {
            let tmp = package.bill.to;
            package.bill.to=package.bill.from;
            package.bill.from=tmp;
        }
        i += 1;
        package.data = Bytes::from(format!("abc - {}", i));
        println!("{:?}",&package);
        let _ = tx.clone().send(package).await.hand_err(|msg|error!("{msg}"));
    }
}