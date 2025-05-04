#[cfg(windows)]
pub mod win;

pub enum Protocol {
    UDP,
    TLS,
    TCP,
}

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

pub use win::FutAsyncRead;
#[cfg(windows)]
pub use win::overlapped::FutOverlappedTcpStream;
pub use win::overlapped::OverlappedTcpListener;
pub use win::overlapped::OverlappedTcpStream;

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
