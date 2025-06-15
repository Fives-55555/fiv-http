use std::rc::Rc;

use windows::Win32::System::IO::OVERLAPPED;

mod listener;
mod stream;

#[derive(Clone)]
#[cfg(feature = "multithreaded")]
pub struct Overlapped(Arc<OVERLAPPED>);

#[cfg(not(feature = "multithreaded"))]
#[derive(Clone)]
pub struct Overlapped(Rc<OVERLAPPED>);

impl Overlapped {
    pub fn new() -> Overlapped {
        Overlapped(Rc::new(OVERLAPPED::default()))
    }
    pub fn inner(&self) -> &OVERLAPPED {
        &self.0
    }
    #[cfg(not(feature = "multithreaded"))]
    pub fn inner_rc(&self) -> &Rc<OVERLAPPED> {
        &self.0
    }
}
