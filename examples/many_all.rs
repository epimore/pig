use pig::net;
use std::net::SocketAddr;
use std::str::FromStr;
use bytes::Bytes;
use log::error;
use pig::err::TransError;
use pig::net::shard::Bill;

//cmd: cargo run --example many_all --features net
#[tokio::main]
async fn main() {
    let _tripe = pig::init();
    let (tx, mut rx) = net::init_net(net::shard::Protocol::ALL, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let (tx1, mut rx1) = net::init_net(net::shard::Protocol::ALL, SocketAddr::from_str("0.0.0.0:18889").unwrap()).await.unwrap();
    tokio::spawn(async move{
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
    });

    let mut i = 0;
    while let Some(mut package) = rx1.recv().await {
        println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
        if package.bill.protocol.eq(&net::shard::Protocol::UDP) {
            let tmp = package.bill.to;
            package.bill.to=package.bill.from;
            package.bill.from=tmp;
        }
        i += 1;
        package.data = Bytes::from(format!("abc - {}", i));
        println!("{:?}",&package);
        let _ = tx1.clone().send(package).await.hand_err(|msg|error!("{msg}"));
    }
}