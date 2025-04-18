use std::{fmt::Display, io::ErrorKind, net::{SocketAddr, ToSocketAddrs}, os::windows::io::AsRawSocket};

mod buffer;
mod comp_queue;
mod request_queue;
mod socket;

pub use buffer::*;
pub use comp_queue::*;
pub use request_queue::*;
pub use riofuncs::init;
use socket::RIOSocket;
use windows::Win32::Networking::WinSock::{RIORESULT, SOCK_STREAM};

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

use super::iocp::IOCP;

pub struct RegisteredTcpStream {
    queue: RequestQueue,
    // Maybe abstract to use also the Event
    send: CompletionQueue,
    recv: CompletionQueue,
}

impl RegisteredTcpStream {
    pub const DEFAULT_THEAD_AMOUNT: u32 = 0;
    pub const DEFAULT_QUEUE_SIZE: usize = 1024;
    pub fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<RegisteredTcpStream> {
        
        let addrs = match addr.to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(e) => return Err(e),
        };
        let mut last_err = None;
        for addr in addrs {
            match RegisteredTcpStream::single_connect(&addr) {
                Ok(l) => return Ok(l),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| {
            std::io::Error::new(ErrorKind::InvalidInput, "could not resolve to any addresses")
        }))
    }
    fn single_connect(addr: &SocketAddr)->std::io::Result<RegisteredTcpStream> {
        let sock = RIOSocket::new(addr, SOCK_STREAM.0)?;
        let iocp: IOCP = IOCP::new()?;
        let send: CompletionQueue =
            CompletionQueue::new_iocp(Self::DEFAULT_QUEUE_SIZE, iocp.clone())?;
        let recv: CompletionQueue = CompletionQueue::new_iocp(Self::DEFAULT_QUEUE_SIZE, iocp)?;
        let queue: RequestQueue = RequestQueue::from_raw(
            sock,
            &send,
            Self::DEFAULT_QUEUE_SIZE,
            &recv,
            Self::DEFAULT_QUEUE_SIZE,
        )?;
        let stream: RegisteredTcpStream = RegisteredTcpStream {
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

impl AsRawSocket for RegisteredTcpStream {
    fn as_raw_socket(&self) -> std::os::windows::prelude::RawSocket {
        self.queue.socket().0 as u64
    }
}

pub struct RIOEvent(RIORESULT);

impl RIOEvent {
    pub fn new()->RIOEvent {
        RIOEvent(RIORESULT::default())
    }
    pub fn is_ok(&self)->bool {
        self.status() == 0
    }
    pub fn is_err(&self)->bool {
        self.status() != 0
    }
    pub fn is_some(&self)->bool {
        self.0.BytesTransferred!=0
    }
    pub fn status(&self)->i32 {
        self.0.Status
    }
    pub fn transfered(&self)->u32 {
        self.0.BytesTransferred
    }
    pub fn socket(&self)->SocketAlias {
        self.0.SocketContext
    }
    pub fn io_action(&self)->IOAlias {
        self.0.RequestContext
    }
    pub fn as_result(&mut self)->&mut RIORESULT {
        &mut self.0
    }
}

impl Display for RIOEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status: {}, Is Err: {}, SocketContext: {}, IOContext: {}, Bytes transferd: {}", self.status(), self.is_err(), self.socket(), self.io_action(), self.transfered())
    }
}

pub type SocketAlias = u64;
pub type IOAlias = u64; 

#[test]
fn test() -> std::io::Result<()> {

    init();

    let mut buffer = RIOBuffer::new().unwrap();
    let mut slice = buffer.get_whole().unwrap();

    let reg = RegisteredTcpStream::connect("127.0.0.1:8080").unwrap();

    println!("What");

    
    println!("Waiting");

    // let x = recv.await_and_compl().unwrap();
// 
    // println!("{}", x);
// 
    // println!("Waiting, Done");
    
    drop(reg);
    
    Ok(())
}

pub const RIO_INVALID_RQ: isize = 0;
pub const RIO_INVALID_CQ: isize = 0;