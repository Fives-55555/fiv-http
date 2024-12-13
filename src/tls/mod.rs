use std::{io::Read, net::TcpStream};

use crate::server::ServerError;


pub struct TLSStream {
    protocol: TLSProtocol,
    version: TLSVersion,
}

pub struct TLSAda {
    layer: TLSProtocol,
}

enum TLSProtocol {
    CipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    Data = 23
}

impl From<u8> for TLSProtocol {
    fn from(value: u8) -> Self {
        if value >= 20 && value < 24 {
            unsafe{std::mem::transmute::<u8, TLSProtocol>(value)}
        } else {
            panic!("Wrong TLSProtocol Parameter")
        }
    }
}

impl From<u8> for TLSVersion {
    fn from(value: u8) -> Self {
        if value >= 20 && value < 24 {
            unsafe{std::mem::transmute::<u8, TLSVersion>(value)}
        } else {
            panic!("Wrong TLSVersion Parameter")
        }
    }
}

enum TLSVersion {
    TLS1_0 = 1,
    TLS1_1 = 2,
    TLS1_2 = 3,
    TLS1_3 = 4
}

impl TLSVersion {
    fn cast(major: u8, minor: u8)->Result<TLSVersion, ServerError> {
        if major != 3 || (minor > 0 && minor < 5) {
            return Err(ServerError::CONERR);
        }
        Ok(minor.into())
    }
}


impl TLSStream {
    pub fn establish(mut stream: TcpStream)->Result<TLSStream, ServerError> {
        let mut main: [u8; 5] = [0;5];
        match stream.read(&mut main) {
            Ok(read) if read == 5=>(),
            _=>return Err(ServerError::CONERR)
        }
        if main[0] < 20 || main[0] > 23 {
            return Err(ServerError::CONERR)
        }
        let protocol: TLSProtocol = main[0].into();
        let version: TLSVersion = TLSVersion::cast(main[1], main[2])?;
        let length: u16 = u16::from_be_bytes([main[3], main[4]]);

        TLSPayload::cast()

        Ok(TLSStream{

        })
    }
}
struct TLSRecord {
    protocol: TLSProtocol,
    version: TLSVersion,
    content: TLSPayload
}

enum TLSPayload {
    ChangeCipher(TLSChangeCipher),
    Alert(TLSAlert),
    Handshake(TLSHandshake),
    Application(TLSData),
}

impl TLSPayload {
    fn cast(kind: TLSProtocol, buf: &[u8])->TLSPayload {
        match kind {
            TLSProtocol::Handshake=>TLSHandshake::cast(),
            _=>todo!("Not impl")
        }
    }
}

struct TLSHandshake {
    kind: TLSHandshakeType,
    
}

enum TLSHandshakeType {
    HelloRequest = 0,
    ClientHello = 1,
    ServerHello = 2,
    Certificate = 11,
    ServerKeyExchange = 12,
    CertificateRequest = 13,
    ServerHelloDone = 14,
    CertificateVerify = 15,
    ClientKeyExchange = 16,
    Finished = 20
}