use std::net::SocketAddr;
use std::sync::Arc;
use bytes::Bytes;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc::{Sender, Receiver};
use constructor::{Get, New, Set};


//TCP连接有状态，需要持有每个连接的句柄
pub static TCP_HANDLE_MAP: Lazy<Arc<DashMap<Bill, Sender<Zip>>>> = Lazy::new(|| {
    Arc::new(DashMap::new())
});
pub const SOCKET_BUFFER_SIZE: usize = 4096;
pub const CHANNEL_BUFFER_SIZE: usize = 10000;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Protocol {
    UDP,
    TCP,
    ALL,
}

#[derive(Debug, Eq, Hash, PartialEq, New, Set, Get, Clone)]
pub struct Bill {
    to: SocketAddr,
    from: SocketAddr,
    protocol: Protocol,
}

// impl Bill {
//     pub fn new(to: SocketAddr, from: SocketAddr, protocol: Protocol) -> Self {
//         Self { to, from, protocol }
//     }
// }

///EVENT:
/// 0-TCP链接断开；input->对端断开连接；output->主动断开连接
#[derive(Debug)]
pub enum Zip {
    Data(Package),
    Event(Event),
}

impl Zip {
    pub fn build_data(package: Package) -> Self {
        Self::Data(package)
    }

    pub fn build_event(event: Event) -> Self {
        Self::Event(event)
    }

    pub fn get_bill(&self) -> Bill {
        match &self {
            Zip::Data(Package { bill, .. }) => { bill.clone() }
            Zip::Event(Event { bill, .. }) => { bill.clone() }
        }
    }

    pub fn get_bill_protocol(&self) -> &Protocol {
        match self {
            Zip::Data(Package { bill: Bill { protocol, .. }, .. }) => { protocol }
            Zip::Event(Event { bill: Bill { protocol, .. }, .. }) => { protocol }
        }
    }
}

#[derive(Debug, New, Set, Get)]
pub struct Event {
    bill: Bill,
    type_code: u8,
}

#[derive(Debug, New, Set, Get)]
pub struct Package {
    bill: Bill,
    data: Bytes,
}

// impl Package {
//     pub fn new(bill: Bill, data: Bytes) -> Self {
//         Self { bill, data }
//     }
// }

#[derive(Debug, New, Set, Get)]
pub struct Gate {
    //监听地址
    local_addr: SocketAddr,
    //从socket读取数据向程序发送
    intput: Sender<Zip>,
    //从程序中接收数据向socket写入
    output: Receiver<Zip>,
}

impl Gate {
    pub fn get_owned_output(self) -> Receiver<Zip> {
        self.output
    }
}

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