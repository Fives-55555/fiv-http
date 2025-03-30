use std::{
    cell::{RefCell, RefMut},
    io::Error,
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

use windows::{
    Win32::Networking::WinSock::{RIO_BUF, RIO_BUFFERID},
    core::PCSTR,
};

use super::riofuncs;

/// Internal representation of a registered RIO buffer.
/// 
/// This structure maintains the RIO buffer identifier, the underlying memory,
/// and tracks the number of bytes already allocated.
struct InnerRIOBuffer {
    id: RIO_BUFFERID,
    buf: Box<[u8]>,
    alloc: usize,
}

impl InnerRIOBuffer {
    /// Attempts to allocate a given number of bytes from the buffer.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The number of bytes to allocate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(offset)` where `offset` is the starting position of the allocated region
    /// if enough space is available, or an `Err(available)` with the number of free bytes left.
    fn allocate(&mut self, bytes: usize) -> Result<usize, usize> {
        let alloc = self.alloc;
        if alloc + bytes <= self.buf.len() {
            self.alloc += bytes;
            Ok(alloc)
        } else {
            Err(self.buf.len() - self.alloc)
        }
    }

    /// Deallocates a given number of bytes from the buffer.
    ///
    /// This simply decreases the allocation counter by the specified number of bytes,
    /// saturating at zero.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The number of bytes to deallocate.
    fn deallocate(&mut self, bytes: usize) {
        self.alloc = self.alloc.saturating_sub(bytes);
    }

    /// Returns the total length (capacity) of the underlying buffer.
    fn len(&self) -> usize {
        self.buf.len()
    }
}

impl Drop for InnerRIOBuffer {
    fn drop(&mut self) {
        unsafe {
            let dereg = riofuncs::deregister_buffer();
            dereg(self.id);
        }
    }
}

/// Represents a registered RIO buffer.
///
/// This type encapsulates an internal buffer that can be used for Registered I/O (RIO)
/// operations. The buffer is managed inside a `RefCell` to allow interior mutability.
pub struct RIOBuffer(RefCell<InnerRIOBuffer>);

impl RIOBuffer {
    /// The default size for a new RIO buffer (4 KB).
    pub const DEFAULT_SIZE: usize = 4 * 1024;
    /// The maximum allowable size for a RIO buffer.
    pub const MAX_SIZE: usize = u32::MAX as usize;

    /// Creates a new RIOBuffer with the default size.
    ///
    /// This function allocates a buffer of `DEFAULT_SIZE` bytes and registers it.
    ///
    /// # Returns
    ///
    /// Returns a new `RIOBuffer` or an error if the registration fails.
    pub fn new() -> std::io::Result<RIOBuffer> {
        let buf: Box<[u8]> = vec![0_u8; Self::DEFAULT_SIZE].into_boxed_slice();
        buf.try_into()
    }

    /// Creates a new RIOBuffer from an existing boxed slice.
    ///
    /// This function registers the provided buffer with the RIO system.
    ///
    /// # Arguments
    ///
    /// * `buf` - A boxed slice of bytes to be used as the buffer.
    ///
    /// # Returns
    ///
    /// Returns a new `RIOBuffer` or an error if registration fails.
    pub fn from_buf(buf: Box<[u8]>) -> std::io::Result<RIOBuffer> {
        let bufferid = unsafe {
            let register = riofuncs::register_buffer();
            register(PCSTR(buf.as_ptr()), buf.len() as u32)
        };
        if bufferid.0 == 0 {
            return Err(Error::last_os_error());
        }
        Ok(RIOBuffer(RefCell::new(InnerRIOBuffer {
            alloc: 0,
            id: bufferid,
            buf: buf,
        })))
    }

    /// Creates a new RIOBuffer with a specified size.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The size in bytes of the new buffer.
    ///
    /// # Returns
    ///
    /// Returns a new `RIOBuffer` or an error if the registration fails.
    pub fn with_size(bytes: usize) -> std::io::Result<RIOBuffer> {
        let buf: Box<[u8]> = vec![0_u8; bytes].into_boxed_slice();
        buf.try_into()
    }

    /// Returns the total capacity of the underlying buffer.
    pub fn len(&self) -> usize {
        self.0.borrow_mut().len()
    }

    /// Allocates a slice of the buffer with the specified number of bytes.
    ///
    /// This function updates the internal allocation counter and returns a `RIOBufferSlice`
    /// representing the allocated region.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The number of bytes to allocate for the slice.
    ///
    /// # Returns
    ///
    /// Returns `Ok(RIOBufferSlice)` if allocation is successful, or `Err(remaining)` where
    /// `remaining` is the number of free bytes left if there is not enough space.
    pub fn get(&mut self, bytes: usize) -> Result<RIOBufferSlice, usize> {
        let mut inner = self.0.borrow_mut();
        let offset = inner.allocate(bytes)?;
        let winbuf = RIO_BUF {
            BufferId: inner.id,
            Offset: offset as u32,
            Length: bytes as u32,
        };
        Ok(RIOBufferSlice {
            buf: inner,
            winbuf: winbuf,
        })
    }

    /// Returns a slice covering the entire buffer if no allocation has been made.
    ///
    /// # Returns
    ///
    /// Returns `Some(RIOBufferSlice)` covering the whole buffer if nothing has been allocated;
    /// otherwise, returns `None`.
    pub fn get_whole(&mut self) -> Option<RIOBufferSlice> {
        if self.0.borrow_mut().alloc == 0 {
            return Some(RIOBufferSlice::from_buf(self.0.borrow_mut()));
        }
        None
    }

    /// Returns a slice covering the remaining unallocated portion of the buffer.
    ///
    /// # Returns
    ///
    /// Returns `Some((RIOBufferSlice, usize))` where the slice represents the rest of the buffer
    /// and the `usize` is the size of the remaining space. Returns `None` if no space is left.
    pub fn get_rest(&mut self) -> Option<(RIOBufferSlice, usize)> {
        let mut inner = self.0.borrow_mut();
        let size = inner.len() - inner.alloc;
        if size != 0 {
            let offset = inner.allocate(size).unwrap();
            let winbuf = RIO_BUF {
                BufferId: inner.id,
                Offset: offset as u32,
                Length: size as u32,
            };
            return Some((
                RIOBufferSlice {
                    buf: inner,
                    winbuf: winbuf,
                },
                size,
            ));
        }
        None
    }
}

impl TryFrom<Box<[u8]>> for RIOBuffer {
    type Error = Error;
    fn try_from(value: Box<[u8]>) -> Result<Self, Self::Error> {
        RIOBuffer::from_buf(value)
    }
}

/// A slice of a RIOBuffer allocated for a specific I/O operation.
///
/// This type holds a mutable reference to the inner buffer along with a corresponding
/// `RIO_BUF` descriptor that defines the slice's offset and length.
pub struct RIOBufferSlice<'a> {
    buf: RefMut<'a, InnerRIOBuffer>,
    winbuf: RIO_BUF,
}

