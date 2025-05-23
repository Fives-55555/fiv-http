use std::{io::Error, os::raw::c_void, time::Duration};

use windows::Win32::Networking::WinSock::{RIO_BUF, RIO_RQ, SOCKET};

use super::{
    IOAlias, RIO_INVALID_RQ, RIOBufferSlice, RIOCompletionQueue, RIOIoOP, riofuncs,
    socket::{RIOSocket, ToWinSocket},
};

/// Represents a RequestQueue for Registered I/O operations.  
/// It maintains a socket handle along with separate completion queues for send and receive operations.
pub struct RequestQueue {
    id: RIO_RQ,
    sock: RIOSocket,
    sendcq: RIOCompletionQueue,
    sendsize: usize,
    recvcq: RIOCompletionQueue,
    recvsize: usize,
}

impl RequestQueue {
    pub fn new(
        sock: RIOSocket,
    ) -> std::io::Result<(RequestQueue, RIOCompletionQueue, RIOCompletionQueue)> {
        let mut send = RIOCompletionQueue::new()?;
        let mut recv = RIOCompletionQueue::new()?;
        let req = RequestQueue::from_raw(
            sock,
            &mut send,
            RIOCompletionQueue::DEFAULT_QUEUE_SIZE,
            &mut recv,
            RIOCompletionQueue::DEFAULT_QUEUE_SIZE,
        )?;
        Ok((req, send, recv))
    }
    pub fn from_comp(
        sock: RIOSocket,
        send: Option<(&mut RIOCompletionQueue, usize)>,
        recv: Option<(&mut RIOCompletionQueue, usize)>,
    ) -> std::io::Result<(
        RequestQueue,
        (Option<RIOCompletionQueue>, Option<RIOCompletionQueue>),
    )> {
        let mut ret_send = None;
        let mut ret_recv = None;

        let (mut send, sendsize, mut recv, recvsize) = match (send, recv) {
            (None, None) => {
                let res = RequestQueue::new(sock)?;
                return Ok((res.0, (Some(res.1), Some(res.2))));
            }
            (Some(send), Some(recv)) => (send.0, send.1, recv.0, recv.1),
            (Some(send), None) => {
                ret_recv = Some(RIOCompletionQueue::new()?);
                (
                    send.0,
                    send.1,
                    ret_recv.as_mut().unwrap(),
                    RIOCompletionQueue::DEFAULT_QUEUE_SIZE,
                )
            }
            (None, Some(recv)) => {
                ret_send = Some(RIOCompletionQueue::new()?);
                (
                    ret_send.as_mut().unwrap(),
                    RIOCompletionQueue::DEFAULT_QUEUE_SIZE,
                    recv.0,
                    recv.1,
                )
            }
        };
        let req = RequestQueue::from_raw(sock, &mut send, sendsize, &mut recv, recvsize)?;
        Ok((req, (ret_send, ret_recv)))
    }
    pub fn from_raw(
        sock: RIOSocket,
        send: &mut RIOCompletionQueue,
        sendsize: usize,
        recv: &mut RIOCompletionQueue,
        recvsize: usize,
    ) -> std::io::Result<RequestQueue> {
        let socka = sock.to_win_socket();

        if recv.is_invalid() {
            return Err(Error::from_raw_os_error(10022));
        }

        if send.is_invalid() {
            return Err(Error::from_raw_os_error(10022));
        }

        if recv.allocate(recvsize).is_err() {
            // Possibly add logging here via a macro in the future
            return Err(Error::from_raw_os_error(10055));
        }
        if send.allocate(sendsize).is_err() {
            // Possibly add logging here via a macro in the future
            return Err(Error::from_raw_os_error(10055));
        }

        std::thread::sleep(Duration::from_secs(1));

        let queue = unsafe {
            let recv_handle = recv.handle();
            let send_handle = send.handle();
            let create = riofuncs::create_request_queue();
            create(
                socka,
                recvsize as u32,
                // NEVER CHANGE ONLY ONE BUFFER CAN BE ASSOCIATED
                1,
                sendsize as u32,
                // NEVER CHANGE ONLY ONE BUFFER CAN BE ASSOCIATED
                1,
                recv_handle,
                send_handle,
                0 as *const c_void,
            )
        };

        if queue == RIO_INVALID_RQ {
            return Err(Error::last_os_error());
        }

        Ok(RequestQueue {
            id: queue,
            sock: sock,
            sendcq: send.clone(),
            sendsize,
            recvcq: recv.clone(),
            recvsize,
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
        todo!("add resize Requestqueue");
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
    pub fn add_read<'a>(
        &mut self,
        buf: RIOBufferSlice,
        alias: IOAlias,
    ) -> std::io::Result<RIOIoOP> {
        unsafe {
            let recv = riofuncs::receive();
            recv(
                self.id.clone(),
                buf.buf() as *const RIO_BUF,
                1,
                0,
                alias as *const c_void,
            )
            .ok()?;
        }
        Ok(RIOIoOP {
            ioalias: alias,
            buffer: buf,
            len: 0,
        })
    }
    pub fn add_read_ex(&mut self) -> std::io::Result<()> {
        unsafe {
            let _read_ex = riofuncs::send_ex();
            todo!();
        }
    }
    pub fn add_write(&mut self, buf: RIOBufferSlice, alias: IOAlias) -> std::io::Result<RIOIoOP> {
        unsafe {
            let send = riofuncs::send();
            send(
                self.id.clone(),
                buf.buf() as *const RIO_BUF,
                1,
                0,
                alias as *const c_void,
            )
            .ok()?;
        }
        Ok(RIOIoOP {
            ioalias: alias,
            buffer: buf,
            len: 0,
        })
    }

    pub fn add_write_ex(&mut self) -> std::io::Result<()> {
        unsafe {
            let _send_ex = riofuncs::send_ex();
            todo!();
        }
    }
    pub fn socket(&self) -> SOCKET {
        self.sock.to_win_socket()
    }
}
