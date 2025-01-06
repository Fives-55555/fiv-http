use super::{
    handshake::{Client, TLSHandshakePayload, TLSHandshakeType},
    TLSPayload, TLSProtocol, TLSVersion, MAX_VERSION,
};
use crate::server::ServerError;

pub fn to_payload(kind: TLSProtocol, buf: &[u8]) -> Result<TLSPayload, ServerError> {
    match kind {
        TLSProtocol::Handshake => to_handshake(buf),
        TLSProtocol::Alert => to_alert(buf),
        TLSProtocol::Data => to_data(buf),
        TLSProtocol::CipherSpec => to_change(buf),
    }
}
//buf must be greater then 0
fn to_handshake(buf: &[u8]) -> Result<TLSPayload, ServerError> {
    let kind: TLSHandshakeType = TLSHandshakeType::cast(buf[0])?;
    let length: usize = u32::from_be_bytes([0, buf[1], buf[2], buf[3]]) as usize;
    if buf.len() - 4 != length {
        return Err(ServerError::INVPAR);
    };
    let mut buf: &[u8] = &buf[4..];
    let inner = match kind {
        TLSHandshakeType::ClientHello => {
            let max_version: TLSVersion = match TLSVersion::cast(buf[0], buf[1]) {
                Ok(suppored) => suppored,
                Err(err) => match err {
                    ServerError::VERHIGH => MAX_VERSION,
                    ServerError::VERLOW => {
                        return Err(ServerError::VERLOW);
                    }
                },
            };
            let client_random: &[u8] = &buf[2..34];
            let session_id_len: usize = buf[34] as usize;
            let mut index: usize = 35 + session_id_len;
            let sessino_id: Option<&[u8]> = if session_id_len != 0 {
                Some(&buf[35..index])
            } else {
                None
            };
            let cipher_len: usize = u16::from_be_bytes([buf[index], buf[index + 2]]) as usize;
            index += 2;
            let cipher_list: Option<&[u8]> = if session_id_len != 0 {
                Some(&buf[index..index + cipher_len])
            } else {
                None
            };
            index += cipher_len;
            let compression_len: usize = buf[index] as usize;
            index += 1;
            let compression: Vec<Compression> = if session_id_len != 0 {
                Some(&buf[index..index + cipher_len])
            } else {
                None
            };
            index += cipher_len;

            TLSHandshakePayload::ClientHello(Client {
                version: max_version,
                random: client_random,
                session: sessino_id,
                ciphers: cipher_list,
                compresion: compression,
            })
        }
        _ => todo!(),
    };
    return Ok(TLSPayload::Handshake(inner));
}

fn to_alert(buf: &[u8]) -> Result<TLSPayload, ServerError> {
    Err(ServerError::CONERR)
}

fn to_data(buf: &[u8]) -> Result<TLSPayload, ServerError> {
    Err(ServerError::CONERR)
}

fn to_change(buf: &[u8]) -> Result<TLSPayload, ServerError> {
    Err(ServerError::CONERR)
}
