use std::{io::Read, net::TcpStream};

use handshake::TLSHandshake;

use crate::server::ServerError;

mod handshake;
mod tls1_0;

pub struct TLSStream {
    stream: TcpStream,
    def_version: TLSVersion,
    cur_buf: Vec<u8>,
}

#[derive(Copy, Clone)]
enum TLSProtocol {
    CipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    Data = 23,
}

impl From<u8> for TLSProtocol {
    fn from(value: u8) -> Self {
        if value >= 20 && value < 24 {
            unsafe { std::mem::transmute::<u8, TLSProtocol>(value) }
        } else {
            panic!("Wrong TLSProtocol Parameter")
        }
    }
}

#[derive(Copy, Clone)]
enum TLSVersion {
    TLS1_0 = 1,
    TLS1_1 = 2,
    TLS1_2 = 3,
    TLS1_3 = 4,
}

impl From<u8> for TLSVersion {
    fn from(value: u8) -> Self {
        if value >= 20 && value < 24 {
            unsafe { std::mem::transmute::<u8, TLSVersion>(value) }
        } else {
            panic!("Wrong TLSVersion Parameter")
        }
    }
}

impl TLSVersion {
    fn cast(major: u8, minor: u8) -> Result<TLSVersion, ServerError> {
        if major != 3 || (minor > 0 && minor < 5) {
            if major < 3 || (major == 3 && minor == 0) {
                return Err(ServerError::VERLOW);
            } else {
                return Err(ServerError::VERHIGH);
            }
        }
        Ok(minor.into())
    }
}

pub const MAX_VERSION: TLSVersion = TLSVersion::TLS1_3;

impl TLSStream {
    pub fn establish(mut stream: TcpStream) -> Result<TLSStream, ServerError> {
        let message: TLSRecordMessage = TLSRecordMessage::from(stream)?;

        Ok(TLSStream { stream: stream })
    }
}

struct TLSRecordMessage {
    protocol: TLSProtocol,
    version: TLSVersion,
    content: TLSPayload,
}

impl TLSRecordMessage {
    fn from(mut stream: TcpStream) -> Result<TLSRecordMessage, ServerError> {
        let mut main: [u8; 5] = [0; 5];
        match stream.read(&mut main) {
            Ok(read) if read == 5 => (),
            _ => return Err(ServerError::CONERR),
        }
        if main[0] < 20 || main[0] > 23 {
            return Err(ServerError::CONERR);
        }
        let protocol: TLSProtocol = main[0].into();
        let version: TLSVersion = TLSVersion::cast(main[1], main[2])?;
        let length: u16 = u16::from_be_bytes([main[3], main[4]]);
        let mut buf: Vec<u8> = Vec::with_capacity(length as usize);
        match stream.read_exact(&mut buf) {
            Ok(_) if buf.len() == length as usize => (),
            _ => return Err(ServerError::CONERR),
        }
        let payload: TLSPayload = TLSPayload::to_payload(protocol, version, &buf)?;
        Ok(TLSRecordMessage {
            protocol: protocol,
            version: version,
            content: payload,
        })
    }
}

enum TLSPayload {
    ChangeCipher(TLSChangeCipher),
    Alert(TLSAlert),
    Handshake(TLSHandshake),
    Application(TLSData),
}

impl TLSPayload {
    fn to_payload(
        kind: TLSProtocol,
        version: TLSVersion,
        buf: &[u8],
    ) -> Result<TLSPayload, ServerError> {
        match version {
            TLSVersion::TLS1_0 => tls1_0::to_payload(kind, buf),
            _ => todo!(),
        }
    }
}

///Avaliable Compression Methods
/// Add if wanted
enum Compression {
    Null = 0,
}

impl Compression {
    fn from(from: &u8)->Result<Compression, ServerError> {
        todo!();
    }
    fn from_slice(&buf: &[u8])->Result<Vec<Compression>, ServerError> {
        buf.bytes().map(|bytes|Compression::from(bytes)).collect()
    }
}

enum Cipher {}

impl Cipher {
    fn from(from: &u8)->Result<Compression, ServerError> {
        todo!();
    }
    fn from_slice(&buf: &[u8])->Result<Vec<Compression>, ServerError> {
        buf.bytes().map(|bytes|Compression::from(bytes)).collect()
    }
}

enum Certificate {
    RSA,
}

enum MACAlgo {
    MMD5,
    SHA1,
}

struct CipherSuite {
    certificate: Certificate,
    mac: MACAlgo,
}
