use std::{ffi::c_void, io::Error};

use windows::{
    core::GUID,
    Win32::Networking::WinSock::{
        WSAIoctl, RIO_EXTENSION_FUNCTION_TABLE, SIO_GET_MULTIPLE_EXTENSION_FUNCTION_POINTER,
        SOCKET, WSAID_MULTIPLE_RIO,
    },
};

use crate::net::OverlappedTcpListener;

pub static mut RIO_FUNCTIONS: RIO_EXTENSION_FUNCTION_TABLE = unsafe { std::mem::zeroed() };

fn init(sock: SOCKET) {
    unsafe {
        let guid: GUID = WSAID_MULTIPLE_RIO;
        let mut rio: RIO_EXTENSION_FUNCTION_TABLE = RIO_EXTENSION_FUNCTION_TABLE::default();
        let mut bytes: u32 = 0;
        let func_len = std::mem::size_of::<RIO_EXTENSION_FUNCTION_TABLE>() as u32;
        if WSAIoctl(
            sock,
            SIO_GET_MULTIPLE_EXTENSION_FUNCTION_POINTER,
            Some(&guid as *const GUID as *const c_void),
            std::mem::size_of::<GUID>() as u32,
            Some(&mut rio as *mut RIO_EXTENSION_FUNCTION_TABLE as *mut c_void),
            func_len,
            &mut bytes,
            None,
            None,
        ) != 0
            && bytes != func_len
        {
            let err: Result<(), Error> = Err(Error::last_os_error());
            err.unwrap();
        }
        RIO_FUNCTIONS = rio;
    }
}

fn create_query()->u32 {
    RIO_FUNCTIONS.RIOCreateCompletionQueue;
    RIO_FUNCTIONS.RIOCreateRequestQueue
}

#[test]
fn test() {
    let sock = OverlappedTcpListener::bind("127.0.0.1:8080").unwrap();
    init(sock.socket)
}
