macro_rules! io_err_res {
    ($err:expr) => {
        Err(std::io::Error::from_raw_os_error($err.code().0))
    };
}

pub mod completion;

pub mod rio;
// Good for short living and easy stuff
pub mod overlapped;

pub mod socket;

mod basic;

pub const LISTEN_BACKLOG: i32 = 128;

#[test]
fn test() -> std::io::Result<()> {
    use rio::{RIOBuffer, RegisteredTcpStream, init};
    init();
    let mut buf = RIOBuffer::new()?;

    let slice = buf.alloc_whole().unwrap();

    let mut sock = RegisteredTcpStream::connect("127.0.0.1:8080").unwrap();

    sock.add_read(slice)?;

    let x = sock.await_read_and_get()?;

    println!("{}", x.len());

    println!("{:?}", x.as_slice());

    // println!("HI"); //Run to here

    // sock.add_read(slice)?;

    // let x = sock.await_read_and_get()?;

    // println!("{}", x.len());

    // println!("{:?}", x.as_slice());

    return Ok(());
}
