use windows::Win32::Foundation::HANDLE;

macro_rules! io_err {
    ($err:expr) => {
        Err(std::io::Error::from_raw_os_error($err.code().0))
    };
}

// Good for long living or multi packet connections
pub mod iocp;
pub mod rio;
// Good for short living and easy stuff
pub mod overlapped;

pub struct FutAsyncRead(pub HANDLE);

#[test]
fn test() -> std::io::Result<()> {
    use rio::{RegisteredTcpStream, RIOBuffer, init};
    init();
    let mut buf = RIOBuffer::new()?;

    let slice = buf.get_whole().unwrap();

    let mut sock = RegisteredTcpStream::connect("127.0.0.1:8080").unwrap();

    sock.read(slice)?;

    let x = sock.await_read_and_get()?;

    println!("{}", x.0);

    println!("{:?}", x.1.as_slice());

    println!("HI"); //Run to here
    
    return Ok(());
}
