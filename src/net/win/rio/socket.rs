use std::{
    io::Error,
    net::SocketAddr,
    os::windows::io::{AsRawSocket, RawSocket},
};

use windows::Win32::{
    Foundation::{HANDLE, HANDLE_FLAG_INHERIT, SetHandleInformation},
    Networking::WinSock::{
        AF_INET, AF_INET6, IN_ADDR, IN_ADDR_0, IN6_ADDR, IN6_ADDR_0, SOCKADDR, SOCKADDR_IN,
        SOCKADDR_IN6, SOCKADDR_IN6_0, SOCKADDR_INET, SOCKET, SOCKET_ERROR,
        WSA_FLAG_NO_HANDLE_INHERIT, WSA_FLAG_REGISTERED_IO, WSAConnect, WSAEINVAL, WSAEPROTOTYPE,
        WSASocketW, closesocket,
    },
};

pub trait ToWinSocket {
    fn to_win_socket(&self) -> SOCKET;
}

//Maybe use OwnedSocket
pub struct RIOSocket(SOCKET);

impl RIOSocket {
    pub fn new(addr: &SocketAddr, typ: i32) -> std::io::Result<RIOSocket> {
        let (family, sock_addr) = match *addr {
            SocketAddr::V4(addr) => {
                let address = SOCKADDR_INET {
                    Ipv4: SOCKADDR_IN {
                        sin_family: AF_INET,
                        sin_port: addr.port().to_be(),
                        sin_addr: IN_ADDR {
                            S_un: IN_ADDR_0 {
                                S_addr: addr.ip().clone().into(),
                            },
                        },
                        sin_zero: [0; 8],
                    },
                };
                (AF_INET, address)
            }
            SocketAddr::V6(addr) => {
                let address = SOCKADDR_INET {
                    Ipv6: SOCKADDR_IN6 {
                        sin6_family: AF_INET6,
                        sin6_port: addr.port().to_be(),
                        sin6_flowinfo: addr.flowinfo(),
                        sin6_addr: IN6_ADDR {
                            u: IN6_ADDR_0 {
                                Word: addr.ip().segments(),
                            },
                        },
                        Anonymous: SOCKADDR_IN6_0 {
                            sin6_scope_id: addr.scope_id(),
                        },
                    },
                };
                (AF_INET6, address)
            }
        };
        let socket = match unsafe {
            WSASocketW(
                family.0 as i32,
                typ,
                0,
                None,
                0,
                WSA_FLAG_REGISTERED_IO | WSA_FLAG_NO_HANDLE_INHERIT,
            )
        } {
            Ok(sock) => sock,
            Err(err) => {
                if err.code().0 != WSAEPROTOTYPE.0 && err.code().0 != WSAEINVAL.0 {
                    return Err(Error::from_raw_os_error(err.code().0));
                }

                unsafe {
                    let socket =
                        WSASocketW(family.0 as i32, typ, 0, None, 0, WSA_FLAG_REGISTERED_IO)?;
                    SetHandleInformation(
                        HANDLE(socket.0 as std::os::windows::raw::HANDLE),
                        0,
                        HANDLE_FLAG_INHERIT,
                    )?;

                    socket
                }
            }
        };
        let result = unsafe {
            WSAConnect(
                socket,
                &sock_addr as *const SOCKADDR_INET as *const SOCKADDR,
                size_of::<SOCKADDR_INET>() as i32,
                None,
                None,
                None,
                None,
            )
        };
        if result == SOCKET_ERROR {
            return Err(Error::last_os_error());
        }
        Ok(RIOSocket(socket))
    }
}

impl AsRawSocket for RIOSocket {
    fn as_raw_socket(&self) -> RawSocket {
        self.0.0 as u64
    }
}

impl ToWinSocket for RIOSocket {
    fn to_win_socket(&self) -> SOCKET {
        self.0
    }
}

impl Drop for RIOSocket {
    fn drop(&mut self) {
        unsafe {
            if closesocket(self.0) == SOCKET_ERROR {
                panic!("FATAL: CLosing Socket Error")
            };
        }
    }
}
