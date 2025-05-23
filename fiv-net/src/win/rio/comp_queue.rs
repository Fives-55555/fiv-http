use std::{
    cell::{RefCell, RefMut},
    ffi::c_void,
    fmt::Display,
    io::Error,
    mem::MaybeUninit,
    num::NonZeroU64,
    ops::Deref,
    rc::Rc,
};

use windows::Win32::{
    Networking::WinSock::{
        RIO_CORRUPT_CQ, RIO_CQ, RIO_EVENT_COMPLETION, RIO_IOCP_COMPLETION,
        RIO_NOTIFICATION_COMPLETION, RIORESULT,
    },
    System::IO::OVERLAPPED,
};

use crate::{
    win::{
        event::Event, iocp::{IOCPEntry, IOCP}, rio::{riofuncs, RIOEvent, RIO_INVALID_CQ}
    }, AsyncIO
};

use super::{IOAlias, SocketAlias};

struct InnerCompletionQueue {
    handle: RIO_CQ,
    capacity: usize,
    alloc: usize,
    completion: Completion,
}

impl InnerCompletionQueue {
    pub const MAX: usize = RIOCompletionQueue::MAX_SIZE;

    fn allocate(&mut self, amount: usize) -> Result<(), usize> {
        let x = self.alloc + amount;
        if x <= self.capacity {
            self.alloc = x;
            Ok(())
        } else {
            Err(self.capacity - self.alloc)
        }
    }
    fn deallocate(&mut self, amount: usize) {
        self.alloc = self.alloc.saturating_sub(amount)
    }
    fn handle(&self) -> RIO_CQ {
        self.handle
    }
    fn shrink_to_fit(&mut self) -> std::io::Result<()> {
        if self.alloc != self.capacity {
            unsafe {
                let resize = riofuncs::resize_completion_queue();
                if !resize(self.handle, self.alloc as u32).as_bool() {
                    return Err(Error::last_os_error());
                }
                self.capacity = self.alloc
            }
        }
        Ok(())
    }
    fn resize(&mut self, new_cap: usize) -> std::io::Result<()> {
        if new_cap > Self::MAX {
            return Err(Error::from_raw_os_error(87));
        }
        if self.alloc > new_cap {
            return Err(Error::from_raw_os_error(122));
        }
        unsafe {
            let resize = riofuncs::resize_completion_queue();
            if !resize(self.handle, new_cap as u32).as_bool() {
                return Err(Error::last_os_error());
            }
            self.capacity = new_cap
        }
        Ok(())
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
pub struct RIOCompletionQueue(Rc<RefCell<InnerCompletionQueue>>);

impl Display for RIOCompletionQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bor = self.inner();
        write!(
            f,
            "CompletionQueue: (Handle: {}, Capacity: {}, Allocated: {}, Free Space: {}, Completion: {})",
            bor.handle.0,
            bor.capacity,
            bor.alloc,
            bor.capacity - bor.alloc,
            bor.completion
        )
    }
}

impl RIOCompletionQueue {
    pub const DEFAULT_QUEUE_SIZE: usize = 1024;
    pub const MAX_SIZE: usize = 0x8000000;

