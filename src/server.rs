use std::{error::Error, fmt::Display, io::Read};

use crate::tls::TLSStream;

pub struct Server<T: ToServer> {
    inner: T,
}

#[derive(Debug)]
pub struct ServerError(u8);

pub trait ToServer {
    type Builder;
    fn new() -> Self::Builder;
    fn open(builder: Self::Builder) -> Result<(), ServerError>;
}

impl ServerError {
    pub const INVPAR: ServerError = ServerError(126);
    pub const CONERR: ServerError = ServerError(127);
    pub const VERLOW: ServerError = ServerError(128);
    pub const VERHIGH: ServerError = ServerError(129);
    pub fn reason(&self) -> &str {
        match self.0 {
            126 => "Invalid Parameter",
            127 => "Connection Error",
            128 => "Version too Low",
            129 => "Version too High",
            _ => "Error not defined",
        }
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason())
    }
}

impl Error for ServerError {}

pub struct TLS {}

pub struct TLSBuilder {}

pub struct HTTP {}

pub struct HTTPBuilder {}

impl ToServer for HTTP {
    type Builder = HTTPBuilder;
    fn new() -> Self::Builder {
        HTTPBuilder {}
    }
    fn open(builder: HTTPBuilder) -> Result<(), ServerError> {
        loop {
            let socket = std::net::TcpListener::bind("127.0.0.1:80").unwrap();
            for stream in socket.incoming() {
                let mut buf: [u8; 10240] = [0; 10240];
                stream.unwrap().read(&mut buf).unwrap();
                println!("{:?}\n\n\n{}", buf, unsafe {
                    String::from_utf8_unchecked(buf.to_vec())
                })
            }
        }
    }
}

impl ToServer for TLS {
    type Builder = HTTPBuilder;
    fn new() -> Self::Builder {
        HTTPBuilder {}
    }
    fn open(builder: HTTPBuilder) -> Result<(), ServerError> {
        loop {
            let socket = std::net::TcpListener::bind("127.0.0.1:443").unwrap();
            for stream in socket.incoming() {
                let stream: TLSStream = TLSStream::establish(stream.unwrap())?;
            }
        }
    }
}
