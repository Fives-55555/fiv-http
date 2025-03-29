use windows::Win32::Foundation::HANDLE;

macro_rules! io_err {
    ($err:expr) => {
        Err(std::io::Error::from_raw_os_error($err.code().0))
    };
}

// Good for long living or multi packet connections
pub mod rio;
pub mod net;
pub mod iocp;
// Good for short living and easy stuff
pub mod overlapped;

pub struct FutAsyncRead(pub HANDLE);

#[test]
fn test()->std::io::Result<()> {
    use iocp::IOCP;
    use std::net::UdpSocket;
    use rio::{init, RIOBuffer, CompletionQueue, RequestQueue};
    init();
    let buf = RIOBuffer::new()?;
    drop(buf);
    println!("HI");//Run to here
    return Ok(())
}