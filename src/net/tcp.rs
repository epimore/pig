use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};
use tokio::{io, time};
use crate::net::shard::{Package, Gate, GateListener, GateAccept, SOCKET_BUFFER_SIZE, Bill, Protocol, TCP_HANDLE_MAP};
use log::{error, debug};
use crate::err::{GlobalResult, TransError};
use bytes::Bytes;
use std::io::Error;
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

//创建tcp监听，并将监听句柄（内含读写句柄）发送出去
//卸载监听 drop listen？
pub async fn listen(gate: Gate, tx: Sender<GateListener>) -> GlobalResult<()> {
    let local_addr = gate.local_addr;
    let tcp_listener = TcpListener::bind(local_addr).await.hand_err(|msg| error!("{msg}"))?;
    let gate_listener = GateListener::build_tcp(gate, tcp_listener);
    tx.send(gate_listener).await.hand_err(|msg| error!("{msg}"))?;
    debug!("开始监听 TCP 地址： {}", local_addr);
    Ok(())
}

//将连接句柄（内含读写句柄，远端地址等）发送出去
pub async fn accept(gate: Gate, tcp_listener: &TcpListener, accept_tx: Sender<GateAccept>, lone_output_tx: Sender<Package>) -> GlobalResult<()> {
    let local_addr = gate.local_addr;
    let gate_accept = check_accept(tcp_listener).await.map(|(tcp_stream, remote_addr)| {
        let bill = Bill::new(local_addr, remote_addr, Protocol::TCP);
        let map = TCP_HANDLE_MAP.clone();
        map.insert(bill, lone_output_tx);
        GateAccept::accept_tcp(gate, remote_addr, tcp_stream)
    })
        .hand_err(|msg| error!("{:?} : TCP accept has failed too many times.{msg}",local_addr))?;
    accept_tx.send(gate_accept).await.hand_err(|msg| error!("{msg}"))?;
    Ok(())
}

//连接检测
async fn check_accept(tcp_listener: &TcpListener) -> Result<(TcpStream, SocketAddr), Error> {
    let mut backoff = 1;
    loop {
        match tcp_listener.accept().await {
            Ok((tcp_stream, remote_addr)) => {
                return Ok((tcp_stream, remote_addr));
            }
            Err(err) => {
                if backoff > 32 {
                    return Err(err);
                }
            }
        }
        time::sleep(Duration::from_secs(backoff)).await;
        backoff *= 2;
    }
}


//连接断开测试
pub async fn read(mut reader: io::ReadHalf<TcpStream>, local_addr: SocketAddr, remote_addr: SocketAddr, tx: Sender<Package>) {
    loop {
        let mut buf = [0u8; SOCKET_BUFFER_SIZE];
        match reader.read(&mut buf[..]).await {
            Ok(len) => {
                if len != 0 {
                    debug!("【TCP read success】 【Local_addr = {}】 【Remote_addr = {}】 【len = {}】",
                            local_addr.to_string(),
                            remote_addr.to_string(),
                            len
                            );
                    let bill = Bill::new(local_addr, remote_addr, Protocol::TCP);
                    let package = Package::new(bill, Bytes::copy_from_slice(&buf[..len]));
                    let res = tx.send(package).await.hand_err(|msg| error!("{msg}"));
                } else {
                    debug!("【TCP connection disconnected】 【Local_addr = {}】 【Remote_addr = {}】",
                            local_addr.to_string(),
                            remote_addr.to_string()
                            );
                    let bill = Bill::new(local_addr, remote_addr, Protocol::TCP);
                    //断开连接移除持有句柄
                    let map = TCP_HANDLE_MAP.clone();
                    map.remove(&bill);
                    break;
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(err) => {
                error!("【TCP read failure】 【Local_addr = {}】 【err = {:?}】",
                            local_addr.to_string(),
                            err,
                            );
                break;
            }
        }
    }
}

pub async fn write(mut writer: io::WriteHalf<TcpStream>, mut rx: Receiver<Package>) {
    while let Some(package) = rx.recv().await {
        let bytes = package.data;
        let local_addr = &package.bill.from.to_string();
        let remote_addr = &package.bill.to.to_string();
        match writer.write(&*bytes).await {
            Ok(len) => {
                debug!("【TCP write success】 【Local_addr = {}】 【Remote_addr = {}】 【len = {}】",
                            local_addr,
                            remote_addr,
                            len
                            );
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(err) => {
                error!("【TCP write failure】 【Local_addr = {}】 【Remote_addr = {}】 【err = {:?}】",
                            local_addr,
                            remote_addr,
                            err
                            );
                break;
            }
        }
    }
}