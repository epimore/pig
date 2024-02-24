use std::net::SocketAddr;
use std::sync::Arc;
use bytes::Bytes;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc::{Sender, Receiver};


//TCP连接有状态，需要持有每个连接的句柄
pub static TCP_HANDLE_MAP: Lazy<Arc<DashMap<Bill, Sender<Package>>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});
pub const SOCKET_BUFFER_SIZE: usize = 4096;
pub const CHANNEL_BUFFER_SIZE: usize = 10000;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Protocol {
    UDP,
    TCP,
    ALL,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Bill {
    pub to: SocketAddr,
    pub from: SocketAddr,
    pub protocol: Protocol,
}

impl Bill {
    pub fn new(to: SocketAddr, from: SocketAddr, protocol: Protocol) -> Self {
        Self { to, from, protocol }
    }
}


#[derive(Debug)]
pub struct Package {
    pub bill: Bill,
    pub data: Bytes,
}

impl Package {
    pub fn new(bill: Bill, data: Bytes) -> Self {
        Self { bill, data }
    }
}

#[derive(Debug)]
pub struct Gate {
    //监听地址
    pub local_addr: SocketAddr,
    //从socket读取数据向程序发送
    pub intput: Sender<Package>,
    //从程序中接收数据向socket写入
    pub output: Receiver<Package>,
}

impl Gate {
    pub fn new(local_addr: SocketAddr, intput: Sender<Package>, output: Receiver<Package>) -> Self {
        Self { local_addr, intput, output }
    }
}

// #[derive(Debug)]
// pub struct Listenner {
//     local_addr: SocketAddr,
//     protocol: Protocol,
// }

#[derive(Debug)]
pub enum GateListener {
    Tcp(Gate, TcpListener),
    Udp(Gate, UdpSocket),
}

impl GateListener {
    pub fn build_tcp(gate: Gate, tcp_listener: TcpListener) -> Self {
        Self::Tcp(gate, tcp_listener)
    }
    pub fn build_udp(gate: Gate, udp_socket: UdpSocket) -> Self {
        Self::Udp(gate, udp_socket)
    }
}

#[derive(Debug)]
pub enum GateAccept {
    //SocketAddr:remote_addr
    Tcp(Gate, SocketAddr, TcpStream),
    Udp(Gate, UdpSocket),
}

impl GateAccept {
    pub fn accept_tcp(gate: Gate, remote_addr: SocketAddr, tcp_stream: TcpStream) -> Self {
        Self::Tcp(gate, remote_addr, tcp_stream)
    }
    pub fn accept_udp(gate: Gate, udp_socket: UdpSocket) -> Self {
        Self::Udp(gate, udp_socket)
    }
}