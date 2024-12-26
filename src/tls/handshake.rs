use crate::server::ServerError;

use super::{Compression, TLSVersion};

pub struct TLSHandshake {
    kind: TLSHandshakeType,
    payload: TLSHandshakePayload,
}

pub(super) enum TLSHandshakeType {
    HelloRequest = 0,
    ClientHello = 1,
    ServerHello = 2,
    Certificate = 11,
    ServerKeyExchange = 12,
    CertificateRequest = 13,
    ServerHelloDone = 14,
    CertificateVerify = 15,
    ClientKeyExchange = 16,
    Finished = 20,
}

impl From<u8> for TLSHandshakeType {
    fn from(value: u8) -> Self {
        if value < 3 || (value > 10 && value < 17) || value == 20 {
            unsafe { std::mem::transmute::<u8, TLSHandshakeType>(value) }
        } else {
            panic!("Wrong TLSHandshakeType Parameter")
        }
    }
}

impl TLSHandshakeType {
    pub(super) fn cast(value: u8) -> Result<TLSHandshakeType, ServerError> {
        if value < 3 || (value > 10 && value < 17) || value == 20 {
            Ok(value.into())
        } else {
            Err(ServerError::CONERR)
        }
    }
}

pub(super) enum TLSHandshakePayload {
    HelloRequest(Hello),
    ClientHello(Client),
    ServerHello(Server),
    Certificate(Cert),
    ServerKeyExchange(SerEx),
    CertificateRequest(CertReq),
    ServerHelloDone(Done),
    CertificateVerify(CertVeri),
    ClientKeyExchange(CliEx),
    Finished(Fine),
}

struct Hello {
    
}

pub(super) struct Client {
    pub(super) version: TLSVersion,
    pub(super) random: [u8;32],
    pub(super) session: Option<Vec<u8>>,
    pub(super) ciphers: Vec<Cipher>,
    pub(super) compresion: Vec<Compression>,
}

struct Server {

}

struct Cert {

}

struct SerEx {

}

struct CertReq {

}

struct Done {

}

struct CertVeri {

}

struct CliEx {

}

struct Fine {

}