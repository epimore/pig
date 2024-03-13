use common::net;
use std::net::SocketAddr;
use std::str::FromStr;

//cmd: cargo run --example many_udp_tcp --features net
#[tokio::main]
async fn main() {
    let _tripe = common::init();
    let (tx0, mut rx0) = net::init_net(net::shard::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18886").unwrap()).await.unwrap();
    let (tx1, mut rx1) = net::init_net(net::shard::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18887").unwrap()).await.unwrap();
    let (tx2, mut rx2) = net::init_net(net::shard::Protocol::TCP, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let (tx3, mut rx3) = net::init_net(net::shard::Protocol::TCP, SocketAddr::from_str("0.0.0.0:18889").unwrap()).await.unwrap();
    tokio::spawn(
        async move {
            while let Some(mut package) = rx0.recv().await {
                let tmp = package.bill.to;
                package.bill.to=package.bill.from;
                package.bill.from=tmp;
                println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
                let _ = tx0.clone().send(package).await;
            }
        }
    );
    tokio::spawn(
        async move {
            while let Some(mut package) = rx1.recv().await {
                let tmp = package.bill.to;
                package.bill.to=package.bill.from;
                package.bill.from=tmp;
                println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
                let _ = tx1.clone().send(package).await;
            }
        }
    );
    tokio::spawn(
        async move {
            while let Some(package) = rx2.recv().await {
                println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
                let _ = tx2.clone().send(package).await;
            }
        }
    );
    while let Some(package) = rx3.recv().await {
        println!("bill = {:?} - data_size: {}", package.bill, package.data.len());
        let _ = tx3.clone().send(package).await;
    }
}