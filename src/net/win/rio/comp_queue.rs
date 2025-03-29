use std::{cell::{Ref, RefCell, RefMut}, ffi::c_void, fmt::Display, io::Error};

use windows::Win32::{
    Networking::WinSock::{
        RIO_CQ, RIO_EVENT_COMPLETION, RIO_IOCP_COMPLETION, RIO_NOTIFICATION_COMPLETION,
    },
    System::IO::OVERLAPPED,
};

use crate::net::win::{
    iocp::{IOCPEntry, IOCP},
    rio::riofuncs,
};

pub(crate) struct InnerCompletionQueue {
    handle: RIO_CQ,
    capacity: u32,
    alloc: u32,
    completion: Completion,
}

impl InnerCompletionQueue {
    pub(crate) fn allocate(&mut self, amount: u32) -> Result<(), u32> {
        let x = self.alloc + amount;
        if x <= self.capacity {
            self.alloc = x;
            Ok(())
        } else {
            Err(self.capacity - self.alloc)
        }
    }
    pub(crate) fn deallocate(&mut self, amount: u32) {
        self.alloc = self.alloc.saturating_sub(amount)
    }
}

impl Drop for InnerCompletionQueue {
    fn drop(&mut self) {
        unsafe {
            let close = riofuncs::close_completion_queue();
            close(self.handle)
        }
    }
}

pub struct CompletionQueue(RefCell<InnerCompletionQueue>);

impl Display for CompletionQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bor = self.0.borrow();
        write!(f, "CompletionQueue: (Handle: {}, Capacity: {}, Allocated: {}, Free Space: {}, Completion: {})", bor.handle.0, bor.capacity, bor.alloc, bor.capacity-bor.alloc, bor.completion)
    }
}

impl CompletionQueue {
    pub const COMPLETION_QUEUE_SIZE: u32 = 64;
    /// Uses the [`COMPLETION_QUEUE_SIZE`] as default queue size for custom size please use
    /// ['with_capacity'].
    /// ## Completion:
    /// It does not use any type of completion tech. If you need one use ['new_iocp'] or ['new_event'].
    pub fn new() -> CompletionQueue {
        let notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
        let result: RIO_CQ = unsafe {
            let create = riofuncs::create_completion_queue();
            create(
                Self::COMPLETION_QUEUE_SIZE,
                &notify as *const RIO_NOTIFICATION_COMPLETION,
            )
        };
        match result.0 {
            (..=0) => panic!("Windows Error: {}", Error::last_os_error()),
            _ => {
                return CompletionQueue(RefCell::new(InnerCompletionQueue {
                    handle: result,
                    capacity: Self::COMPLETION_QUEUE_SIZE,
                    alloc: 0,
                    completion: Completion::None,
                }))
            }
        }
    }
    /// Also assumes no Completion type needed if required use ['new_iocp'] or ['new_event'].
    pub fn with_capacity(size: u32) -> std::io::Result<CompletionQueue> {
        unsafe {
            let create = riofuncs::create_completion_queue();
            let notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
            let result: RIO_CQ = create(size, &notify as *const RIO_NOTIFICATION_COMPLETION);
            match result.0 {
                (..=0) => return Err(Error::last_os_error()),
                _ => {
                    return Ok(CompletionQueue(RefCell::new(
                        InnerCompletionQueue {
                            handle: result,
                            capacity: size,
                            alloc: 0,
                            completion: Completion::None,
                        },
                    )))
                }
            }
        }
    }
    pub fn new_event(_size: u32) -> std::io::Result<CompletionQueue> {
        unsafe {
            let _create = riofuncs::create_completion_queue();
            let mut notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
            notify.Type = RIO_EVENT_COMPLETION;
            todo!("Not done")
        }
    }
    pub fn new_iocp(size: u32, iocp: IOCP) -> std::io::Result<CompletionQueue> {
        unsafe {
            let create = riofuncs::create_completion_queue();
            let entry = IOCPEntry::new(iocp);
            let mut overlapped = *entry.overlapped();
            let mut notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
            notify.Type = RIO_IOCP_COMPLETION;
            notify.Anonymous.Iocp.IocpHandle = entry.handle();
            notify.Anonymous.Iocp.Overlapped = &mut overlapped as *mut OVERLAPPED as *mut c_void;
            let result = create(size, &notify);
            if result.0 == 0 {
                return Err(Error::last_os_error());
            }
            return Ok(CompletionQueue(RefCell::new(
                InnerCompletionQueue {
                    handle: result,
                    capacity: size,
                    alloc: 0,
                    completion: Completion::IOCP(entry),
                },
            )));
        }
    }
    pub fn allocate(&mut self, bytes: usize)-> Result<(), u32>{
        self.0.borrow_mut().allocate(bytes as u32)
    }
    pub fn deallocate(&mut self, bytes: usize) {
        self.0.borrow_mut().deallocate(bytes as u32)
    }
    pub fn handle(&self) -> RIO_CQ {
        self.0.borrow().handle
    }
    pub(crate) fn inner(&self)->Ref<InnerCompletionQueue> {
        self.0.borrow()
    }
    pub(crate) fn inner_mut(&mut self)->RefMut<InnerCompletionQueue> {
        self.0.borrow_mut()
    }
}

pub enum Completion {
    IOCP(IOCPEntry),
    Event(() /*Please Add*/),
    None,
}

impl Display for Completion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Completion::Event(_) => write!(f, "Event-Completion"),
            Completion::IOCP(entry) => write!(
                f,
                "IOCP-Completion: (Handle: {:#?}, Overlapped: !TODO)",
                entry.handle().0
            ),
            Completion::None => write!(f, "No-Completion"),
        }
    }
}
