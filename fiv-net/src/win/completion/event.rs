use windows::{
    Win32::{Foundation::HANDLE, Security::SECURITY_ATTRIBUTES, System::Threading::CreateEventW},
    core::PCWSTR,
};

pub struct Event {
    handle: HANDLE,
    reset: Reset,
}

impl Event {
    pub fn new(init_state: bool, reset: Reset) -> std::io::Result<Event> {
        Event::new_raw(init_state, reset, None, None)
    }
    pub fn new_raw(
        init_state: bool,
        reset: Reset,
        security: Option<Security>,
        name: Option<&str>,
    ) -> std::io::Result<Event> {
        let sec: Option<SECURITY_ATTRIBUTES> = security.and_then(Security::as_sec);
        let secref = match sec.as_ref() {
            Some(x) => Some(x as *const SECURITY_ATTRIBUTES),
            None => None,
        };
        let utf16: Option<Vec<u16>> = match name {
            Some(str) => {
                let mut vec: Vec<u16> = str.encode_utf16().collect();
                vec.push(0);
                Some(vec)
            }
            None => None,
        };
        let winname = match utf16 {
            Some(utf) => PCWSTR::from_raw(utf.as_ptr()),
            None => PCWSTR::null(),
        };
        let handle =
            unsafe { io_err_res!(CreateEventW(secref, reset.as_bool(), init_state, winname))? };
        Ok(Event {
            handle: handle,
            reset: reset,
        })
    }
}

pub enum Reset {
    Automatic = 0,
    Manual = 1,
}

impl Reset {
    fn as_bool(&self) -> bool {
        match self {
            Reset::Automatic => false,
            Reset::Manual => true,
        }
    }
}

pub struct Security {}

impl Security {
    fn as_sec(self) -> Option<SECURITY_ATTRIBUTES> {
        Some(SECURITY_ATTRIBUTES {
            nLength: 0,
            lpSecurityDescriptor: std::ptr::null_mut(),
            bInheritHandle: false.into(),
        })
    }
}
