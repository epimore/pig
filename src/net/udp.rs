use tokio::sync::mpsc::{Sender, Receiver};
use log::{debug, error, warn};
use crate::err::{GlobalResult, TransError};
use crate::net::shard::{Package, Gate, GateListener, GateAccept, SOCKET_BUFFER_SIZE, Bill, Protocol};
use tokio::net::UdpSocket;
use std::net::SocketAddr;
use bytes::Bytes;
use tokio::io;

//监听，将socket句柄发送出去
pub async fn listen(gate: Gate, tx: Sender<GateListener>) -> GlobalResult<()> {
    let local_addr = gate.local_addr;
    let socket = UdpSocket::bind(local_addr).await.hand_err(|msg| error!("{msg}"))?;
    let gate_listener = GateListener::build_udp(gate, socket);
    tx.send(gate_listener).await.hand_err(|msg| error!("{msg}"))?;
    debug!("开始监听 UDP 地址： {}", local_addr);
    Ok(())
}

//将socket句柄包装发送出去
pub async fn accept(gate: Gate, udp_socket: UdpSocket, accept_tx: Sender<GateAccept>) -> GlobalResult<()> {
    let gate_accept = GateAccept::accept_udp(gate, udp_socket);
    accept_tx.send(gate_accept).await.hand_err(|msg| error!("{msg}"))?;
    Ok(())
}

pub async fn read(local_addr: SocketAddr, udp_socket: &UdpSocket, tx: Sender<Package>) {
    loop {
        let _ = udp_socket.readable().await;
        let mut buf = [0u8; SOCKET_BUFFER_SIZE];
        match udp_socket.try_recv_from(&mut buf) {
            Ok((len, remote_addr)) => {
                if len != 0 {
                    debug!("【UDP read success】 【Local_addr = {}】 【Remote_addr = {}】 【len = {}】",
                            local_addr.to_string(),
                            remote_addr.to_string(),
                            len
                            );
                    let bill = Bill::new(local_addr, remote_addr, Protocol::UDP);
                    let package = Package::new(bill, Bytes::copy_from_slice(&buf[..len]));
                    let _ = tx.send(package).await.hand_err(|msg| error!("{msg}"));
                }
            }

            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(err) => {
                warn!("【UDP read failure】 【Local_addr = {}】 【err = {:?}】",
                            local_addr.to_string(),
                            err,
                            );
                break;
            }
        }
    }
}

pub async fn write(udp_socket: &UdpSocket, mut rx: Receiver<Package>) {
    while let Some(package) = rx.recv().await {
        let _ = udp_socket.writable().await;
        let bytes = package.data;
        let local_addr = &package.bill.from.to_string();
        let remote_addr = &package.bill.to.to_string();
        match udp_socket.try_send_to(&*bytes, package.bill.to) {
            Ok(len) => {
                debug!("【UDP write success】 【Local_addr = {}】 【Remote_addr = {}】 【len = {}】",
                            local_addr,
                            remote_addr,
                            len
                            );
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(err) => {
                error!("【UDP write failure】 【Local_addr = {}】 【Remote_addr = {}】 【err = {:?}】",
                            local_addr,
                            remote_addr,
                            err
                            );
                break;
            }
        }
    }
}