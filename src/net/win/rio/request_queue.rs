use std::{io::Error, os::windows::io::AsRawSocket};

use windows::Win32::Networking::WinSock::{RIO_BUF, RIO_CORRUPT_CQ, RIO_RQ, SOCKET};

use super::{riofuncs, CompletionQueue, RIOBufferSlice};

pub struct RequestQueue {
    id: RIO_RQ,
    sock: SOCKET,
    sendcq: CompletionQueue,
    sendsize: usize,
    recvcq: CompletionQueue,
    recvsize: usize,
}

impl RequestQueue {
    /// The completion queues needs to be diffrent ones
    /// Returns first the ReqeustQueue then SendQueue then RecvQueue
    pub fn new<T: AsRawSocket>(
        sock: T,
    ) -> std::io::Result<(RequestQueue, CompletionQueue, CompletionQueue)> {
        let send = CompletionQueue::new()?;

        let recv = CompletionQueue::new()?;

        let req = RequestQueue::from_raw(
            sock,
            &send,
            CompletionQueue::DEFAULT_QUEUE_SIZE,
            &recv,
            CompletionQueue::DEFAULT_QUEUE_SIZE,
        )?;

        return Ok((req, send, recv));
    }
    /// Returns first the ReqeustQueue then SendQueue then RecvQueue
    /// If send or recv is None the eqivalent return is Some and they should be preserved
    pub fn from_comp<T: AsRawSocket>(
        sock: T,
        send: Option<(&CompletionQueue, usize)>,
        recv: Option<(&CompletionQueue, usize)>,
    ) -> std::io::Result<(
        RequestQueue,
        (Option<CompletionQueue>, Option<CompletionQueue>),
    )> {
        let mut ret_send = None;
        let mut ret_recv = None;

        let (send, sendsize, recv, recvsize) = match (send, recv) {
            (None, None) => {
                let res = RequestQueue::new(sock)?;
                return Ok((res.0, (Some(res.1), Some(res.2))));
            }
            (Some(send), Some(recv)) => (send.0, send.1, recv.0, recv.1),
            (Some(send), None) =>{
                ret_recv = Some(CompletionQueue::new()?);
                (send.0, send.1, ret_recv.as_ref().unwrap(), CompletionQueue::DEFAULT_QUEUE_SIZE)
            },
            (None, Some(recv)) =>{
                ret_send = Some(CompletionQueue::new()?);
                (ret_send.as_ref().unwrap(), CompletionQueue::DEFAULT_QUEUE_SIZE, recv.0, recv.1)
            },
        };
        let req = RequestQueue::from_raw(sock, &send, sendsize, &recv, recvsize)?;
        Ok((req, (ret_send, ret_recv)))
    }
    pub fn from_raw<T: AsRawSocket>(
        sock: T,
        send: &CompletionQueue,
        sendsize: usize,
        recv: &CompletionQueue,
        recvsize: usize,
    ) -> std::io::Result<RequestQueue> {
        let sock = SOCKET(sock.as_raw_socket() as usize);

        if recv.allocate(recvsize).is_err() {
            // Maybe add possibilty to add logging over a macro
            return Err(Error::from_raw_os_error(10055));
        }
        if send.allocate(sendsize).is_err() {
            // Maybe add possibilty to add logging over a macro
            return Err(Error::from_raw_os_error(10055));
        }

        let queue = unsafe {
            let create = riofuncs::create_request_queue();

            create(
                sock,
                recvsize as u32,
                8,
                sendsize as u32,
                8,
                recv.handle(),
                send.handle(),
                std::ptr::null(),
            )
        };

        if queue.0 as u32 == RIO_CORRUPT_CQ {
            return Err(Error::last_os_error());
        }

        Ok(RequestQueue {
            id: queue,
            sock: sock,
            sendcq: send.clone(),
            sendsize: sendsize,
            recvcq: recv.clone(),
            recvsize: recvsize,
        })
    }
    pub fn resize_send(&mut self, newsize: usize) -> std::io::Result<()> {
        let size = self.sendsize;
        if size < newsize {
            let alloc = newsize - size;
            if self.sendcq.allocate(alloc).is_err() {
                return Err(Error::from_raw_os_error(10055));
            } else {
                Ok(())
            }
        } else if size > newsize {
            let alloc = size - newsize;
            self.sendcq.deallocate(alloc);
            Ok(())
        } else {
            Ok(())
        }
    }
    pub fn resize_recv(&mut self, newsize: usize) -> std::io::Result<()> {
        let size = self.recvsize;
        if size < newsize {
            let alloc = newsize - size;
            if self.recvcq.allocate(alloc).is_err() {
                return Err(Error::from_raw_os_error(10055));
            } else {
                Ok(())
            }
        } else if size > newsize {
            let alloc = size - newsize;
            self.recvcq.deallocate(alloc);
            Ok(())
        } else {
            Ok(())
        }
    }
    pub fn resize(&mut self, sendsize: usize, recvsize: usize) -> std::io::Result<()> {
        self.resize_send(sendsize)?;
        self.resize_recv(recvsize)
    }
    pub fn add_read(&mut self, buf: &mut RIOBufferSlice) -> std::io::Result<()> {
        unsafe {
            let recv = riofuncs::receive();
            let result = recv(self.id, buf.buf() as *const RIO_BUF, 0, 0, std::ptr::null());
            if !result.as_bool() {
                return Err(Error::last_os_error());
            }
            Ok(())
        }
    }
    pub fn add_read_ex(&mut self) -> std::io::Result<()> {
        unsafe {
            let _read_ex = riofuncs::send_ex();
            todo!();
        }
    }
    pub fn add_write(&mut self, _buf: &RIOBufferSlice) -> std::io::Result<()> {
        unsafe {
            let _recv = riofuncs::send();
            todo!();
        }
    }
    pub fn socket(&self) -> SOCKET {
        self.sock
    }
}
