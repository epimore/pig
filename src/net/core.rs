use std::net::SocketAddr;
use std::sync::{Arc};
use dashmap::DashMap;
use tokio::{io};
use crate::net::shard::{Package, Gate, GateListener, GateAccept, Protocol, CHANNEL_BUFFER_SIZE, Bill, TCP_HANDLE_MAP};
use crate::net::{tcp, udp};
use log::{error, warn};
use crate::err::{GlobalResult, TransError};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

//启动监听并返回读写句柄
pub async fn listen(protocol: Protocol, local_addr: SocketAddr, tx: Sender<GateListener>) -> GlobalResult<(Sender<Package>, Receiver<Package>)> {
    //socket 读数据通道 intput
    let (input_tx, input_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
    //socket 写数据通道 output
    let (output_tx, output_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
    match protocol {
        Protocol::TCP => {
            let gate = Gate::new(local_addr, input_tx, output_rx);
            tcp::listen(gate, tx.clone()).await?;
        }
        Protocol::UDP => {
            let gate = Gate::new(local_addr, input_tx, output_rx);
            udp::listen(gate, tx.clone()).await?;
        }
        Protocol::ALL => {
            let (tw_tx, tw_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
            let tgate = Gate::new(local_addr, input_tx.clone(), tw_rx);
            tcp::listen(tgate, tx.clone()).await?;
            let (uw_tx, uw_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
            let ugate = Gate::new(local_addr, input_tx.clone(), uw_rx);
            udp::listen(ugate, tx.clone()).await?;
            calssify(output_rx, tw_tx, uw_tx);
        }
    }
    Ok((output_tx, input_rx))
}

fn calssify(mut output: Receiver<Package>, tw_tx: Sender<Package>, uw_tx: Sender<Package>) {
    tokio::spawn(async move {
        while let Some(package) = output.recv().await {
            match package {
                Package {
                    bill: Bill { to: _, from: _, protocol: Protocol::UDP },
                    data: _
                } => {
                    let _ = uw_tx.clone().send(package).await.hand_err(|msg| error!("{msg}"));
                }
                Package {
                    bill: Bill { to: _, from: _, protocol: Protocol::TCP },
                    data: _
                } => {
                    let _ = tw_tx.clone().send(package).await.hand_err(|msg| error!("{msg}"));
                }
                _ => {
                    warn!("全协议发送？？？？")
                }
            }
        }
    });
}

//接收监听句柄，开启socket接入，将tcp/udp连接发送出去
pub async fn accept(mut rx: Receiver<GateListener>, tx: Sender<GateAccept>) -> GlobalResult<()> {
    while let Some(gate_listenner) = rx.recv().await {
        match gate_listenner {
            GateListener::Tcp(mut gate, listenner) => {
                let sender = tx.clone();
                tokio::spawn(async move {
                    let local_addr = gate.local_addr;
                    loop {
                        //给予每个对外发送数据tcp连接一个接收句柄，并将其对应的发送句柄保存起来
                        let (lone_output_tx, lone_output_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
                        let gate1 = Gate::new(local_addr, gate.intput.clone(), lone_output_rx);
                        let _ = tcp::accept(gate1, &listenner, sender.clone(), lone_output_tx).await.hand_err(|msg| error!("{msg}"));
                    }
                });
                tokio::spawn(async move {
                    //接收对外输出信息，并根据package上的账单信息，发送到对应的TCP发送通道
                    while let Some(package) = gate.output.recv().await {
                        match TCP_HANDLE_MAP.clone().get(&package.bill) {
                            None => {
                                warn!("【TCP】连接不存在 => {:?}",&package.bill);
                            }
                            Some(lone_output_tx) => {
                                let _ = lone_output_tx.send(package).await.hand_err(|msg| error!("{msg}"));
                            }
                        }
                    }
                });
            }
            GateListener::Udp(gate, socket) => {
                udp::accept(gate, socket, tx.clone()).await?;
            }
        }
    }
    Ok(())
}

pub async fn rw(mut rx: Receiver<GateAccept>) {
    while let Some(gate_accept) = rx.recv().await {
        match gate_accept {
            GateAccept::Tcp(gate, remote_addr, tcp_stream) => {
                let (read, write) = io::split(tcp_stream);
                let local_addr = gate.local_addr;
                let sender = gate.intput;
                tokio::spawn(async move {
                    let _ = tcp::read(read, local_addr, remote_addr, sender).await;
                });
                let receiver = gate.output;
                tokio::spawn(async move {
                    let _ = tcp::write(write, receiver).await;
                });
            }
            GateAccept::Udp(gate, udp_socket) => {
                let local_addr = gate.local_addr;
                let receiver = gate.output;
                let sender = gate.intput;
                let aus = Arc::new(udp_socket);
                let ausc = aus.clone();
                tokio::spawn(async move {
                    let _ = udp::read(local_addr, &*aus, sender).await;
                });
                tokio::spawn(async move {
                    let _ = udp::write(&*ausc, receiver).await;
                });
            }
        }
    }
}