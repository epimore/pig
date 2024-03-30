use common::net;
use std::net::SocketAddr;
use std::str::FromStr;
use bytes::Bytes;
use log::error;
use common::err::TransError;
use common::net::shared::Zip;

//cmd: cargo run --example many_udp --features net
#[tokio::main]
async fn main() {
    let _tripe = common::init();
    let (tx1, mut rx1) = net::init_net(net::shared::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18887").unwrap()).await.unwrap();
    let (tx2, mut rx2) = net::init_net(net::shared::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18888").unwrap()).await.unwrap();
    let (tx3, mut rx3) = net::init_net(net::shared::Protocol::UDP, SocketAddr::from_str("0.0.0.0:18889").unwrap()).await.unwrap();
    let mut i = 0;
    tokio::spawn(
        async move {
            while let Some(zip) = rx1.recv().await {
                match zip {
                    Zip::Data(mut package) => {
                        println!("bill = {:?} - data_size: {}", package.get_bill(), package.get_data().len());
                        i += 1;
                        package.set_data(Bytes::from(format!("abc - {}", i)));
                        println!("{:?}", &package);
                        let _ = tx1.clone().send(Zip::build_data(package)).await.hand_err(|msg| error!("{msg}"));
                    }
                    Zip::Event(_) => {}
                }
            }
        }
    );
    tokio::spawn(
        async move {
            while let Some(zip) = rx2.recv().await {
                match zip {
                    Zip::Data(mut package) => {
                        println!("bill = {:?} - data_size: {}", package.get_bill(), package.get_data().len());
                        i += 1;
                        package.set_data(Bytes::from(format!("abc - {}", i)));
                        println!("{:?}", &package);
                        let _ = tx2.clone().send(Zip::build_data(package)).await.hand_err(|msg| error!("{msg}"));
                    }
                    Zip::Event(_) => {}
                }
            }
        }
    );
    while let Some(zip) = rx3.recv().await {
        match zip {
            Zip::Data(mut package) => {
                println!("bill = {:?} - data_size: {}", package.get_bill(), package.get_data().len());
                i += 1;
                package.set_data(Bytes::from(format!("abc - {}", i)));
                println!("{:?}", &package);
                let _ = tx3.clone().send(Zip::build_data(package)).await.hand_err(|msg| error!("{msg}"));
            }
            Zip::Event(_) => {}
        }
    }
}