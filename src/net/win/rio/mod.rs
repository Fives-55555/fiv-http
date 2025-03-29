use std::{io::Error, os::windows::io::AsRawSocket};

mod buffer;
mod comp_queue;
mod request_queue;

pub use buffer::*;
pub use comp_queue::*;
pub use request_queue::*;
pub use riofuncs::init;

mod riofuncs {
    use funcs::*;
    use std::{ffi::c_void, io::Error, os::windows::io::AsRawSocket};
    use windows::{
        Win32::Networking::WinSock::{
            RIO_EXTENSION_FUNCTION_TABLE, SIO_GET_MULTIPLE_EXTENSION_FUNCTION_POINTER, SOCKET,
            WSAID_MULTIPLE_RIO, WSAIoctl,
        },
        core::GUID,
    };

    #[rustfmt::skip]
    #[allow(non_camel_case_types)]
    mod funcs {
        use std::ffi::c_void;
        use windows::{
            core::PCSTR,
            Win32::{
                Foundation::BOOL,
                Networking::WinSock::{
                    RIORESULT, RIO_BUF, RIO_BUFFERID, RIO_CQ, RIO_NOTIFICATION_COMPLETION, RIO_RQ, SOCKET
                },
            },
        };
        
        pub(crate) type FN_RIOCREATECOMPLETIONQUEUE = unsafe extern "system" fn(u32, *const RIO_NOTIFICATION_COMPLETION) -> RIO_CQ;
        pub(crate) type FN_RIORESIZECOMPLETIONQUEUE = unsafe extern "system" fn(RIO_CQ, u32) -> BOOL;
        pub(crate) type FN_RIOCLOSECOMPLETIONQUEUE = unsafe extern "system" fn(RIO_CQ);
        pub(crate) type FN_RIOCREATEREQUESTQUEUE = unsafe extern "system" fn(SOCKET, u32, u32, u32, u32, RIO_CQ, RIO_CQ, *const c_void) -> RIO_RQ;
        pub(crate) type FN_RIORESIZEREQUESTQUEUE = unsafe extern "system" fn(RIO_RQ, u32, u32) -> BOOL;
        pub(crate) type FN_RIORECEIVE = unsafe extern "system" fn(RIO_RQ, *const RIO_BUF, u32, u32, *const c_void) -> BOOL;
        pub(crate) type FN_RIORECEIVEEX = unsafe extern "system" fn(RIO_RQ, *const RIO_BUF, u32, *const RIO_BUF, *const RIO_BUF, *const RIO_BUF, *const RIO_BUF, u32, *const c_void) -> i32;
        pub(crate) type FN_RIOSEND = unsafe extern "system" fn(RIO_RQ, *const RIO_BUF, u32, u32, *const c_void) -> BOOL;
        pub(crate) type FN_RIOSENDEX = unsafe extern "system" fn(RIO_RQ, *const RIO_BUF, u32, *const RIO_BUF, *const RIO_BUF, *const RIO_BUF, *const RIO_BUF, u32, *const c_void) -> BOOL;
        pub(crate) type FN_RIOREGISTERBUFFER = unsafe extern "system" fn(PCSTR, u32) -> RIO_BUFFERID;
        pub(crate) type FN_RIODEREGISTERBUFFER = unsafe extern "system" fn(RIO_BUFFERID);
        pub(crate) type FN_RIONOTIFY = unsafe extern "system" fn(RIO_CQ) -> i32;
        pub(crate) type FN_RIODEQUEUECOMPLETION = unsafe extern "system" fn(RIO_CQ, *mut RIORESULT, u32) -> u32;
    }

    static mut RIO_FUNCTIONS: RIO_EXTENSION_FUNCTION_TABLE = unsafe { std::mem::zeroed() };

