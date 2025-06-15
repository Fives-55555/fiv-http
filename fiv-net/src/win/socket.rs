use std::{
    io::Error,
    net::SocketAddr,
    os::{raw::c_void, windows::io::AsRawSocket},
};

use windows::Win32::{
    Foundation::{SetHandleInformation, HANDLE, HANDLE_FLAGS},
    Networking::WinSock::{
        bind, closesocket, listen, WSASocketW, SOCKADDR, SOCKADDR_INET, SOCKET, WSAEINVAL, WSAEPROTOTYPE
    },
};

use crate::{AddrsFamily, LLProtocol};

use super::LISTEN_BACKLOG;

pub struct FivSocket {
    sock: u64,
    initflags: u32,
    family: AddrsFamily,
    proto: LLProtocol,
    //This is the local_addrs which gets set by bind or connect(under teh hod)
    local_addr: Option<SocketAddr>,
    remote_addr: Option<SocketAddr>,
    nameme1: Option<NAMEME1>,
}

pub const OVERRLAPPED_CAPABLE: u32 = 1;
pub const NO_HANDLE_INHERIT: u32 = 128;
pub const RIO_CAPABLE: u32 = 256;

impl FivSocket {
    pub fn new() -> std::io::Result<FivSocket> {
        let address_family = AddrsFamily::IPV4;
        let protocol = LLProtocol::TCP;
        let flags = 0;
        FivSocket::new_raw(address_family, protocol, flags)
    }
    pub fn new_overlapped() -> std::io::Result<FivSocket> {
        let address_family = AddrsFamily::IPV4;
        let protocol = LLProtocol::TCP;
        let flags = OVERRLAPPED_CAPABLE;
        FivSocket::new_raw(address_family, protocol, flags)
    }
    pub fn new_rio() -> std::io::Result<FivSocket> {
        let address_family = AddrsFamily::IPV4;
        let protocol = LLProtocol::TCP;
        let flags = RIO_CAPABLE;
        FivSocket::new_raw(address_family, protocol, flags)
    }
    pub fn new_raw(
        family: AddrsFamily,
        protocol: LLProtocol,
        flags: u32,
    ) -> std::io::Result<FivSocket> {
        let result = unsafe {
            WSASocketW(
                family as i32,
                protocol.to_type(),
                protocol.to_proto(),
                None,
                0,
                flags | NO_HANDLE_INHERIT,
            )
        };
        let sock = match result {
            Ok(sock) => sock,
            Err(err) => {
                let code = err.code().0;
                if code != WSAEPROTOTYPE.0 && code != WSAEINVAL.0 {
                    return Err(Error::from_raw_os_error(code));
                }

                let result = unsafe {
                    WSASocketW(
                        family as i32,
                        protocol.to_type(),
                        protocol.to_proto(),
                        None,
                        0,
                        flags & (!NO_HANDLE_INHERIT),
                    )
                };
                let sec_sock = match result {
                    Ok(sec_sock) => sec_sock,
                    Err(sec_err) => return io_err_res!(sec_err),
                };
                let result = unsafe {
                    SetHandleInformation(HANDLE(sec_sock.0 as *mut c_void), 1, HANDLE_FLAGS(0))
                };
                match result {
                    Ok(()) => sec_sock,
                    Err(err) => return io_err_res!(err),
                }
            }
        };
        Ok(FivSocket {
            sock: sock.0 as u64,
            initflags: flags,
            family: family,
            proto: protocol,
            local_addr: None,
            remote_addr: None,
            nameme1: None,
        })
    }
    pub fn bind(&mut self, addr: SocketAddr) -> std::io::Result<()> {
        let win_addr: SOCKADDR_INET = addr.into();
        let result = unsafe {
            bind(
                self.win_socket(),
                &win_addr as *const SOCKADDR_INET as *const SOCKADDR,
                size_of::<SOCKADDR>() as i32,
            )
        };
        if result == 0 {
            self.local_addr = Some(addr);
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }
    pub fn listen(&mut self) -> std::io::Result<()> {
        let result = unsafe { listen(self.win_socket(), LISTEN_BACKLOG) };
        if result == 0 {
            self.set_nameme1(NAMEME1::Listener);
            Ok(())
        } else {
            return Err(Error::last_os_error());
        }
    }
    pub fn from_raw(
        socket: u64,
        flags: u32,
        local_addr: Option<SocketAddr>,
        remote_addr: Option<SocketAddr>,
        protocol: LLProtocol,
        addrs_family: AddrsFamily,
        nameme1: Option<NAMEME1>,
    ) -> FivSocket {
        FivSocket {
            sock: socket,
            initflags: flags,
            family: addrs_family,
            proto: protocol,
            local_addr: local_addr,
            remote_addr: remote_addr,
            nameme1: nameme1,
        }
    }
    pub fn protocol(&self) -> &LLProtocol {
        &self.proto
    }
    pub fn is_tcp(&self) -> bool {
        match self.proto {
            LLProtocol::TCP => true,
            LLProtocol::UDP => false,
        }
    }
    pub fn is_udp(&self) -> bool {
        match self.proto {
            LLProtocol::UDP => true,
            LLProtocol::TCP => false,
        }
    }
    pub fn is_bound(&self) -> bool {
        match self.nameme1 {
            Some(_) => true,
            None => false,
        }
    }
    pub fn set_nameme1(&mut self, nameme1: NAMEME1) {
        self.nameme1 = Some(nameme1)
    }
    pub fn can_rio(&self) -> bool {
        self.flags() & RIO_CAPABLE != 0
    }
    pub fn can_overlapped(&self) -> bool {
        self.flags() & OVERRLAPPED_CAPABLE != 0
    }
    pub fn can_iocp(&self) -> bool {
        self.can_overlapped()
    }
    pub fn flags(&self) -> u32 {
        self.initflags
    }
    pub fn family(&self) -> &AddrsFamily {
        &self.family
    }
    pub fn win_socket(&self) -> SOCKET {
        SOCKET(self.sock as usize)
    }
}

impl AsRawSocket for FivSocket {
    fn as_raw_socket(&self) -> std::os::windows::prelude::RawSocket {
        self.sock
    }
}

impl Drop for FivSocket {
    fn drop(&mut self) {
        unsafe {
            if closesocket(self.win_socket()) == -1 {
                panic!("FATAL: CLosing Socket Error")
            };
        }
    }
}

pub enum NAMEME1 {
    Listener,
    Stream(Direction),
}

impl NAMEME1 {
    pub fn is_listener(&self) -> bool {
        match self {
            NAMEME1::Listener => true,
            NAMEME1::Stream(_) => false,
        }
    }
    pub fn is_stream(&self) -> bool {
        match self {
            NAMEME1::Stream(_) => true,
            NAMEME1::Listener => false,
        }
    }
}

pub enum Direction {
    Host,
    Client,
}

impl Direction {
    pub const HOST: Direction = Direction::Host;
    pub const CLIENT: Direction = Direction::Client;
    pub fn is_host(&self) -> bool {
        match self {
            Direction::Host => true,
            Direction::Client => false,
        }
    }
    pub fn is_client(&self) -> bool {
        match self {
            Direction::Client => true,
            Direction::Host => false,
        }
    }
}
