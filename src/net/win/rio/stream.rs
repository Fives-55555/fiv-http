use std::{
    io::ErrorKind,
    net::{SocketAddr, ToSocketAddrs},
    os::windows::io::AsRawSocket,
};

use windows::Win32::Networking::WinSock::SOCK_STREAM;

use crate::net::win::iocp::IOCP;

use super::{socket::RIOSocket, RIOBufferSlice, RIOCompletionQueue, RIOEvent, RIOIoOP, RequestQueue};

pub struct RegisteredTcpStream<'a> {
    queue: RequestQueue,
    // Maybe abstract to use also the Event
    send: RIOCompletionQueue,
    send_iocp: IOCP,
    recv: RIOCompletionQueue,
    recv_iocp: IOCP,
    io_ops: [Option<RIOIoOP<'a>>; 2],
}

impl<'a> RegisteredTcpStream<'a> {
    pub const DEFAULT_THEAD_AMOUNT: u32 = 0;
    pub const DEFAULT_QUEUE_SIZE: usize = 1024;
    pub fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<RegisteredTcpStream<'a>> {
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
    fn single_connect<'b>(addr: &'b SocketAddr) -> std::io::Result<RegisteredTcpStream<'a>> {
        let sock = RIOSocket::new(addr, SOCK_STREAM.0)?;
        let send_iocp: IOCP = IOCP::new()?;
        let recv_iocp: IOCP = IOCP::new()?;
        let mut send: RIOCompletionQueue =
            RIOCompletionQueue::new_iocp(Self::DEFAULT_QUEUE_SIZE, send_iocp.clone())?;
        let mut recv: RIOCompletionQueue =
            RIOCompletionQueue::new_iocp(Self::DEFAULT_QUEUE_SIZE, recv_iocp.clone())?;
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
            recv: recv,
            recv_iocp: recv_iocp,
            io_ops: [const { None }; 2],
        };
        Ok(stream)
    }
    /// Important the stream can only be associated with one buffer
    pub fn read(&mut self, buf: RIOBufferSlice<'a>) -> std::io::Result<()> {
        if self.io_ops[0].is_some() {
            return Err(std::io::Error::new(
                ErrorKind::StorageFull,
                "Already Read in Queue",
            ));
        }
        self.io_ops[0] = Some(self.queue.add_read(buf, 0)?);
        Ok(())
    }
    /// Important the stream can only be associated with one buffer
    pub fn write(&mut self, buf: RIOBufferSlice<'a>) -> std::io::Result<()> {
        if self.io_ops[1].is_some() {
            return Err(std::io::Error::new(
                ErrorKind::StorageFull,
                "Already Read in Queue",
            ));
        }
        self.io_ops[1] = Some(self.queue.add_write(buf, 1)?);
        Ok(())
    }
    pub fn await_read(&self)->std::io::Result<()> {
        self.recv_iocp.await_compl()
    }
    pub fn get_read(&self)->std::io::Result<Option<()>> {
        self.recv_iocp.poll_compl()?;
        Ok(Some(()))
    }
    pub fn await_read_and_get(&mut self)->std::io::Result<(RIOEvent, RIOIoOP)> {
        self.recv.await_and_poll_compl().and_then(|res| Ok((res, self.io_ops[0].take().unwrap())))
    }
    pub fn await_write(&self)->std::io::Result<()> {
        self.send.await_compl()
    }
    pub fn get_write(&self)->std::io::Result<Option<RIOEvent>> {
        self.send.poll_compl();todo!()
    }
    pub fn await_write_and_get(&mut self)->std::io::Result<(RIOEvent, RIOIoOP)> {
        self.send.await_and_poll_compl().and_then(|res| Ok((res, self.io_ops[1].take().unwrap())))
    }
}

impl AsRawSocket for RegisteredTcpStream<'_> {
    fn as_raw_socket(&self) -> std::os::windows::prelude::RawSocket {
        self.queue.socket().0 as u64
    }
}
