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

struct InnerRIOBuffer {
    id: RIO_BUFFERID,
    buf: Box<[u8]>,
    alloc: usize,
}

impl InnerRIOBuffer {
    /// Returns Ok if enough size is avalible
    fn allocate(&mut self, bytes: usize) -> Result<usize, usize> {
        let alloc = self.alloc;
        if alloc + bytes <= self.buf.len() {
            self.alloc += bytes;
            return Ok(alloc);
        }
        Err(self.buf.len() - self.alloc)
    }
    fn deallocate(&mut self, bytes: usize) {
        self.alloc = self.alloc.saturating_sub(bytes);
    }
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

pub struct RIOBuffer(RefCell<InnerRIOBuffer>);

impl RIOBuffer {
    pub const DEFAULT_SIZE: usize = 4 * 1024;
    pub const MAX_SIZE: usize = u32::MAX as usize;
    pub fn new() -> std::io::Result<RIOBuffer> {
        let buf: Box<[u8]> = vec![0_u8; Self::DEFAULT_SIZE].into_boxed_slice();
        buf.try_into()
    }
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
    pub fn with_size(bytes: usize) -> std::io::Result<RIOBuffer> {
        let buf: Box<[u8]> = vec![0_u8; bytes].into_boxed_slice();
        buf.try_into()
    }
    pub fn len(&self) -> usize {
        self.0.borrow_mut().len()
    }
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
    pub fn get_whole(&mut self) -> Option<RIOBufferSlice> {
        if self.0.borrow().alloc == 0 {
            return Some(RIOBufferSlice::from_buf(self.0.borrow_mut()));
        }
        None
    }
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

pub struct RIOBufferSlice<'a> {
    buf: RefMut<'a, InnerRIOBuffer>,
    winbuf: RIO_BUF,
}

impl<'a> RIOBufferSlice<'a> {
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