    pub fn new() -> std::io::Result<RIOCompletionQueue> {
        let result: RIO_CQ = unsafe {
            let create = riofuncs::create_completion_queue();
            create(Self::DEFAULT_QUEUE_SIZE as u32, std::ptr::null())
        };
        match result {
            RIO_INVALID_CQ => Err(Error::last_os_error()),
            _ => Ok(RIOCompletionQueue(Rc::new(RefCell::new(
                InnerCompletionQueue {
                    handle: result,
                    capacity: Self::DEFAULT_QUEUE_SIZE,
                    alloc: 0,
                    completion: Completion::None,
                },
            )))),
        }
    }
    pub fn with_capacity(size: usize) -> std::io::Result<RIOCompletionQueue> {
        if size > Self::MAX_SIZE {
            return Err(Error::from_raw_os_error(87));
        }
        let notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
        unsafe {
            let create = riofuncs::create_completion_queue();
            let result: RIO_CQ = create(size as u32, &notify as *const RIO_NOTIFICATION_COMPLETION);
            match result.0 {
                (..=0) => Err(Error::last_os_error()),
                _ => Ok(RIOCompletionQueue(Rc::new(RefCell::new(
                    InnerCompletionQueue {
                        handle: result,
                        capacity: size,
                        alloc: 0,
                        completion: Completion::None,
                    },
                )))),
            }
        }
    }
    pub fn new_event(_size: u32) -> std::io::Result<RIOCompletionQueue> {
        unsafe {
            let _create = riofuncs::create_completion_queue();
            let mut notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
            notify.Type = RIO_EVENT_COMPLETION;
            todo!("Not done")
        }
    }
    pub fn new_iocp(size: usize, iocp: IOCP, id: u64) -> std::io::Result<RIOCompletionQueue> {
        if size > Self::MAX_SIZE {
            return Err(Error::from_raw_os_error(87));
        }
        let entry = IOCPEntry::new(iocp, NonZeroU64::new(id));
        let mut overlapped = *entry.overlapped().inner_rc().deref();
        let mut notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
        notify.Type = RIO_IOCP_COMPLETION;
        notify.Anonymous.Iocp.CompletionKey = id as *mut c_void;
        notify.Anonymous.Iocp.IocpHandle = entry.handle();
        unsafe {
            let create = riofuncs::create_completion_queue();
            notify.Anonymous.Iocp.Overlapped = &mut overlapped as *mut OVERLAPPED as *mut c_void;
            let result = create(size as u32, &notify);
            if result.0 == 0 {
                return Err(Error::last_os_error());
            }
            Ok(RIOCompletionQueue(Rc::new(RefCell::new(
                InnerCompletionQueue {
                    handle: result,
                    capacity: size,
                    alloc: 0,
                    completion: Completion::IOCP(entry),
                },
            ))))
        }
    }
    pub fn allocate(&mut self, slots: usize) -> Result<(), usize> {
        self.inner().allocate(slots)
    }
    pub fn deallocate(&mut self, slots: usize) {
        self.inner().deallocate(slots)
    }
    pub fn shrink_to_fit(&mut self) -> std::io::Result<()> {
        self.inner().shrink_to_fit()
    }
    pub fn resize(&mut self, new_cap: usize) -> std::io::Result<()> {
        self.inner().resize(new_cap)
    }
    pub fn handle(&self) -> RIO_CQ {
        self.inner().handle()
    }
    fn inner(&self) -> RefMut<InnerCompletionQueue> {
        self.0.borrow_mut()
    }
    pub fn is_invalid(&self) -> bool {
        self.handle() == RIO_INVALID_CQ
    }
}

impl AsyncIO for RIOCompletionQueue {
    type Output = RIOPoll;
    fn poll(&mut self) -> std::io::Result<Option<Self::Output>> {
        let mut event: MaybeUninit<RIOEvent> = MaybeUninit::uninit();
        let (result, event) = unsafe {
            let poll = riofuncs::dequeue();
            (
                poll(self.handle(), event.as_mut_ptr() as *mut RIORESULT, 1),
                event.assume_init(),
            )
        };
        if result == 0 {
            return Ok(None);
        } else if result == 1 {
            return Ok(Some(event.as_poll()));
        } else {
            return Err(Error::from_raw_os_error(6));
        }
    }
    fn mass_poll(&mut self, len: usize) -> std::io::Result<Vec<Self::Output>> {
        let mut vec: Vec<RIOEvent> = Vec::with_capacity(len);
        let result = unsafe {
            let poll = riofuncs::dequeue();
            poll(
                self.handle(),
                vec.as_mut_ptr() as *mut RIORESULT,
                len as u32,
            )
        };
        if result == RIO_CORRUPT_CQ {
            return Err(Error::last_os_error());
        }
        unsafe {
            vec.set_len(result as usize);
        }
        let mut v = Vec::with_capacity(result as usize);
        for i in 0..result as usize {
            let sock_cxt = vec[i].socket();
            let req_cxt = vec[i].io_action();
            let len = vec[i].transfered();
            v.push(RIOPoll::from_raw(sock_cxt, req_cxt, len));
        }
        Ok(v)
    }
    fn await_cmpl(&self) -> std::io::Result<()> {
        match self.inner().completion {
            Completion::None=>unimplemented!(),
            _=>()
        }
        unsafe {
            let notify = riofuncs::notify();
            let result = notify(self.handle());
            if result != 0 {
                return Err(Error::from_raw_os_error(result));
            }
        }
        match &self.inner().completion {
            Completion::IOCP(inner) => inner.iocp().await_cmpl()?,
            Completion::Event(_inner) => {
                todo!()
            }
            Completion::None => (),
        }
        Ok(())
    }
}

pub enum Completion {
    /// IOCP-based completion using the provided `IOCPEntry`.
    IOCP(IOCPEntry),
    /// Event-based completion (placeholder).
    Event(Event),
    /// No completion mechanism.
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

pub struct RIOPoll {
    sock_ctx: SocketAlias,
    req_ctx: IOAlias,
    len: usize,
}

impl RIOPoll {
    pub fn from_raw(sock_cxt: u64, req_cxt: u64, len: usize) -> RIOPoll {
        RIOPoll {
            sock_ctx: sock_cxt,
            req_ctx: req_cxt,
            len: len,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
}
