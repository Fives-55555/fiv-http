use crate::win::socket::FivSocket;

use super::Overlapped;

pub struct OverlappedStream {
    socket: FivSocket,
    overlapped: Overlapped,
}
