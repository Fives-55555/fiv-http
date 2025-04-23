use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_FAILED, WAIT_OBJECT_0},
    System::{
        IO::{CreateIoCompletionPort, GetQueuedCompletionStatus, OVERLAPPED},
        Threading::{INFINITE, WaitForSingleObject},
    },
};

use std::{
    io::Error,
    os::windows::prelude::{AsRawHandle, RawHandle},
};

#[cfg(not(feature = "multithreaded"))]
use std::rc::Rc;
#[cfg(feature = "multithreaded")]
use std::sync::Arc;

use super::overlapped::Overlapped;

pub struct InnerIocp(HANDLE);

impl Drop for InnerIocp {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0).unwrap() }
    }
}

/// Threadammount can only be set at construction
#[derive(Clone)]
pub struct IOCP(Rc<InnerIocp>);

impl IOCP {
    pub const THREADS: u32 = 0;
    pub fn new() -> std::io::Result<IOCP> {
        unsafe {
            match CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, Self::THREADS) {
                Ok(handle) => Ok(IOCP(Rc::new(InnerIocp(handle)))),
                Err(err) => io_err!(err),
            }
        }
    }
    pub fn add_entry(&self, overlapped: Option<OVERLAPPED>) -> IOCPEntry {
        let overlapped = overlapped.unwrap_or(OVERLAPPED::default());
        IOCPEntry {
            iocp: self.clone(),
            overlapped: Overlapped::new(),
        }
    }
    /// Creates a IOCompletionPort and registers a Handle in the IOCP
    pub fn from_fd<T: AsRawHandle>(
        threads: u32,
        iohandle: T,
        ioid: usize,
    ) -> std::io::Result<IOCP> {
        unsafe {
            match CreateIoCompletionPort(HANDLE(iohandle.as_raw_handle()), None, ioid, threads) {
                Ok(handle) => Ok(IOCP::from_handle(handle)),
                Err(err) => io_err!(err),
            }
        }
    }
    pub fn handle(&self) -> HANDLE {
        self.0.0
    }
    /// You need to provide a Id for the handle which gets associated to the IOCP
    pub fn add_fd<T: AsRawHandle>(&mut self, iohandle: T, ioid: usize) -> std::io::Result<()> {
        unsafe {
            match CreateIoCompletionPort(HANDLE(iohandle.as_raw_handle()), Some(self.0.0), ioid, 0)
            {
                Ok(_) => Ok(()),
                Err(err) => io_err!(err),
            }
        }
    }
    pub fn await_compl(&self) -> std::io::Result<()> {
        unsafe {
            match WaitForSingleObject(self.handle(), INFINITE) {
                WAIT_OBJECT_0 => return Ok(()),
                WAIT_FAILED => Err(Error::last_os_error()),
                _ => panic!(),
            }
        }
    }
    pub fn poll_compl(&self) -> std::io::Result<Option<IOCPPoll>> {
        let num = 0;
        let mut overlapped = std::ptr::null_mut();
        let id = 0;
        let result = unsafe {
            GetQueuedCompletionStatus(
                self.handle(),
                num as *mut u32,
                id as *mut usize,
                &mut overlapped as *mut *mut OVERLAPPED,
                0,
            )
        };
        match result {
            Err(err) => return Err(Error::from_raw_os_error(err.code().0)),
            Ok(_) => {
                return Ok(Some(IOCPPoll {
                    handle: self.clone(),
                    id: id,
                }));
            }
        }
    }
    fn from_handle(handle: HANDLE) -> IOCP {
        IOCP(Rc::new(InnerIocp(handle)))
    }
}

impl AsRawHandle for IOCP {
    fn as_raw_handle(&self) -> RawHandle {
        self.0.0.0
    }
}

pub struct IOCPPoll {
    handle: IOCP,
    id: usize,
}

/// A dequeued Entry on an IOCP
#[derive(Clone)]
pub struct IOCPEntry {
    iocp: IOCP,
    overlapped: Overlapped,
}

impl AsRawHandle for IOCPEntry {
    fn as_raw_handle(&self) -> RawHandle {
        self.iocp.0.0.0
    }
}

impl IOCPEntry {
    pub fn new(iocp: IOCP) -> IOCPEntry {
        IOCPEntry {
            iocp: iocp,
            overlapped: Overlapped::new(),
        }
    }
    pub fn handle(&self) -> HANDLE {
        self.iocp.0.0
    }
    pub fn iocp(&self) -> IOCP {
        self.iocp.clone()
    }
    pub fn overlapped(&self) -> Overlapped {
        self.overlapped.clone()
    }
}
