use std::{
    io::Error,
    os::raw::c_void, time::Duration,
};

use windows::Win32::Networking::WinSock::{RIO_BUF, RIO_RQ, SOCKET};

use super::{riofuncs, socket::{RIOSocket, ToWinSocket}, CompletionQueue, IOAlias, RIOBufferSlice, RIO_INVALID_RQ};

/// Represents a RequestQueue for Registered I/O operations.  
/// It maintains a socket handle along with separate completion queues for send and receive operations.
pub struct RequestQueue {
    id: RIO_RQ,
    sock: RIOSocket,
    sendcq: CompletionQueue,
    sendsize: usize,
    recvcq: CompletionQueue,
    recvsize: usize,
}

impl RequestQueue {
    /// Creates a new RequestQueue along with its associated send and receive CompletionQueues.
    ///
    /// The two completion queues must be distinct.
    ///
    /// # Arguments
    ///
    /// * `sock` - A socket that implements `AsRawSocket`.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The created `RequestQueue`
    /// - The send `CompletionQueue`
    /// - The receive `CompletionQueue`
    ///
    /// # Errors
    ///
    /// Returns an error if the completion queues cannot be created or if allocation fails.
    pub fn new(
        sock: RIOSocket,
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
        Ok((req, send, recv))
    }

    /// Creates a RequestQueue from optionally provided completion queues.
    ///
    /// If either `send` or `recv` is `None`, a new CompletionQueue is created for that side.
    ///
    /// # Arguments
    ///
    /// * `sock` - A socket that implements `AsRawSocket`.
    /// * `send` - An optional tuple of a reference to a send `CompletionQueue` and its size.
    /// * `recv` - An optional tuple of a reference to a receive `CompletionQueue` and its size.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The created `RequestQueue`
    /// - A tuple with optional send and receive `CompletionQueue`s that were created
    ///
    /// # Errors
    ///
    /// Returns an error if the creation of the RequestQueue fails.
    pub fn from_comp(
        sock: RIOSocket,
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
            (Some(send), None) => {
                ret_recv = Some(CompletionQueue::new()?);
                (
                    send.0,
                    send.1,
                    ret_recv.as_ref().unwrap(),
                    CompletionQueue::DEFAULT_QUEUE_SIZE,
                )
            }
            (None, Some(recv)) => {
                ret_send = Some(CompletionQueue::new()?);
                (
                    ret_send.as_ref().unwrap(),
                    CompletionQueue::DEFAULT_QUEUE_SIZE,
                    recv.0,
                    recv.1,
                )
            }
        };
        let req = RequestQueue::from_raw(sock, &send, sendsize, &recv, recvsize)?;
        Ok((req, (ret_send, ret_recv)))
    }

    /// Creates a RequestQueue from raw components.
    ///
    /// This function takes ownership of the socket and registers the send and receive CompletionQueues.
    ///
    /// # Arguments
    ///
    /// * `sock` - A socket that implements `AsRawSocket`.
    /// * `send` - A reference to the send `CompletionQueue`.
    /// * `sendsize` - The size (number of buffers) allocated for sending.
    /// * `recv` - A reference to the receive `CompletionQueue`.
    /// * `recvsize` - The size (number of buffers) allocated for receiving.
    ///
    /// # Returns
    ///
    /// Returns the constructed `RequestQueue`.
    ///
    /// # Errors
    ///
    /// Returns an error if allocation fails or if the underlying RIO request queue is corrupted.
    pub fn from_raw(
        sock: RIOSocket,
        send: &CompletionQueue,
        sendsize: usize,
        recv: &CompletionQueue,
        recvsize: usize,
    ) -> std::io::Result<RequestQueue> {
        let socka = sock.to_win_socket();

        if recv.is_invalid() {
            return Err(Error::from_raw_os_error(10022))
        }

        if send.is_invalid() {
            return Err(Error::from_raw_os_error(10022))
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
            let create = riofuncs::create_request_queue();
            let recv_handle = recv.handle();
            let send_handle = send.handle();
            create(
                socka,
                recvsize as u32,
                8,
                sendsize as u32,
                8,
                recv_handle,
                send_handle,
                0 as *const c_void,
            )
        };

        println!("Hello");

        if queue.0 == RIO_INVALID_RQ {
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

    /// Resizes the send CompletionQueue.
    ///
    /// If the new size is greater than the current size, it allocates additional buffers.
    /// If the new size is less, it deallocates the extra buffers.
    ///
    /// # Arguments
    ///
    /// * `newsize` - The new desired size for the send CompletionQueue.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if allocation fails.
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

    /// Resizes the receive CompletionQueue.
    ///
    /// If the new size is greater than the current size, it allocates additional buffers.
    /// If the new size is less, it deallocates the extra buffers.
    ///
    /// # Arguments
    ///
    /// * `newsize` - The new desired size for the receive CompletionQueue.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if allocation fails.
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

    /// Resizes both the send and receive CompletionQueues.
    ///
    /// # Arguments
    ///
    /// * `sendsize` - The new desired size for the send CompletionQueue.
    /// * `recvsize` - The new desired size for the receive CompletionQueue.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if either resize operation fails.
    pub fn resize(&mut self, sendsize: usize, recvsize: usize) -> std::io::Result<()> {
        self.resize_send(sendsize)?;
        self.resize_recv(recvsize)
    }

    /// Adds a read request to the RequestQueue.
    ///
    /// This function submits a read operation using the provided buffer slice.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable reference to a `RIOBufferSlice` which holds the buffer for the operation.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the underlying RIO function fails.
    pub fn add_read(&mut self, buf: &mut RIOBufferSlice, alias: IOAlias) -> std::io::Result<()> {
        unsafe {
            let recv = riofuncs::receive();
            println!(
                "{:#?}, {:#?} {}, {}, {:#?}",
                self.id,
                buf.buf() as *const RIO_BUF,
                1,
                0,
                alias as *const c_void
            );
            Ok(recv(
                self.id,
                buf.buf() as *const RIO_BUF,
                1,
                0,
                alias as *const c_void,
            )
            .ok()?)
        }
    }

    /// Adds an extended read request to the RequestQueue.
    ///
    /// This function is currently a placeholder for an extended read operation.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the underlying RIO function fails.
    pub fn add_read_ex(&mut self) -> std::io::Result<()> {
        unsafe {
            let _read_ex = riofuncs::send_ex();
            todo!();
        }
    }

    /// Adds a write request to the RequestQueue.
    ///
    /// This function is currently a placeholder for a write operation.
    ///
    /// # Arguments
    ///
    /// * `_buf` - A reference to a `RIOBufferSlice` which holds the buffer for the operation.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the underlying RIO function fails.
    pub fn add_write(&mut self, _buf: &RIOBufferSlice) -> std::io::Result<()> {
        unsafe {
            let _recv = riofuncs::send();
            todo!();
        }
    }

    /// Returns the underlying SOCKET associated with this RequestQueue.
    pub fn socket(&self) -> SOCKET {
        self.sock.to_win_socket()
    }
}
