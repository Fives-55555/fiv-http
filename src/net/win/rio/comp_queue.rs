use std::{cell::RefCell, ffi::c_void, fmt::Display, io::Error, rc::Rc};

use windows::Win32::{
    Networking::WinSock::{
        RIO_CQ, RIO_EVENT_COMPLETION, RIO_IOCP_COMPLETION, RIO_NOTIFICATION_COMPLETION
    }, System::IO::OVERLAPPED
};

use crate::net::win::{
    iocp::{IOCPEntry, IOCP},
    rio::riofuncs,
};

use super::{RIOEvent, RIO_INVALID_CQ};

/// The inner representation of a CompletionQueue.
/// 
/// This structure manages the underlying RIO completion queue handle, its capacity,
/// the number of currently allocated slots, and the type of completion mechanism being used.
pub(crate) struct InnerCompletionQueue {
    handle: RIO_CQ,
    capacity: usize,
    alloc: usize,
    completion: Completion,
}

impl InnerCompletionQueue {
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
    pub(crate) fn allocate(&mut self, amount: usize) -> Result<(), usize> {
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
    pub(crate) fn deallocate(&mut self, amount: usize) {
        self.alloc = self.alloc.saturating_sub(amount)
    }

    /// Retrieves the underlying RIO completion queue handle.
    ///
    /// # Returns
    ///
    /// The RIO_CQ handle.
    pub(crate) fn handle(&self) -> RIO_CQ {
        self.handle
    }
}

impl Drop for InnerCompletionQueue {
    fn drop(&mut self) {
        unsafe {
            let close = riofuncs::close_completion_queue();
            //close(self.handle)
        }
    }
}

/// A CompletionQueue represents a registered I/O completion queue that can be used
/// for Registered I/O operations.
///
/// This type is clonable and uses shared ownership (via `Rc<RefCell<_>>`) for the inner queue.
#[derive(Clone)]
pub struct CompletionQueue(Rc<RefCell<InnerCompletionQueue>>);

impl Display for CompletionQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bor = self.0.borrow();
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

impl CompletionQueue {
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
    pub fn new() -> std::io::Result<CompletionQueue> {
        let result: RIO_CQ = unsafe {
            let create = riofuncs::create_completion_queue();
            create(
                Self::DEFAULT_QUEUE_SIZE as u32,
                std::ptr::null()
            )
        };
        match result.0 {
            RIO_INVALID_CQ => Err(Error::last_os_error()),
            _ => Ok(CompletionQueue(Rc::new(RefCell::new(InnerCompletionQueue {
                handle: result,
                capacity: Self::DEFAULT_QUEUE_SIZE,
                alloc: 0,
                completion: Completion::None,
            })))),
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
    pub fn with_capacity(size: usize) -> std::io::Result<CompletionQueue> {
        unsafe {
            let create = riofuncs::create_completion_queue();
            let notify: RIO_NOTIFICATION_COMPLETION = RIO_NOTIFICATION_COMPLETION::default();
            let result: RIO_CQ = create(size as u32, &notify as *const RIO_NOTIFICATION_COMPLETION);
            match result.0 {
                (..=0) => Err(Error::last_os_error()),
                _ => Ok(CompletionQueue(Rc::new(RefCell::new(InnerCompletionQueue {
                    handle: result,
                    capacity: size,
                    alloc: 0,
                    completion: Completion::None,
                })))),
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
    pub fn new_event(_size: u32) -> std::io::Result<CompletionQueue> {
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
            Ok(CompletionQueue(Rc::new(RefCell::new(InnerCompletionQueue {
                handle: result,
                capacity: size,
                alloc: 0,
                completion: Completion::IOCP(entry),
            }))))
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
    pub fn allocate(&self, slots: usize) -> Result<(), usize> {
        self.0.borrow_mut().allocate(slots)
    }

    /// Deallocates a specified number of slots from the CompletionQueue.
    ///
    /// # Arguments
    ///
    /// * `slots` - The number of slots to deallocate.
    pub fn deallocate(&self, slots: usize) {
        self.0.borrow_mut().deallocate(slots)
    }

    /// Retrieves the underlying RIO completion queue handle.
    ///
    /// # Returns
    ///
    /// The RIO_CQ handle.
    pub fn handle(&self) -> RIO_CQ {
        self.0.borrow_mut().handle()
    }
    pub fn poll_compl(&self)->std::io::Result<Option<RIOEvent>> {
        let mut event = RIOEvent::new();
        let result = unsafe {
            let poll = riofuncs::dequeue();
            poll(self.handle(), event.as_result() as *mut _, 1)
        };
        if result == 0 {
            return Ok(None)
        } else if result == 1 {
            return Ok(Some(event));
        } else {
            return Err(Error::from_raw_os_error(6));
        }
    }
    pub fn await_and_compl(&self)->std::io::Result<RIOEvent> {
        let result = unsafe {
            let notify = riofuncs::notify();
            notify(self.handle())
        };
        if result != 0 {
            return Err(Error::from_raw_os_error(result))
        };
        Ok(self.poll_compl()?.unwrap())
    }
}

/// Represents the completion mechanism used by a CompletionQueue.
///
/// It can be associated with an IOCP entry, an event-based mechanism, or none.
pub enum Completion {
    /// IOCP-based completion using the provided `IOCPEntry`.
    IOCP(IOCPEntry),
    /// Event-based completion (placeholder).
    Event(() /*Please Add*/),
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
