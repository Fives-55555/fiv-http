use windows::Win32::Foundation::HANDLE;

macro_rules! io_err {
    ($err:expr) => {
        Err(std::io::Error::from_raw_os_error($err.code().0))
    };
}

pub mod net;
pub mod iocp;
pub mod overlapped;

pub struct FutAsyncRead(pub HANDLE);
