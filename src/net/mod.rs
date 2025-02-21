#[cfg(windows)]
pub mod win;

pub enum Protocol {
    UDP,
    TLS,
    TCP,
}

use std::sync::Once;
use std::task::Poll;

pub fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| drop(std::net::UdpSocket::bind("127.0.0.1:0")));
}

pub trait Subscribe {
    fn sub(&self) -> std::io::Result<()>;
}

pub trait Wait {
    type Output;
    fn wait(&self, timeout: OSTimeout) -> Poll<Self::Output>;
    fn wait_for(&self) -> Self::Output;
}

#[cfg(windows)]
pub type OSTimeout = u32;

#[cfg(windows)]
pub use win::overlapped::FutOverlappedTcpStream;
pub use win::overlapped::OverlappedTcpListener;
pub use win::overlapped::OverlappedTcpStream;
pub use win::FutAsyncRead;

pub trait AsyncRead {
    fn read(&self, buf: &mut [u8]) -> FutAsyncRead;
}

pub trait AsyncWrite {}


