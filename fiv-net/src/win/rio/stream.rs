use std::{
    io::ErrorKind,
    net::{SocketAddr, ToSocketAddrs},
    os::windows::io::AsRawSocket,
};

use crate::{
    win::{
        completion::IOCP,
        rio::{comp_queue::Completion, socket::RIOSocket, RIOBufferSlice, RIOCompletionQueue, RIOIoOP, RequestQueue}, socket::FivSocket,
    }, AsyncIO
};

pub struct RequestInner {
    queue: RIOCompletionQueue,
    iocp: Completion,
    op: Option<RIOIoOP>,
}

pub struct RegisteredTcpStream {
    queue: RequestQueue,
    // Maybe abstract to use also the Event
    send: RequestInner,
    recv: RequestInner,
}

impl RegisteredTcpStream {
    pub const DEFAULT_THEAD_AMOUNT: u32 = 0;
    pub const DEFAULT_QUEUE_SIZE: usize = 1024;
    pub fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<RegisteredTcpStream> {
        let addrs = match addr.to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(e) => return Err(e),
        };
        let mut last_err = None;
        for addr in addrs {
            match RegisteredTcpStream::single_connect(&addr) {
                Ok(l) => return Ok(l),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| {
            std::io::Error::new(
                ErrorKind::InvalidInput,
                "could not resolve to any addresses",
            )
        }))
    }
    fn single_connect<'b>(addr: SocketAddr) -> std::io::Result<RegisteredTcpStream> {
        let sock = FivSocket::new_rio()?;
        let send_iocp: IOCP = IOCP::new()?;
        let recv_iocp: IOCP = IOCP::new()?;
        let mut send: RIOCompletionQueue =
            RIOCompletionQueue::new_iocp(Self::DEFAULT_QUEUE_SIZE, send_iocp.clone(), 1)?;
        let mut recv: RIOCompletionQueue =
            RIOCompletionQueue::new_iocp(Self::DEFAULT_QUEUE_SIZE, recv_iocp.clone(), 2)?;
        let queue: RequestQueue = RequestQueue::from_raw(
            sock,
            &mut send,
            Self::DEFAULT_QUEUE_SIZE,
            &mut recv,
            Self::DEFAULT_QUEUE_SIZE,
        )?;
        let stream: RegisteredTcpStream = RegisteredTcpStream {
            queue: queue,
            send: send,
            send_iocp: send_iocp,
            send_op: None,
            recv: recv,
            recv_iocp: recv_iocp,
            recv_op: None,
        };
        Ok(stream)
    }
    /// Important the stream can only be associated with one buffer
    pub fn add_read(&mut self, buf: RIOBufferSlice) -> std::io::Result<()> {
        if self.recv_op.is_some() {
            return Err(std::io::Error::new(
                ErrorKind::StorageFull,
                "Already Read in Queue",
            ));
        }
        self.recv_op = Some(self.queue.add_read(buf, 1)?);
        Ok(())
    }
    /// Important the stream can only be associated with one buffer
    pub fn add_write(&mut self, buf: RIOBufferSlice) -> std::io::Result<()> {
        if self.send_op.is_some() {
            return Err(std::io::Error::new(
                ErrorKind::StorageFull,
                "Already Read in Queue",
            ));
        }
        self.send_op = Some(self.queue.add_write(buf, 2)?);
        Ok(())
    }

    // Read
    pub fn has_read(&self) -> bool {
        self.recv_op.is_some()
    }
    pub fn await_read(&self) -> std::io::Result<()> {
        self.recv.await_cmpl()
    }
    pub fn poll_read(&mut self) -> std::io::Result<Option<RIOIoOP>> {
        if !self.has_read() {
            return Ok(None);
        }
        let x = match self.recv.poll()? {
            Some(inner) => inner,
            None => return Ok(None),
        };
        let mut ret = self.get_read().unwrap();
        ret.len = x.len();
        Ok(Some(ret))
    }
    fn get_read(&mut self) -> Option<RIOIoOP> {
        self.recv_op.take()
    }
    /// Expects a queued read request
    pub fn await_read_and_get<'b>(&mut self) -> std::io::Result<RIOIoOP> {
        self.await_read()?;
        let poll = self.poll_read()?;
        Ok(poll.unwrap())
    }

    // Write
    pub fn has_write(&self) -> bool {
        self.send_op.is_some()
    }
    pub fn await_write(&self) -> std::io::Result<()> {
        self.send.await_cmpl()
    }
    pub fn poll_write(&mut self) -> std::io::Result<Option<RIOIoOP>> {
        if !self.has_write() {
            return Ok(None);
        }
        let x = match self.send.poll()? {
            Some(inner) => inner,
            None => return Ok(None),
        };
        let mut ret = self.get_write().unwrap();
        ret.len = x.len();
        Ok(Some(ret))
    }
    fn get_write(&mut self) -> Option<RIOIoOP> {
        self.send_op.take()
    }
    /// Expects a queued write request
    pub fn await_write_and_get(&mut self) -> std::io::Result<RIOIoOP> {
        self.await_write()?;
        Ok(self.poll_write()?.unwrap())
    }
}

impl AsRawSocket for RegisteredTcpStream {
    fn as_raw_socket(&self) -> std::os::windows::prelude::RawSocket {
        self.queue.socket().0 as u64
    }
}
