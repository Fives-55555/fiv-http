use std::{cell::RefCell, ffi::c_void, fmt::Display, io::Error, rc::Rc};

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
    capacity: usize,
    alloc: usize,
    completion: Completion,
}

impl InnerCompletionQueue {
    pub(crate) fn allocate(&mut self, amount: usize) -> Result<(), usize> {
        let x = self.alloc + amount;
        if x <= self.capacity {
            self.alloc = x;
            Ok(())
        } else {
            Err(self.capacity - self.alloc)
        }
    }
    pub(crate) fn deallocate(&mut self, amount: usize) {
        self.alloc = self.alloc.saturating_sub(amount)
    }
    pub(crate) fn handle(&self)->RIO_CQ {
        self.handle
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

#[derive(Clone)]
pub struct CompletionQueue(Rc<RefCell<InnerCompletionQueue>>);

impl Display for CompletionQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bor = self.0.borrow();
        write!(f, "CompletionQueue: (Handle: {}, Capacity: {}, Allocated: {}, Free Space: {}, Completion: {})", bor.handle.0, bor.capacity, bor.alloc, bor.capacity-bor.alloc, bor.completion)
    }
}

impl CompletionQueue {
    pub const DEFAULT_QUEUE_SIZE: usize = 1024;
    /// Uses the [`COMPLETION_QUEUE_SIZE`] as default queue size for custom size please use
    /// ['with_capacity'].
    /// ## Completion:
    /// It does not use any type of completion tech. If you need one use ['new_iocp'] or ['new_event'].
    pub fn new() -> std::io::Result<CompletionQueue> {
        let notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
        let result: RIO_CQ = unsafe {
            let create = riofuncs::create_completion_queue();
            create(
                Self::DEFAULT_QUEUE_SIZE as u32,
                &notify as *const RIO_NOTIFICATION_COMPLETION,
            )
        };
        match result.0 {
            (..=0) => Err(Error::last_os_error()),
            _ => {
                return Ok(CompletionQueue(Rc::new(RefCell::new(InnerCompletionQueue {
                    handle: result,
                    capacity: Self::DEFAULT_QUEUE_SIZE,
                    alloc: 0,
                    completion: Completion::None,
                }))))
            }
        }
    }
    /// Also assumes no Completion type needed if required use ['new_iocp'] or ['new_event'].
    pub fn with_capacity(size: usize) -> std::io::Result<CompletionQueue> {
        unsafe {
            let create = riofuncs::create_completion_queue();
            let notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
            let result: RIO_CQ = create(size as u32, &notify as *const RIO_NOTIFICATION_COMPLETION);
            match result.0 {
                (..=0) => return Err(Error::last_os_error()),
                _ => {
                    return Ok(CompletionQueue(Rc::new(RefCell::new(
                        InnerCompletionQueue {
                            handle: result,
                            capacity: size,
                            alloc: 0,
                            completion: Completion::None,
                        },
                    ))))
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
    pub fn new_iocp(size: usize, iocp: IOCP) -> std::io::Result<CompletionQueue> {
        unsafe {
            let create = riofuncs::create_completion_queue();
            let entry = IOCPEntry::new(iocp);
            let mut overlapped = *entry.overlapped();
            let mut notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
            notify.Type = RIO_IOCP_COMPLETION;
            notify.Anonymous.Iocp.IocpHandle = entry.handle();
            notify.Anonymous.Iocp.Overlapped = &mut overlapped as *mut OVERLAPPED as *mut c_void;
            let result = create(size as u32, &notify);
            if result.0 == 0 {
                return Err(Error::last_os_error());
            }
            return Ok(CompletionQueue(Rc::new(RefCell::new(
                InnerCompletionQueue {
                    handle: result,
                    capacity: size,
                    alloc: 0,
                    completion: Completion::IOCP(entry),
                },
            ))));
        }
    }
    pub fn allocate(&self, slots: usize)-> Result<(), usize>{
        self.0.borrow_mut().allocate(slots)
    }
    pub fn deallocate(&self, slots: usize) {
        self.0.borrow_mut().deallocate(slots)
    }
    pub fn handle(&self) -> RIO_CQ {
        self.0.borrow_mut().handle()
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
