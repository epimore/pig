use common::net;
use std::net::SocketAddr;
use std::str::FromStr;
use bytes::Bytes;
use log::error;
use common::err::TransError;
use common::net::shard::{Zip};

//cmd: cargo run --example single_all --features net
#[tokio::main]
async fn main() {
    let _tripe = common::init();
    let (tx, mut rx) = net::init_net(net::shard::Protocol::ALL, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let mut i = 0;
    while let Some(zip) = rx.recv().await {
        match zip {
            Zip::Data(mut package) => {
                println!("bill = {:?} - data_size: {}", package.get_bill(), package.get_data().len());
                if package.get_bill().get_protocol().eq(&net::shard::Protocol::UDP) {
                    let mut bill = package.get_bill().clone();
                    bill.set_to(package.get_bill().get_from().clone());
                    bill.set_from(package.get_bill().get_to().clone());
                    package.set_bill(bill);
                }
                i += 1;
                package.set_data(Bytes::from(format!("abc - {}", i)));
                println!("{:?}",&package);
                let _ = tx.clone().send(Zip::build_data(package)).await.hand_err(|msg|error!("{msg}"));
            }
            Zip::Event(_) => {}
        }
    }
}