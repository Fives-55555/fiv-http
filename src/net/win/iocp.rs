use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_FAILED, WAIT_OBJECT_0},
    System::{
        IO::{
            CreateIoCompletionPort, GetQueuedCompletionStatus, GetQueuedCompletionStatusEx,
            OVERLAPPED, OVERLAPPED_ENTRY,
        },
        Threading::{INFINITE, WaitForSingleObject},
    },
};

use std::{
    io::Error,
    num::NonZeroU64,
    os::windows::prelude::{AsRawHandle, RawHandle},
};

#[cfg(not(feature = "multithreaded"))]
use std::rc::Rc;
#[cfg(feature = "multithreaded")]
use std::sync::Arc;

use crate::net::AsyncIO;

use super::overlapped::Overlapped;

#[derive(PartialEq)]
pub struct InnerIocp(HANDLE);

impl Drop for InnerIocp {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0).unwrap() }
    }
}

/// Threadammount can only be set at construction
#[derive(Clone, PartialEq)]
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
    pub fn add_entry(&self, overlapped: Option<Overlapped>, id: Option<u64>) -> IOCPEntry {
        let overlapped = overlapped.unwrap_or(Overlapped::new());
        IOCPEntry::from_raw_parts(
            self.clone(),
            overlapped,
            id.and_then(|id| NonZeroU64::new(id)),
        )
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
    fn from_handle(handle: HANDLE) -> IOCP {
        IOCP(Rc::new(InnerIocp(handle)))
    }
}

impl AsyncIO for IOCP {
    type Output = IOCPPoll;
    fn poll(&mut self) -> std::io::Result<Option<Self::Output>> {
        let num: u32 = 0;
        let mut overlapped = std::ptr::null_mut();
        let id: usize = 0;
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
                    len: num as usize,
                }));
            }
        }
    }
    fn poll_timeout(&mut self, to: std::time::Duration) -> std::io::Result<Option<Self::Output>> {
        let num: u32 = 0;
        let mut overlapped = std::ptr::null_mut();
        let id: usize = 0;
        let result = unsafe {
            GetQueuedCompletionStatus(
                self.handle(),
                num as *mut u32,
                id as *mut usize,
                &mut overlapped as *mut *mut OVERLAPPED,
                to.as_millis() as u32,
            )
        };
        match result {
            Err(err) => return Err(Error::from_raw_os_error(err.code().0)),
            Ok(_) => {
                return Ok(Some(IOCPPoll {
                    handle: self.clone(),
                    id: id,
                    len: num as usize,
                }));
            }
        }
    }
    fn mass_poll(&mut self, len: usize) -> std::io::Result<Vec<Self::Output>> {
        let num: u32 = 0;
        let mut vec: Vec<OVERLAPPED_ENTRY> = vec![OVERLAPPED_ENTRY::default(); len];
        let result = unsafe {
            GetQueuedCompletionStatusEx(self.handle(), &mut vec, num as *mut u32, 0, false)
        };
        match result {
            Err(err) => return Err(Error::from_raw_os_error(err.code().0)),
            Ok(_) => {
                let mut v_out = Vec::with_capacity(num as usize);
                for i in 0..num as usize {
                    let entry = vec[i];
                    let poll = IOCPPoll {
                        handle: self.clone(),
                        id: entry.lpCompletionKey,
                        len: entry.dwNumberOfBytesTransferred as usize,
                    };
                    v_out.push(poll);
                }
                return Ok(v_out);
            }
        }
    }
    fn mass_poll_timeout(
        &mut self,
        len: usize,
        to: std::time::Duration,
    ) -> std::io::Result<Vec<Self::Output>> {
        let num: u32 = 0;
        let mut vec: Vec<OVERLAPPED_ENTRY> = vec![OVERLAPPED_ENTRY::default(); len];
        let result = unsafe {
            GetQueuedCompletionStatusEx(
                self.handle(),
                &mut vec,
                num as *mut u32,
                to.as_millis() as u32,
                false,
            )
        };
        match result {
            Err(err) => return Err(Error::from_raw_os_error(err.code().0)),
            Ok(_) => {
                let mut v_out = Vec::with_capacity(num as usize);
                for i in 0..num as usize {
                    let entry = vec[i];
                    let poll = IOCPPoll {
                        handle: self.clone(),
                        id: entry.lpCompletionKey,
                        len: entry.dwNumberOfBytesTransferred as usize,
                    };
                    v_out.push(poll);
                }
                return Ok(v_out);
            }
        }
    }
    fn await_cmpl(&self) -> std::io::Result<()> {
        unsafe {
            match WaitForSingleObject(self.handle(), INFINITE) {
                WAIT_OBJECT_0 => return Ok(()),
                WAIT_FAILED => Err(Error::last_os_error()),
                _ => panic!(),
            }
        }
    }
    fn await_and_poll(&mut self) -> std::io::Result<Self::Output> {
        let num: u32 = 0;
        let mut overlapped = std::ptr::null_mut();
        let id: usize = 0;
        let result = unsafe {
            GetQueuedCompletionStatus(
                self.handle(),
                num as *mut u32,
                id as *mut usize,
                &mut overlapped as *mut *mut OVERLAPPED,
                INFINITE,
            )
        };
        match result {
            Err(err) => return Err(Error::from_raw_os_error(err.code().0)),
            Ok(_) => {
                return Ok(IOCPPoll {
                    handle: self.clone(),
                    id: id,
                    len: num as usize,
                });
            }
        }
    }
}

impl AsRawHandle for IOCP {
    fn as_raw_handle(&self) -> RawHandle {
        self.0.0.0
    }
}

#[derive(PartialEq)]
pub struct IOCPPoll {
    handle: IOCP,
    id: usize,
    len: usize,
}

/// A dequeued Entry on an IOCP
#[derive(Clone, PartialEq)]
pub struct IOCPEntry {
    iocp: IOCP,
    overlapped: Overlapped,
    id: Option<NonZeroU64>,
}

impl AsRawHandle for IOCPEntry {
    fn as_raw_handle(&self) -> RawHandle {
        self.iocp.0.0.0
    }
}

impl IOCPEntry {
    /// IF there should be no id please input 0
    pub fn new(iocp: IOCP, id: Option<NonZeroU64>) -> IOCPEntry {
        IOCPEntry {
            iocp: iocp,
            overlapped: Overlapped::new(),
            id: id,
        }
    }
    pub fn from_raw_parts(iocp: IOCP, overlapped: Overlapped, id: Option<NonZeroU64>) -> IOCPEntry {
        IOCPEntry {
            iocp: iocp,
            overlapped: overlapped,
            id: id,
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
