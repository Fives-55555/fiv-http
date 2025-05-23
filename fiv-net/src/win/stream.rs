use super::{basic::BasicStream, socket::FivSocket};


pub struct FivStream<T=BasicStream>
    where T: WinStream
{
    socket: FivSocket,
    sock_store: T,
}

pub trait WinStream {}

impl WinStream for BasicStream {}

pub trait TryFromSock {
    fn try_from.
}

impl WinStream for BasicStream {}

impl TryFrom<FivSocket> for FivStream<BasicStream> {
    type Error = NetConvError;
    fn try_from(value: FivSocket) -> Result<Self, Self::Error> {
        if !value.is_bound() {
            Ok(
            
            ) 
        }
        return Err(NetConvError::IsBound)
    }
}

pub enum NetConvError {
    IsBound,
    NotCapable
}