pub mod funcs;

use std::fmt::Display;

mod socket;

mod buffer;
pub use buffer::{RIOBuffer, RIOBufferSlice};

mod comp_queue;
pub use comp_queue::RIOCompletionQueue;

mod request_queue;
use comp_queue::RIOPoll;
pub use request_queue::RequestQueue;

pub use funcs::init;

mod stream;
pub use stream::RegisteredTcpStream;

use windows::Win32::Networking::WinSock::{RIO_CQ, RIO_RQ, RIORESULT};

#[repr(transparent)]
#[derive(Clone)]
pub struct RIOEvent(RIORESULT);

impl RIOEvent {
    pub fn new() -> RIOEvent {
        RIOEvent(RIORESULT::default())
    }
    pub fn is_ok(&self) -> bool {
        self.status() == 0
    }
    pub fn is_err(&self) -> bool {
        self.status() != 0
    }
    pub fn is_some(&self) -> bool {
        self.0.BytesTransferred != 0
    }
    pub fn status(&self) -> i32 {
        self.0.Status
    }
    pub fn transfered(&self) -> usize {
        self.0.BytesTransferred as usize
    }
    pub fn socket(&self) -> SocketAlias {
        self.0.SocketContext
    }
    pub fn io_action(&self) -> IOAlias {
        self.0.RequestContext
    }
    pub fn as_result(&mut self) -> &mut RIORESULT {
        &mut self.0
    }
    pub fn as_poll(&self) -> RIOPoll {
        RIOPoll::from_raw(self.socket(), self.io_action(), self.transfered())
    }
}

impl Display for RIOEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Status: {}, Is Err: {}, SocketContext: {}, IOContext: {}, Bytes transferd: {}",
            self.status(),
            self.is_err(),
            self.socket(),
            self.io_action(),
            self.transfered()
        )
    }
}

pub struct RIOIoOP {
    ioalias: IOAlias,
    buffer: RIOBufferSlice,
    len: usize,
}

impl RIOIoOP {
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }
    pub fn buf(self) -> RIOBufferSlice {
        self.buffer
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn ioalias(&self) -> IOAlias {
        self.ioalias
    }
}

pub type SocketAlias = u64;

pub type IOAlias = u64;

#[test]
fn test() -> std::io::Result<()> {
    init();

    let mut buffer = RIOBuffer::new().unwrap();
    let slice = buffer.alloc_whole().unwrap();

    let reg = RegisteredTcpStream::connect("127.0.0.1:8080").unwrap();

    println!("What");

    println!("Waiting");

    drop(reg);
    drop(slice);

    Ok(())
}

pub const RIO_INVALID_RQ: RIO_RQ = RIO_RQ(0);
pub const RIO_INVALID_CQ: RIO_CQ = RIO_CQ(0);
