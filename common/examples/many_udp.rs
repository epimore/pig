use common::net;
use std::net::SocketAddr;
use std::str::FromStr;
use bytes::Bytes;
use log::error;
use common::err::TransError;

//cmd: cargo run --example many_udp --features net
#[tokio::main]
async fn main() {
    let _tripe = common::init();
    let (tx1, mut rx1) = net::init_net(net::shard::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18887").unwrap()).await.unwrap();
    let (tx2, mut rx2) = net::init_net(net::shard::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let (tx3, mut rx3) = net::init_net(net::shard::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18889").unwrap()).await.unwrap();
    let mut i = 0;
    tokio::spawn(
        async move {
            while let Some(mut package) = rx1.recv().await {
                println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
                i += 1;
                let tmp = package.bill.to;
                package.bill.to=package.bill.from;
                package.bill.from=tmp;
                package.data = Bytes::from(format!("abc - {}", i));
                println!("{:?}",&package);
                let _ = tx1.clone().send(package).await.hand_err(|msg|error!("{msg}"));
            }
        }
    );
    tokio::spawn(
        async move {
            while let Some(mut package) = rx2.recv().await {
                println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
                i += 1;
                let tmp = package.bill.to;
                package.bill.to=package.bill.from;
                package.bill.from=tmp;
                package.data = Bytes::from(format!("abc - {}", i));
                println!("{:?}",&package);
                let _ = tx2.clone().send(package).await.hand_err(|msg|error!("{msg}"));
            }
        }
    );
    while let Some(mut package) = rx3.recv().await {
        println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
        i += 1;
        let tmp = package.bill.to;
        package.bill.to=package.bill.from;
        package.bill.from=tmp;
        package.data = Bytes::from(format!("abc - {}", i));
        println!("{:?}",&package);
        let _ = tx3.clone().send(package).await.hand_err(|msg|error!("{msg}"));
    }
}