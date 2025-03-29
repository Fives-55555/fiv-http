use std::{cell::RefMut, io::Error, os::windows::io::AsRawSocket};

use windows::Win32::Networking::WinSock::{RIO_BUF, RIO_CORRUPT_CQ, RIO_RQ, SOCKET};

use super::{buffer::RIOBuffer, riofuncs, InnerCompletionQueue};

pub struct RequestQueue<'a> {
    id: RIO_RQ,
    sock: SOCKET,
    sendcq: RefMut<'a, InnerCompletionQueue>,
    sendsize: u32,
    recvcq: RefMut<'a, InnerCompletionQueue>,
    recvsize: u32,
}

impl<'a> Drop for RequestQueue<'a> {
    fn drop(&mut self) {
        unsafe {
            let close_cmp = riofuncs::close_completion_queue();
            close_cmp(self.recvcq.);
        }
    }
}

impl<'a> RequestQueue<'a> {
    /// The completion queues needs to be diffrent ones
    pub fn new<T: AsRawSocket>(
        sock: T,
        mut recv: RefMut<InnerCompletionQueue>,
        recvsize: u32,
        mut send: RefMut<InnerCompletionQueue>,
        sendsize: u32,
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
                recvsize,
                8,
                sendsize,
                8,
                recv.handle(),
                send.handle(),
                std::ptr::null(),
            )
        };

        if queue.0 as u32 == RIO_CORRUPT_CQ {
            return Err(Error::last_os_error())
        }

        Ok(RequestQueue {
            id: queue,
            sock: sock,
            sendcq: send,
            sendsize: sendsize,
            recvcq: recv,
            recvsize: recvsize,
        })
    }
    pub fn resize_send(&mut self, newsize: u32) -> std::io::Result<()> {
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
    pub fn resize_recv(&mut self, newsize: u32) -> std::io::Result<()> {
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
    pub fn resize(&mut self, sendsize: u32, recvsize: u32) -> std::io::Result<()> {
        self.resize_send(sendsize)?;
        self.resize_recv(recvsize)
    }
    pub fn add_read(&mut self, buf: &mut RIOBuffer) -> std::io::Result<()> {
        unsafe {
            let recv = riofuncs::receive();
            let result = recv(self.id, buf.buf() as *const RIO_BUF, 0, 0, std::ptr::null());
            if !result.as_bool() {
                return Err(Error::last_os_error());
            }
            Ok(())
        }
    }
    pub fn add_read_ex(&mut self)->std::io::Result<()> {
        unsafe {
            let _read_ex = riofuncs::send_ex();
            todo!();
        }
    }
    pub fn add_write(&mut self, _buf: &RIOBuffer) -> std::io::Result<()> {
        unsafe {
            let _recv = riofuncs::send();
            todo!();
        }
    }
    pub fn socket(&self) -> SOCKET {
        self.sock
    }
}
