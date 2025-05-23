use std::net::SocketAddr;

use windows::Win32::Networking::WinSock::{WSA_FLAG_OVERLAPPED, WSA_FLAG_REGISTERED_IO};

use crate::LLProtocol;

pub struct FivSocket {
    sock: u64,
    initflags: u32,
    proto: LLProtocol,
    bound: Option<Bound>,
}

pub struct Bound {
    addrs: SocketAddr,
    direction: Direction,
}

enum Direction {
    Host,
    Client
}

impl Direction {
    pub const HOST: Direction = Direction::Host;
    pub const CLIENT: Direction = Direction::Client;    
    pub fn is_host(&self)->bool {
        match self {
            Direction::Host =>true,
            Direction::Client=>false,
        }
    }
    pub fn is_client(&self)->bool {
        match self {
            Direction::Client=>true,
            Direction::Host =>false,
        }
    }
}

impl FivSocket {
    pub fn is_bound(&self)->bool {
        match self.bound {
            Some(_)=>true,
            None=>false,
        }
    }
    pub fn can_rio(&self)->bool {
        self.flags() & WSA_FLAG_REGISTERED_IO != 0
    }
    pub fn can_overlapped(&self)->bool {
        self.flags() & WSA_FLAG_OVERLAPPED != 0
    }
    pub fn can_iocp(&self)->bool {
        self.can_overlapped()
    }
    pub fn flags(&self)->u32 {
        self.initflags
    }
}