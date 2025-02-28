use std::{ffi::c_void, io::Error};
use std::sync::OnceLock;
use windows::{
    core::GUID,
    Win32::Networking::WinSock::{
        WSAIoctl, RIO_EXTENSION_FUNCTION_TABLE, SIO_GET_MULTIPLE_EXTENSION_FUNCTION_POINTER,
        SOCKET, WSAID_MULTIPLE_RIO,
    },
};

use crate::net::OverlappedTcpListener;

pub static mut RIO_FUNCTIONS: OnceLock<RIO_EXTENSION_FUNCTION_TABLE> = OnceLock::new();

fn init(sock: SOCKET)-> RIO_EXTENSION_FUNCTION_TABLE {
    unsafe {
        let mut bytes: u32 = 0;
        let guid: GUID = WSAID_MULTIPLE_RIO;
        let mut rio: RIO_EXTENSION_FUNCTION_TABLE = RIO_EXTENSION_FUNCTION_TABLE::default();
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
        rio
    }
}

unsafe fn create_completion_query()->LPFN_RIOCREATECOMPLETIONQUEUE {
    RIO_FUNCTIONS.RIOCreateCompletionQueue
}
unsafe fn create_request_query()->LPFN_RIOCREATEREQUESTQUEUE {
    RIO_FUNCTIONS.RIOCreateRequestQueue
}
unsafe fn receive()->LPFN_RIORECEIVE {
    RIO_FUNCTIONS.RIOReceive
}
unsafe fn receive_ex()->LPFN_RIORECEIVEEX {
    RIO_FUNCTIONS.RIOReceiveEx
}
unsafe fn send()->LPFN_RIOSEND {
    RIO_FUNCTIONS.RIOSend
}
unsafe fn send_ex()->LPFN_RIOSENDEX {
    RIO_FUNCTIONS.RIOSendEx
}
unsafe fn create_query()->LPFN_RIORECEIVE {
    RIO_FUNCTIONS.RIOReceive
}
unsafe fn create_query()->LPFN_RIORECEIVEEX {
    RIO_FUNCTIONS.RIOReceiveEx
}

#[test]
fn test() {
    let sock = OverlappedTcpListener::bind("127.0.0.1:8080").unwrap();
    init(sock.socket)
}
