use std::{cell::{RefCell, RefMut}, ffi::c_void, fmt::Display, io::Error, ops::Deref, rc::Rc};

use windows::Win32::{
    Networking::WinSock::{
        RIO_CQ, RIO_EVENT_COMPLETION, RIO_IOCP_COMPLETION, RIO_NOTIFICATION_COMPLETION,
    },
    System::IO::OVERLAPPED,
};

use crate::net::win::{
    iocp::{IOCP, IOCPEntry},
    rio::riofuncs,
};

use super::{RIOEvent, RIO_INVALID_CQ};

/// The inner representation of a CompletionQueue.
///
/// This structure manages the underlying RIO completion queue handle, its capacity,
/// the number of currently allocated slots, and the type of completion mechanism being used.
struct InnerCompletionQueue {
    handle: RIO_CQ,
    capacity: usize,
    alloc: usize,
    completion: Completion,
}

impl InnerCompletionQueue {
    pub const MAX: usize = RIOCompletionQueue::MAX_SIZE;
    /// Attempts to allocate a given number of slots in the queue.
    ///
    /// # Arguments
    ///
    /// * `amount` - The number of additional slots to allocate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the allocation is successful, or an `Err` containing the number
    /// of available slots if the allocation would exceed the capacity.
    fn allocate(&mut self, amount: usize) -> Result<(), usize> {
        let x = self.alloc + amount;
        if x <= self.capacity {
            self.alloc = x;
            Ok(())
        } else {
            Err(self.capacity - self.alloc)
        }
    }

    /// Deallocates a given number of slots from the queue.
    ///
    /// # Arguments
    ///
    /// * `amount` - The number of slots to deallocate.
    fn deallocate(&mut self, amount: usize) {
        self.alloc = self.alloc.saturating_sub(amount)
    }