impl<'a> RIOBufferSlice<'a> {
    /// Constructs a `RIOBufferSlice` covering the entire buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable borrow of the internal buffer.
    ///
    /// # Returns
    ///
    /// Returns a `RIOBufferSlice` representing the whole buffer.
    fn from_buf(buf: RefMut<InnerRIOBuffer>) -> RIOBufferSlice {
        let winbuf = RIO_BUF {
            BufferId: buf.id,
            Offset: 0,
            Length: buf.buf.len() as u32,
        };
        RIOBufferSlice {
            buf: buf,
            winbuf: winbuf,
        }
    }

    /// Returns a reference to the underlying `RIO_BUF` descriptor.
    pub fn buf(&self) -> &RIO_BUF {
        &self.winbuf
    }

    /// Returns the length of the buffer slice.
    pub fn len(&self) -> usize {
        self.winbuf.Length as usize
    }
}

impl<'a> Drop for RIOBufferSlice<'a> {
    fn drop(&mut self) {
        let len = self.len();
        self.buf.deallocate(len);
    }
}

impl<'a, I: SliceIndex<[u8]>> Index<I> for RIOBufferSlice<'a> {
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &I::Output {
        let a = self.winbuf.Offset as usize;
        let b = a + self.winbuf.Length as usize;
        index.index(&self.buf.buf[a..b])
    }
}

impl<'a, I: SliceIndex<[u8]>> IndexMut<I> for RIOBufferSlice<'a> {
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        let a = self.winbuf.Offset as usize;
        let b = a + self.winbuf.Length as usize;
        index.index_mut(&mut self.buf.buf[a..b])
    }
}
