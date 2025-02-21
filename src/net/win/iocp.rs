use std::{future::Future, ptr::null_mut, task::Poll};
use windows::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE},
    System::IO::{CreateIoCompletionPort, GetQueuedCompletionStatus, OVERLAPPED},
};

/// Threadammount can only be set at construction
#[derive(Debug)]
pub struct IOCompletionPort {
    handle: HANDLE, // Multiple Types of io
}

impl IOCompletionPort {
    pub fn new(threads: u32) -> std::io::Result<IOCompletionPort> {
        unsafe {
            match CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, threads) {
                Ok(handle) => Ok(IOCompletionPort { handle: handle }),
                Err(err) => io_err!(err),
            }
        }
    }
}

impl IOCompletionPort {
    pub fn from_handle(
        threads: u32,
        iohandle: HANDLE,
        ioid: usize,
    ) -> std::io::Result<IOCompletionPort> {
        unsafe {
            match CreateIoCompletionPort(INVALID_HANDLE_VALUE, Some(iohandle), ioid, threads) {
                Ok(handle) => Ok(IOCompletionPort { handle: handle }),
                Err(err) => io_err!(err),
            }
        }
    }
    /// You need to provide a Id for the handle which gets associated to the IOCP
    pub fn associate(&mut self, iohandle: HANDLE, ioid: usize) -> std::io::Result<()> {
        unsafe {
            match CreateIoCompletionPort(iohandle, Some(self.handle), ioid, 0) {
                Ok(_) => Ok(()),
                Err(err) => io_err!(err),
            }
        }
    }
    pub fn change(self) -> IOCompletionPort {
        IOCompletionPort {
            handle: self.handle,
        }
    }
}

impl Future for IOCompletionPort {
    type Output = std::io::Result<AsyncIoOut>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe {
            let mut num: u32 = 0;
            let mut key: usize = 0;
            let mut x: *mut OVERLAPPED = null_mut();
            match GetQueuedCompletionStatus(self.handle, &mut num, &mut key, &mut x, 0) {
                Ok(_) => {
                    return Poll::Ready(Ok(AsyncIoOut {
                        ioid: key,
                        len: num,
                        overlapped: x,
                    }))
                }
                Err(err) => return Poll::Ready(Err(std::io::Error::from(err))),
            }
        }
    }
}

pub struct AsyncIoOut {
    ioid: usize,
    len: u32,
    overlapped: *mut OVERLAPPED,
}

impl AsyncIoOut {
    pub fn id(&self) -> usize {
        self.ioid
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn overlapped(&self) -> &mut OVERLAPPED {
        unsafe { self.overlapped.as_mut().unwrap() }
    }
}
