#[cfg(windows)]
pub mod win;

pub enum LLProtocol {
    UDP,
    TCP,
}

impl LLProtocol {
    pub fn to_type(&self) -> i32 {
        match self {
            LLProtocol::TCP => 1,
            LLProtocol::UDP => 2,
        }
    }
    pub fn to_proto(&self) -> i32 {
        match self {
            LLProtocol::TCP | LLProtocol::UDP => 0,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(i32)]
pub enum AddrsFamily {
    IPV4 = 2,
    IPV6 = 23,
}

use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Once;
use std::task::Poll;
use std::time::Duration;

pub fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| drop(std::net::UdpSocket::bind("127.0.0.1:0")));
}

pub trait Wait {
    type Output;
    fn wait(&self, timeout: OSTimeout) -> Poll<Self::Output>;
    fn wait_for(&self) -> Self::Output;
}

#[cfg(windows)]
pub type OSTimeout = u32;

pub trait AsyncIO {
    type Output;
    /// Every Poll does dequeue an event
    fn poll(&mut self) -> std::io::Result<Option<Self::Output>>;
    fn poll_timeout(&mut self, _to: Duration) -> std::io::Result<Option<Self::Output>> {
        unimplemented!()
    }
    fn mass_poll(&mut self, _len: usize) -> std::io::Result<Vec<Self::Output>> {
        unimplemented!()
    }
    fn mass_poll_timeout(
        &mut self,
        _len: usize,
        _to: Duration,
    ) -> std::io::Result<Vec<Self::Output>> {
        unimplemented!()
    }
    fn await_cmpl(&self) -> std::io::Result<()>;
    fn await_and_poll(&mut self) -> std::io::Result<Self::Output> {
        self.await_cmpl()?;
        Ok(self.poll()?.unwrap())
    }
}

pub fn for_each_addrs<A: ToSocketAddrs, T, F: Fn(SocketAddr) -> std::io::Result<T>>(
    addrs: A,
    func: F,
) -> std::io::Result<T> {
    let mut error = None;
    for addr in addrs.to_socket_addrs()? {
        match func(addr) {
            Ok(ok) => return Ok(ok),
            Err(err) => error = Some(err),
        }
    }
    return Err(error.unwrap());
}
