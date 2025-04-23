use std::{
    future::Future,
    io::Error,
    net::{self, ToSocketAddrs},
    os::windows::io::IntoRawSocket,
    pin::Pin,
    rc::Rc,
    sync::OnceLock,
    task::Poll,
};

use windows::Win32::Foundation::{GetLastError, HANDLE};
use windows::Win32::Networking::WinSock::{
    ADDRESS_FAMILY, LPFN_ACCEPTEX, SIO_GET_EXTENSION_FUNCTION_POINTER, SOCK_STREAM, SOCKADDR,
    SOCKADDR_IN, SOCKET, WSA_FLAG_NO_HANDLE_INHERIT, WSA_FLAG_OVERLAPPED, WSA_IO_INCOMPLETE,
    WSA_IO_PENDING, WSAGetOverlappedResult, WSAID_ACCEPTEX, WSAIoctl, WSASocketW, closesocket,
    getsockname,
};
use windows::Win32::System::IO::OVERLAPPED;
use windows::Win32::System::Threading::CreateEventW;
use windows::core::GUID;

pub type TcpListener = OverlappedTcpListener;
pub type TcpStream = OverlappedTcpStream;

#[derive(Debug)]
pub struct OverlappedTcpListener {
    pub socket: SOCKET,
    family: ADDRESS_FAMILY,
}

#[derive(Debug)]
pub struct OverlappedTcpStream(pub SOCKET);

pub type FutTcpStream = FutOverlappedTcpStream;

pub struct FutOverlappedTcpStream {
    socket: SOCKET,
    overlapped: Pin<Box<OVERLAPPED>>,
}

impl Future for FutOverlappedTcpStream {
    type Output = std::io::Result<OverlappedTcpStream>;
    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe {
            let mut transfered = 0;
            let mut flags = 0;
            match WSAGetOverlappedResult(
                self.socket,
                &*self.overlapped,
                &mut transfered,
                false,
                &mut flags,
            ) {
                Ok(_) => {
                    return Poll::Ready(Ok(OverlappedTcpStream(self.socket)));
                }
                Err(err) => {
                    if err.code().0 == WSA_IO_INCOMPLETE.0 {
                        return Poll::Pending;
                    } else {
                        return Poll::Ready(Err(Error::from_raw_os_error(err.code().0)));
                    }
                }
            }
        }
    }
}

impl Drop for FutOverlappedTcpStream {
    fn drop(&mut self) {
        unsafe {
            closesocket(self.socket);
        }
    }
}

impl OverlappedTcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> std::io::Result<OverlappedTcpListener> {
        let socket: SOCKET = SOCKET(net::TcpListener::bind(addr)?.into_raw_socket() as usize);
        let mut family = SOCKADDR::default();
        unsafe {
            let mut len: i32 = std::mem::size_of::<SOCKADDR>() as i32;
            if getsockname(socket, &mut family, &mut len) != 0 {
                return Err(Error::last_os_error());
            };
        }
        Ok(OverlappedTcpListener {
            socket: socket,
            family: family.sa_family,
        })
    }
    pub fn accept(&self, eventhdl: Option<HANDLE>) -> std::io::Result<FutOverlappedTcpStream> {
        unsafe {
            static mut ACCEPTEX: OnceLock<LPFN_ACCEPTEX> = OnceLock::new();

            unsafe fn acceptex_init(sock: SOCKET) -> LPFN_ACCEPTEX {
                let guid = WSAID_ACCEPTEX;
                let mut bytes: u32 = 0;
                let mut func: LPFN_ACCEPTEX = None;

                let x = unsafe {
                    WSAIoctl(
                        sock,
                        SIO_GET_EXTENSION_FUNCTION_POINTER,
                        Some(&guid as *const GUID as *const std::ffi::c_void),
                        std::mem::size_of::<GUID>() as u32,
                        Some(&mut func as *mut LPFN_ACCEPTEX as *mut std::ffi::c_void),
                        std::mem::size_of::<LPFN_ACCEPTEX>() as u32,
                        &mut bytes as *mut u32,
                        None,
                        None,
                    )
                };
                if x != 0 {
                    let err: Result<(), Error> = Err(Error::last_os_error());
                    err.unwrap();
                };

                func
            }

            #[allow(static_mut_refs)]
            let acceptex = ACCEPTEX.get_or_init(|| acceptex_init(self.socket)).unwrap();

            let _event = match eventhdl {
                Some(event) => event,
                None => match CreateEventW(None, true, false, None) {
                    Ok(handle) => handle,
                    Err(s) => return Err(Error::from_raw_os_error(s.code().0)),
                },
            };

            let mut overlapped: Pin<Box<OVERLAPPED>> = Box::pin(std::mem::zeroed());
            //overlapped.hEvent = event;

            let socket = match WSASocketW(
                self.family.0 as i32,
                SOCK_STREAM.0,
                0,
                None,
                0,
                WSA_FLAG_OVERLAPPED | WSA_FLAG_NO_HANDLE_INHERIT,
            ) {
                Ok(socket) => socket,
                Err(err) => {
                    println!("Hä");
                    return Err(Error::from_raw_os_error(err.code().0));
                }
            };

            let mut buffer: [u8; (std::mem::size_of::<SOCKADDR_IN>() + 16) * 2] = [0; 64];

            if acceptex(
                self.socket,
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

#[test]
fn test() -> std::io::Result<()> {
    let sock = OverlappedTcpListener::bind("127.0.0.1:8080")?;
    let _stream = sock.accept(None)?;

    Ok(())
}

struct OverlappedSocket;

#[derive(Clone)]
#[cfg(feature = "multithreaded")]
pub struct Overlapped(Arc<OVERLAPPED>);

#[cfg(not(feature = "multithreaded"))]
#[derive(Clone)]
pub struct Overlapped(Rc<OVERLAPPED>);

impl Overlapped {
    pub fn new()->Overlapped {
        Overlapped(Rc::new(OVERLAPPED::default()))
    }
    pub fn inner(&self)->&OVERLAPPED {
        &self.0
    }
    pub fn inner_rc(&self)->&Rc<OVERLAPPED> {
        &self.0
    }
}