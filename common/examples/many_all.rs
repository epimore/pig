use common::net;
use std::net::SocketAddr;
use std::str::FromStr;
use bytes::Bytes;
use log::error;
use common::err::TransError;
use common::net::shared::{Zip};

//cmd: cargo run --example many_all --features net
#[tokio::main]
async fn main() {
    let _tripe = common::init();
    let (tx, mut rx) = net::init_net(net::shared::Protocol::ALL, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let (tx1, mut rx1) = net::init_net(net::shared::Protocol::ALL, SocketAddr::from_str("0.0.0.0:18889").unwrap()).await.unwrap();
    tokio::spawn(async move{
        let mut i = 0;
        while let Some(zip) = rx.recv().await {
            match zip {
                Zip::Data(mut package) => {
                    println!("bill = {:?} - data_size: {}", package.get_bill(), package.get_data().len());
                    i += 1;
                    package.set_data(Bytes::from(format!("abc - {}", i)));
                    println!("{:?}",&package);
                    let _ = tx.clone().send(Zip::build_data(package)).await.hand_err(|msg|error!("{msg}"));
                }
                Zip::Event(_) => {}
            }
        }
    });

    let mut i = 0;
    while let Some(zip) = rx1.recv().await {
        match zip {
            Zip::Data(mut package) => {
                println!("bill = {:?} - data_size: {}", package.get_bill(), package.get_data().len());
                i += 1;
                package.set_data(Bytes::from(format!("abc - {}", i)));
                println!("{:?}",&package);
                let _ = tx1.clone().send(Zip::build_data(package)).await.hand_err(|msg|error!("{msg}"));
            }
            Zip::Event(_) => {}
        }
    }
}