    pub fn init() {
        let sock = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let mut bytes: u32 = 0;
        let guid: GUID = WSAID_MULTIPLE_RIO;
        let mut rio: RIO_EXTENSION_FUNCTION_TABLE = RIO_EXTENSION_FUNCTION_TABLE::default();
        let func_len = std::mem::size_of::<RIO_EXTENSION_FUNCTION_TABLE>() as u32;
        unsafe {
            if WSAIoctl(
                SOCKET(sock.as_raw_socket() as usize),
                SIO_GET_MULTIPLE_EXTENSION_FUNCTION_POINTER,
                Some(&guid as *const GUID as *const c_void),
                std::mem::size_of::<GUID>() as u32,
                Some(&mut rio as *mut RIO_EXTENSION_FUNCTION_TABLE as *mut c_void),
                func_len,
                &mut bytes,
                None,
                None,
            ) != 0
                || bytes != func_len
            {
                let err: Result<(), Error> = Err(Error::last_os_error());
                err.unwrap();
            }
            RIO_FUNCTIONS = rio
        }
    }
    pub(crate) unsafe fn create_completion_queue() -> FN_RIOCREATECOMPLETIONQUEUE {
        unsafe {
            RIO_FUNCTIONS
                .RIOCreateCompletionQueue
                .expect("Libary never initaliesed")
        }
    }
    pub(crate) unsafe fn resize_completion_queue() -> FN_RIORESIZECOMPLETIONQUEUE {
        unsafe {
            RIO_FUNCTIONS.RIOResizeCompletionQueue.expect("Libary never initaliesed")
        }
    }
    pub(crate) unsafe fn close_completion_queue() -> FN_RIOCLOSECOMPLETIONQUEUE {
        unsafe {
            RIO_FUNCTIONS
                .RIOCloseCompletionQueue
                .expect("Libary never initaliesed")
        }
    }
    pub(crate) unsafe fn create_request_queue() -> FN_RIOCREATEREQUESTQUEUE {
        unsafe {
            RIO_FUNCTIONS
                .RIOCreateRequestQueue
                .expect("Libary never initaliesed")
        }
    }
    pub(crate) unsafe fn resize_request_queue() -> FN_RIORESIZEREQUESTQUEUE {
        unsafe {
            RIO_FUNCTIONS
                .RIOResizeRequestQueue
                .expect("Libary never initaliesed")
        }
    }
    pub(crate) unsafe fn receive() -> FN_RIORECEIVE {
        unsafe { RIO_FUNCTIONS.RIOReceive.expect("Libary never initaliesed") }
    }
    pub(crate) unsafe fn receive_ex() -> FN_RIORECEIVEEX {
        unsafe {
            RIO_FUNCTIONS
                .RIOReceiveEx
                .expect("Libary never initaliesed")
        }
    }
    pub(crate) unsafe fn send() -> FN_RIOSEND {
        unsafe { RIO_FUNCTIONS.RIOSend.expect("Libary never initaliesed") }
    }
    pub(crate) unsafe fn send_ex() -> FN_RIOSENDEX {
        unsafe { RIO_FUNCTIONS.RIOSendEx.expect("Libary never initaliesed") }
    }
    pub(crate) unsafe fn register_buffer() -> FN_RIOREGISTERBUFFER {
        unsafe {
            RIO_FUNCTIONS.RIORegisterBuffer.expect("Libary never initaliesed")
        }
    }
    pub(crate) unsafe fn deregister_buffer() -> FN_RIODEREGISTERBUFFER {
        unsafe {
            RIO_FUNCTIONS
                .RIODeregisterBuffer
                .expect("Libary never initaliesed")
        }
    }
    pub(crate) unsafe fn notify() -> FN_RIONOTIFY {
        unsafe { RIO_FUNCTIONS.RIONotify.expect("Libary never initaliesed") }
    }
    pub(crate) unsafe fn dequeue() -> FN_RIODEQUEUECOMPLETION {
        unsafe {
            RIO_FUNCTIONS
                .RIODequeueCompletion
                .expect("Libary never initaliesed")
        }
    }
}

use super::iocp::IOCP;

pub struct RegisteredTcpStream<'a> {
    queue: RequestQueue<'a>,
    // Maybe abstract to use also the Event
    send: CompletionQueue,
    recv: CompletionQueue,
}

impl<'a> RegisteredTcpStream<'a> {
    pub const DEFAULT_THEAD_AMOUNT: u32 = 0;
    pub const DEFAULT_QUEUE_SIZE: u32 = 1024;
    pub fn new<T: AsRawSocket>(sock: T) -> Result<RegisteredTcpStream<'a>, Error> {
        let iocp: IOCP = IOCP::new()?;
        let mut send: CompletionQueue = CompletionQueue::new_iocp(Self::DEFAULT_QUEUE_SIZE, iocp.clone())?;
        let mut recv: CompletionQueue = CompletionQueue::new_iocp(Self::DEFAULT_QUEUE_SIZE, iocp)?;
        let queue: RequestQueue = RequestQueue::new(
            sock,
            recv.inner_mut(),
            Self::DEFAULT_QUEUE_SIZE,
            send.inner_mut(),
            Self::DEFAULT_QUEUE_SIZE,
        )?;
        let stream = RegisteredTcpStream {
            queue: queue,
            send: send,
            recv: recv,
        };
        Ok(stream)
    }
    pub fn read() -> u8 {
        todo!()
    }
    pub fn write() -> u8 {
        todo!()
    }
}

impl<'a> AsRawSocket for RegisteredTcpStream<'a> {
    fn as_raw_socket(&self) -> std::os::windows::prelude::RawSocket {
        self.queue.socket().0 as u64
    }
}
