use windows::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE},
    System::IO::{CreateIoCompletionPort, OVERLAPPED},
};

use std::os::windows::prelude::{AsRawHandle, RawHandle};

#[cfg(not(feature = "multithreaded"))]
use std::rc::Rc;
#[cfg(feature = "multithreaded")]
use std::sync::Arc;

/// Threadammount can only be set at construction
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct IOCP(HANDLE);

impl IOCP {
    pub const THREADS: u32 = 0;
    pub fn new() -> std::io::Result<IOCP> {
        unsafe {
            match CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, Self::THREADS) {
                Ok(handle) => Ok(IOCP(handle)),
                Err(err) => io_err!(err),
            }
        }
    }
    pub fn add_entry(&self, overlapped: Option<OVERLAPPED>) -> IOCPEntry {
        let overlapped = overlapped.unwrap_or(OVERLAPPED::default());
        IOCPEntry {
            iocp: *self,
            #[cfg(feature = "multithreaded")]
            overlapped: Arc::new(overlapped),
            #[cfg(not(feature = "multithreaded"))]
            overlapped: Rc::new(overlapped),
        }
    }
    /// Creates a IOCompletionPort and registers a Handle in the IOCP which gets wraped in an Entry
    pub fn from_handle<T: AsRawHandle>(
        threads: u32,
        iohandle: T,
        ioid: usize,
    ) -> std::io::Result<IOCPEntry> {
        unsafe {
            match CreateIoCompletionPort(HANDLE(iohandle.as_raw_handle()), None, ioid, threads) {
                Ok(handle) => Ok(IOCPEntry {
                    iocp: IOCP(handle),
                    #[cfg(feature = "multithreaded")]
                    overlapped: Arc::new(OVERLAPPED::default()),
                    #[cfg(not(feature = "multithreaded"))]
                    overlapped: Rc::new(OVERLAPPED::default()),
                }),
                Err(err) => io_err!(err),
            }
        }
    }
    pub fn handle(&self) -> HANDLE {
        self.0
    }
    /// You need to provide a Id for the handle which gets associated to the IOCP
    pub fn associate<T: AsRawHandle>(&mut self, iohandle: T, ioid: usize) -> std::io::Result<()> {
        unsafe {
            match CreateIoCompletionPort(HANDLE(iohandle.as_raw_handle()), Some(self.0), ioid, 0) {
                Ok(_) => Ok(()),
                Err(err) => io_err!(err),
            }
        }
    }
}

impl AsRawHandle for IOCP {
    fn as_raw_handle(&self) -> RawHandle {
        self.0 .0
    }
}

pub struct IOCPEntry {
    iocp: IOCP,
    #[cfg(feature = "multithreaded")]
    overlapped: Arc<OVERLAPPED>,
    #[cfg(not(feature = "multithreaded"))]
    overlapped: Rc<OVERLAPPED>,
}

impl Clone for IOCPEntry {
    fn clone(&self) -> Self {
        IOCPEntry {
            iocp: self.iocp,
            overlapped: self.overlapped.clone(),
        }
    }
}

impl AsRawHandle for IOCPEntry {
    fn as_raw_handle(&self) -> RawHandle {
        self.iocp.0 .0
    }
}

impl IOCPEntry {
    pub fn new(iocp: IOCP) -> IOCPEntry {
        IOCPEntry {
            iocp: iocp,
            overlapped: Rc::new(OVERLAPPED::default()),
        }
    }
    pub fn handle(&self) -> HANDLE {
        self.iocp.0
    }
    pub fn iocp(&self) -> IOCP {
        self.iocp
    }
    pub fn overlapped(&self) -> Rc<OVERLAPPED> {
        self.overlapped.clone()
    }
}