    /// Retrieves the underlying RIO completion queue handle.
    ///
    /// # Returns
    ///
    /// The RIO_CQ handle.
    fn handle(&self) -> RIO_CQ {
        self.handle
    }
    fn shrink_to_fit(&mut self) -> std::io::Result<()> {
        if self.alloc != self.capacity {
            unsafe {
                let resize = riofuncs::resize_completion_queue();
                if !resize(self.handle,self.alloc as u32).as_bool() {
                    return Err(Error::last_os_error())
                }
                self.capacity=self.alloc
            }
        }
        Ok(())
    }
    fn resize(&mut self, new_cap: usize)->std::io::Result<()> {
        if new_cap > Self::MAX {
            return Err(Error::from_raw_os_error(87));
        }
        if self.alloc > new_cap {
            return Err(Error::from_raw_os_error(122));
        }
            unsafe {
                let resize = riofuncs::resize_completion_queue();
                if !resize(self.handle,new_cap as u32).as_bool() {
                    return Err(Error::last_os_error())
                }
                self.capacity=new_cap
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

/// A CompletionQueue represents a registered I/O completion queue that can be used
/// for Registered I/O operations.
///
/// This type is clonable and uses shared ownership (via `Rc<RefCell<_>>`) for the inner queue.
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
    /// The default queue size used for new CompletionQueues.
    pub const DEFAULT_QUEUE_SIZE: usize = 1024;
    pub const MAX_SIZE: usize = 0x8000000;

    /// Creates a new CompletionQueue using the default queue size.
    ///
    /// This function creates a completion queue without a specific completion mechanism.
    /// For custom sizes or other completion types, consider using [`with_capacity`], [`new_iocp`], or [`new_event`].
    ///
    /// # Returns
    ///
    /// Returns a new `CompletionQueue` or an error if the underlying system call fails.
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

    /// Creates a new CompletionQueue with a custom capacity.
    ///
    /// This function creates a completion queue without a specific completion mechanism.
    /// For a specific mechanism use [`new_iocp`] or [`new_event`].
    ///
    /// # Arguments
    ///
    /// * `size` - The desired capacity for the CompletionQueue.
    ///
    /// # Returns
    ///
    /// Returns a new `CompletionQueue` or an error if the underlying system call fails.
    pub fn with_capacity(size: usize) -> std::io::Result<RIOCompletionQueue> {
        if size > Self::MAX_SIZE {
            return Err(Error::from_raw_os_error(87))
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

    /// Creates a new CompletionQueue that uses event-based notifications.
    ///
    /// This is a placeholder for future implementation. For now, it returns a "not done" error.
    ///
    /// # Arguments
    ///
    /// * `_size` - The desired capacity for the CompletionQueue.
    ///
    /// # Returns
    ///
    /// Returns a new `CompletionQueue` or an error if not implemented.
    pub fn new_event(_size: u32) -> std::io::Result<RIOCompletionQueue> {
        unsafe {
            let _create = riofuncs::create_completion_queue();
            let mut notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
            notify.Type = RIO_EVENT_COMPLETION;
            todo!("Not done")
        }
    }

    /// Creates a new CompletionQueue that uses IOCP (I/O Completion Ports) for notifications.
    ///
    /// # Arguments
    ///
    /// * `size` - The desired capacity for the CompletionQueue.
    /// * `iocp` - An `IOCP` instance to associate with the CompletionQueue.
    ///
    /// # Returns
    ///
    /// Returns a new `CompletionQueue` or an error if the underlying system call fails.
    pub fn new_iocp(size: usize, iocp: IOCP) -> std::io::Result<RIOCompletionQueue> {
        if size > Self::MAX_SIZE {
            return Err(Error::from_raw_os_error(87))
        }
        let entry = IOCPEntry::new(iocp);
        let mut overlapped = *entry.overlapped().inner_rc().deref();
        let mut notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
        notify.Type = RIO_IOCP_COMPLETION;
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

    /// Allocates a specified number of slots in the CompletionQueue.
    ///
    /// # Arguments
    ///
    /// * `slots` - The number of slots to allocate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the allocation is successful or an error with the number of
    /// free slots available if it fails.
    pub fn allocate(&mut self, slots: usize) -> Result<(), usize> {
        self.inner().allocate(slots)
    }

    /// Deallocates a specified number of slots from the CompletionQueue.
    ///
    /// # Arguments
    ///
    /// * `slots` - The number of slots to deallocate.
    pub fn deallocate(&mut self, slots: usize) {
        self.inner().deallocate(slots)
    }

    pub fn shrink_to_fit(&mut self) -> std::io::Result<()> {
        self.inner().shrink_to_fit()
    }

    pub fn resize(&mut self, new_cap: usize) -> std::io::Result<()> {
        self.inner().resize(new_cap)
    }

    /// Retrieves the underlying RIO completion queue handle.
    ///
    /// # Returns
    ///
    /// The RIO_CQ handle.
    pub fn handle(&self) -> RIO_CQ {
        self.inner().handle()
    }

    fn inner(&self)->RefMut<InnerCompletionQueue> {
        self.0.borrow_mut()
    }

    //Await

    pub fn basic_await_compl(&self)->std::io::Result<()> {
        let result = unsafe {
            let notify = riofuncs::notify();
            notify(self.handle())
        };
        if result != 0 {
            return Err(Error::from_raw_os_error(result));
        };
        Ok(())
    }
    pub fn iocp_await_compl(&self)->std::io::Result<()> {
        let inner = self.inner();
        let handle = inner.completion.iocp_handle();
        handle.await_compl()
    }
    pub fn event_await_compl(&self)->std::io::Result<()> {
        todo!()
    }
    pub fn await_compl(&self)->std::io::Result<()> {
        match self.inner().completion {
            Completion::IOCP(_)=>self.iocp_await_compl(),
            Completion::Event(_)=>self.event_await_compl(),
            Completion::None=>self.basic_await_compl()
        }
    }

    // Polling
    // Removes the Event from the Queue

    pub fn basic_poll_compl(&self) -> std::io::Result<Option<RIOEvent>> {
        let mut event = RIOEvent::new();
        let result = unsafe {
            let poll = riofuncs::dequeue();
            poll(self.handle(), event.as_result() as *mut _, 1)
        };
        if result == 0 {
            return Ok(None);
        } else if result == 1 {
            return Ok(Some(event));
        } else {
            return Err(Error::from_raw_os_error(6));
        }
    }
    pub fn iocp_poll_compl(&self)->std::io::Result<Option<RIOEvent>> {
        let inner = self.inner();
        let handle = inner.completion.iocp_handle();
        handle.poll_compl()?
    }
    pub fn await_and_poll_compl(&self) -> std::io::Result<RIOEvent> {
        self.await_compl()?;
        Ok(self.basic_poll_compl()?.unwrap())
    }
    pub fn is_invalid(&self) -> bool {
        self.handle() == RIO_INVALID_CQ
    }
}

/// Represents the completion mechanism used by a CompletionQueue.
///
/// It can be associated with an IOCP entry, an event-based mechanism, or none.
pub enum Completion {
    /// IOCP-based completion using the provided `IOCPEntry`.
    IOCP(IOCPEntry),
    /// Event-based completion (placeholder).
    Event((/*TODO*/)),
    /// No completion mechanism.
    None,
}

impl Completion {
    pub fn iocp_handle(&self)->IOCP {
        match self {
            Completion::IOCP(handle)=>handle.iocp(),
            _=>panic!()
        }
    }
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
