use std::{
    cell::{Ref, RefCell, RefMut},
    io::Error,
    ops::{Index, IndexMut},
    ptr::NonNull,
    rc::Rc,
    slice::SliceIndex,
};

use windows::{
    Win32::Networking::WinSock::{RIO_BUF, RIO_BUFFERID},
    core::PCSTR,
};

use super::riofuncs;

pub struct RIOBuffer {
    id: RIO_BUFFERID,
    buf: Box<[u8]>,
    root: BufferNode,
}

impl RIOBuffer {
    pub const DEFAULT_SIZE: usize = 4 * 1024;
    pub const MAX_SIZE: usize = u32::MAX as usize;
    pub fn new() -> Result<RIOBuffer, Error> {
        let buf: Box<[u8]> = vec![0; Self::DEFAULT_SIZE].into_boxed_slice();
        buf.try_into()
    }
    pub fn with_capacity(len: usize) -> Result<RIOBuffer, Error> {
        let buf: Box<[u8]> = vec![0; len].into_boxed_slice();
        buf.try_into()
    }
    pub fn from_buf(mut buf: Box<[u8]>) -> std::io::Result<RIOBuffer> {
        let bufferid = unsafe {
            let register = riofuncs::register_buffer();
            register(PCSTR(buf.as_ptr()), buf.len() as u32)
        };
        if bufferid.0 == 0 {
            return Err(Error::last_os_error());
        }
        let root = BufferNode::root(NonNull::new(&mut buf[..] as *mut [u8]).unwrap());
        Ok(RIOBuffer {
            id: bufferid,
            buf: buf,
            root: root,
        })
    }
    pub fn alloc(&mut self, size: usize) -> Result<RIOBufferSlice, usize> {
        let mut vec: Vec<(usize, BufferNode)> = Vec::new();
        self.root.push_empty_size(&mut vec);
        if vec.len() == 0 {
            return Err(0);
        }
        vec.sort_by(|a, b| a.0.cmp(&b.0));
        for elem in vec.iter_mut() {
            if elem.0 < size {
                continue;
            } else if elem.0 > size {
                let mut children = match elem.1.split(size) {
                    Some(children) => children,
                    None => continue,
                };
                match children.0.to_used() {
                    Some(_) => return Ok(RIOBufferSlice::from_node(self.id, children.0)),
                    None => continue,
                }
            } else {
                elem.1.to_used().unwrap();
                return Ok(RIOBufferSlice::from_node(self.id, elem.1.clone()));
            }
        }
        return Err(vec[vec.len() - 1].0);
    }
    pub fn alloc_whole(&mut self) -> Option<RIOBufferSlice> {
        let node = self.root.clone();
        let mut inner = self.root.inner_mut();
        match inner.useage {
            Useage::NotUsed => {
                inner.to_used().unwrap();
                drop(inner);
                Some(RIOBufferSlice::from_node(self.id, node))
            }
            _ => None,
        }
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }
}

impl TryFrom<Box<[u8]>> for RIOBuffer {
    type Error = Error;
    fn try_from(value: Box<[u8]>) -> Result<Self, Self::Error> {
        RIOBuffer::from_buf(value)
    }
}

impl Drop for RIOBuffer {
    fn drop(&mut self) {
        unsafe {
            let dereg = riofuncs::deregister_buffer();
            dereg(self.id);
        }
    }
}

#[derive(Clone)]
enum Useage {
    NotUsed,
    Used,
    Split((BufferNode, BufferNode)),
}

struct InnerBufferNode {
    useage: Useage,
    offset: usize,
    slice: NonNull<[u8]>,
}

impl InnerBufferNode {
    fn to_used(&mut self) -> Option<()> {
        match self.useage {
            Useage::NotUsed => {
                self.useage = Useage::Used;
                Some(())
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
struct BufferNode(Rc<RefCell<InnerBufferNode>>);

impl BufferNode {
    fn push_empty_size(&self, v: &mut Vec<(usize, BufferNode)>) {
        let inner = self.inner();
        match &inner.useage {
            Useage::NotUsed => v.push((unsafe { inner.slice.as_ref().len() }, self.clone())),
            Useage::Split((left, right)) => {
                left.push_empty_size(v);
                right.push_empty_size(v);
            }
            _ => (),
        }
    }
    fn root(ptr: NonNull<[u8]>) -> BufferNode {
        BufferNode(Rc::new(RefCell::new(InnerBufferNode {
            useage: Useage::NotUsed,
            offset: 0,
            slice: ptr,
        })))
    }
    fn from_raw(slice: &mut [u8], offset: usize) -> BufferNode {
        BufferNode(Rc::new(RefCell::new(InnerBufferNode {
            useage: Useage::NotUsed,
            offset: offset,
            slice: NonNull::new(slice).unwrap(),
        })))
    }
    fn inner(&self) -> Ref<InnerBufferNode> {
        self.0.borrow()
    }
    fn inner_mut(&mut self) -> RefMut<InnerBufferNode> {
        self.0.borrow_mut()
    }
    fn to_used(&mut self) -> Option<()> {
        let mut inner = self.inner_mut();
        inner.to_used()
    }
    fn split(&mut self, mid: usize) -> Option<(BufferNode, BufferNode)> {
        let mut inner = self.inner_mut();
        match inner.useage {
            Useage::NotUsed => {
                let offset = inner.offset;
                let (left, right) = unsafe { inner.slice.as_mut().split_at_mut(mid) };
                let x = (
                    BufferNode::from_raw(left, offset),
                    BufferNode::from_raw(right, offset + mid),
                );
                inner.useage = Useage::Split(x.clone());
                Some(x)
            }
            _ => None,
        }
    }
    fn as_slice(&self) -> Option<&[u8]> {
        let inner = self.inner();
        match inner.useage {
            Useage::Used => Some(unsafe { inner.slice.as_ref() }),
            _ => None,
        }
    }
    fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        let mut inner = self.inner_mut();
        match inner.useage {
            Useage::Used => Some(unsafe { inner.slice.as_mut() }),
            _ => None,
        }
    }
    pub fn len(&self) -> usize {
        self.inner().slice.len()
    }
    pub fn offset(&self) -> usize {
        self.inner().offset
    }
}

pub struct RIOBufferSlice {
    inner: BufferNode,
    buf: Box<RIO_BUF>,
}

impl RIOBufferSlice {
    fn from_node(id: RIO_BUFFERID, node: BufferNode) -> RIOBufferSlice {
        RIOBufferSlice {
            buf: Box::new(RIO_BUF {
                BufferId: id,
                Offset: node.offset() as u32,
                Length: node.len() as u32,
            }),
            inner: node,
        }
    }
    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice().unwrap()
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.inner.as_mut_slice().unwrap()
    }
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
    pub fn buf(&self) -> &RIO_BUF {
        &self.buf
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for RIOBufferSlice {
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &I::Output {
        self.as_slice().index(index)
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for RIOBufferSlice {
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        self.as_mut_slice().index_mut(index)
    }
}
