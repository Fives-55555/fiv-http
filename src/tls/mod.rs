use std::{io::Read, net::TcpStream};

use crate::server::ServerError;


pub struct TLSStream {
    stream: TcpStream,
    def_version: TLSVersion,
    cur_buf: Vec<u8>,
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

enum TLSVersion {
    TLS1_0 = 1,
    TLS1_1 = 2,
    TLS1_2 = 3,
    TLS1_3 = 4
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
        let message = TLSVersion::cast(stream);

        
        Ok(TLSStream{

        })
    }
}
struct TLSRecordMessage {
    protocol: TLSProtocol,
    version: TLSVersion,
    content: TLSPayload
}

impl TLSRecordMessage {
    fn from(mut stream: TcpStream)->TLSRecordMessage {
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
        let mut buf = Vec::with_cappacity(length as usize);
        match stream.read_exact(&mut buf) {
            Ok(read) if read == length as usize=>(),
            _=>return Err(ServerError::CONERR),
        }
        let payload: TLSPayload = TLSPayload::to_payload()?;
        TLSRecordMessage{
            protocol: protocol,
            version: version,
            content: payload,
        }
    }
}

enum TLSPayload {
    ChangeCipher(TLSChangeCipher),
    Alert(TLSAlert),
    Handshake(TLSHandshake),
    Application(TLSData),
}

impl TLSPayload {
    fn to_payload(kind: TLSProtocol, version: TLSVersion, buf: &[u8])->Result<TLSPayload, ServerError> {
        match version {
            TLSVersion::TLS1_0=>{
                match kind {
                    TLSProtocol::Handshake=>tls_1_0::to_handshake(buf),
                    _=>todo!(),
                }
            },
            _=>todo!(),
        }
    }
}

struct TLSHandshake {
    kind: TLSHandshakeType,
    payload: TLSHandshakePayload,
}

mod Handshake {
    enum TLSHandshakeType {
        HelloRequest() = 0,
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
}

mod tls_1_0 {
    fn to_handshake(buf: &[u8])->Result<TLSPayload, ServerError> {
        let kind: TLSHandshakeType = buf[0];
        let length: u32 = u32::from_be_bytes([0, buf[1] , buf[2], buf[3]]);
        
        
    }
    
}
