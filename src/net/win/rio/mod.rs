use std::fmt::Display;

mod socket;

mod buffer;
pub use buffer::{RIOBuffer, RIOBufferSlice};

mod comp_queue;
pub use comp_queue::RIOCompletionQueue;

mod request_queue;
use comp_queue::RIOPoll;
pub use request_queue::RequestQueue;

pub use riofuncs::init;

mod stream;
pub use stream::RegisteredTcpStream;

use windows::Win32::Networking::WinSock::{RIO_CQ, RIO_RQ, RIORESULT};

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
        /// Enables the notification type, resets after one event
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
            RIO_FUNCTIONS
                .RIOResizeCompletionQueue
                .expect("Libary never initaliesed")
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
            RIO_FUNCTIONS
                .RIORegisterBuffer
                .expect("Libary never initaliesed")
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

#[repr(transparent)]
#[derive(Clone)]
pub struct RIOEvent(RIORESULT);

impl RIOEvent {
    pub fn new() -> RIOEvent {
        RIOEvent(RIORESULT::default())
    }
    pub fn is_ok(&self) -> bool {
        self.status() == 0
    }
    pub fn is_err(&self) -> bool {
        self.status() != 0
    }
    pub fn is_some(&self) -> bool {
        self.0.BytesTransferred != 0
    }
    pub fn status(&self) -> i32 {
        self.0.Status
    }
    pub fn transfered(&self) -> usize {
        self.0.BytesTransferred as usize
    }
    pub fn socket(&self) -> SocketAlias {
        self.0.SocketContext
    }
    pub fn io_action(&self) -> IOAlias {
        self.0.RequestContext
    }
    pub fn as_result(&mut self) -> &mut RIORESULT {
        &mut self.0
    }
    pub fn as_poll(&self) -> RIOPoll {
        RIOPoll::from_raw(self.socket(), self.io_action(), self.transfered())
    }
}

impl Display for RIOEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Status: {}, Is Err: {}, SocketContext: {}, IOContext: {}, Bytes transferd: {}",
            self.status(),
            self.is_err(),
            self.socket(),
            self.io_action(),
            self.transfered()
        )
    }
}

pub struct RIOIoOP {
    ioalias: IOAlias,
    buffer: RIOBufferSlice,
    len: usize,
}

impl RIOIoOP {
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }
    pub fn buf(self) -> RIOBufferSlice {
        self.buffer
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn ioalias(&self)->IOAlias {
        self.ioalias
    }
}

pub type SocketAlias = u64;

pub type IOAlias = u64;

#[test]
fn test() -> std::io::Result<()> {
    init();

    let mut buffer = RIOBuffer::new().unwrap();
    let slice = buffer.alloc_whole().unwrap();

    let reg = RegisteredTcpStream::connect("127.0.0.1:8080").unwrap();

    println!("What");

    println!("Waiting");

    drop(reg);
    drop(slice);

    Ok(())
}

pub const RIO_INVALID_RQ: RIO_RQ = RIO_RQ(0);
pub const RIO_INVALID_CQ: RIO_CQ = RIO_CQ(0);
