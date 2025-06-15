use std::{
    io::Error,
    net::{SocketAddr, ToSocketAddrs},
    sync::OnceLock,
};

use windows::{core::GUID, Win32::{Networking::WinSock::{
    WSAIoctl, LPFN_ACCEPTEX, SIO_GET_EXTENSION_FUNCTION_POINTER, SOCKET, WSAID_ACCEPTEX
}, System::IO::OVERLAPPED}};
use windows_result::BOOL;

use crate::{for_each_addrs, win::socket::FivSocket};

use super::Overlapped;

pub type ACCEPTEXFN = unsafe extern "system" fn(
    slistensocket: SOCKET,
    sacceptsocket: SOCKET,
    lpoutputbuffer: *mut core::ffi::c_void,
    dwreceivedatalength: u32,
    dwlocaladdresslength: u32,
    dwremoteaddresslength: u32,
    lpdwbytesreceived: *mut u32,
    lpoverlapped: *mut OVERLAPPED,
) -> BOOL;

static mut ACCEPTEX: OnceLock<ACCEPTEXFN> = OnceLock::new();

unsafe fn acceptex_init(sock: SOCKET) -> ACCEPTEXFN {
    let guid = WSAID_ACCEPTEX;
    let mut bytes: u32 = 0;
    let mut func: LPFN_ACCEPTEX = None;

    let fn_len = std::mem::size_of::<LPFN_ACCEPTEX>();

    let x = unsafe {
        WSAIoctl(
            sock,
            SIO_GET_EXTENSION_FUNCTION_POINTER,
            Some(&guid as *const GUID as *const std::ffi::c_void),
            std::mem::size_of::<GUID>() as u32,
            Some(&mut func as *mut LPFN_ACCEPTEX as *mut std::ffi::c_void),
            fn_len as u32,
            &mut bytes as *mut u32,
            None,
            None,
        )
    };
    if x != 0 && x == fn_len as i32 {
        let err: Result<(), Error> = Err(Error::last_os_error());
        err.unwrap();
    };
    func.unwrap()
}

pub struct OverlappedListener {
    socket: FivSocket,
    overlapped: Overlapped,
}

impl OverlappedListener {
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> std::io::Result<OverlappedListener> {
        for_each_addrs(addrs, OverlappedListener::bind_single)
    }
    fn bind_single(addr: SocketAddr) -> std::io::Result<OverlappedListener> {
        let mut sock = FivSocket::new_overlapped()?;
        sock.bind(addr)?;
        sock.listen()?;
        Ok(OverlappedListener {
            socket: sock,
            overlapped: Overlapped::new(),
        })
    }
    pub fn accept(&self, socket: FivSocket, eventhdl: Option<HANDLE>) -> std::io::Result<FutOverlappedTcpStream> {
        unsafe {
            let acceptex = ACCEPTEX.get_or_init(||acceptex_init(self.socket.win_socket()));

            let _event = match eventhdl {
                Some(event) => event,
                None => match CreateEventW(None, true, false, None) {
                    Ok(handle) => handle,
                    Err(s) => return Err(Error::from_raw_os_error(s.code().0)),
                },
            };

            let mut buffer: [u8; (std::mem::size_of::<SOCKADDR_IN>() + 16) * 2] = [0; 64];

            if acceptex(
                self.socket.win_socket(),
                socket,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                0,
                std::mem::size_of::<SOCKADDR_IN>() as u32 + 16,
                std::mem::size_of::<SOCKADDR_IN>() as u32 + 16,
                std::ptr::null_mut(),
                &mut *overlapped,
            )
            .0 != 0
            {
                return Ok(FutOverlappedTcpStream {
                    socket: socket,
                    overlapped: overlapped,
                });
            }
            let result = GetLastError();
            if result.0 != WSA_IO_PENDING.0 as u32 {
                return Err(Error::from_raw_os_error(result.to_hresult().0));
            }

            Ok(FutOverlappedTcpStream {
                socket: socket,
                overlapped,
            })
        }
    }
}
