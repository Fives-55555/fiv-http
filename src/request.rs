use crate::{
    ferrors::HttpRequestErr,
    http::{
        account::SessionId,
        method::Method,
        server::Client,
        uri::Uri,
        utils::{
            cookie_parser, get_by_key, AllowedExtentions, HTTPExtentions, HTTPHeader, Version,
        },
    },
    traits::New,
};

pub struct HTTPRequest {
    pub parts: RequestHeader,
    pub body: Vec<u8>,
}

impl HTTPRequest {
    pub fn from_string(&mut self, src: &String, res: &Client) -> Option<HttpRequestErr> {
        match src.len() {
            14.. => {
                {
                    let mut lines = src.lines();
                    let mut header = match lines.next() {
                        Some(h) => h.split_ascii_whitespace(),
                        None => return Some(HttpRequestErr::new()),
                    };
                    self.parts.method = match header.next() {
                        Some(m) => Method::from_str(m),
                        None => return Some(HttpRequestErr::new()),
                    };
                    match header.next() {
                        Some(u) => self.parts.uri.from_string(u.to_string()),
                        None => return Some(HttpRequestErr::new()),
                    };
                    self.parts.version = match header.next() {
                        Some(v) => match Version::from_string(v.to_string()) {
                            Ok(v) => v,
                            Err(_) => return Some(HttpRequestErr::new()),
                        },
                        None => return Some(HttpRequestErr::new()),
                    };
                    self.parts.headcont = lines
                        .filter_map(|line| {
                            let (key, values) = match line.split_once(':') {
                                Some(s) => s,
                                None => return None,
                            };
                            Some(HTTPHeader {
                                key: HTTPHeader::key_parse(key),
                                value: values.trim().to_string(),
                            })
                        })
                        .collect::<HTTPExtentions>();
                }
                for header in self.parts.headcont.iter() {
                    if header.key == AllowedExtentions::Cookie {
                        let r = get_by_key(cookie_parser(header.value.clone()), "sessionId");
                        if r.is_ok() {                            
                            self.parts.account = SessionId::from_id(&res.sessions, r.unwrap().1);
                        }
                    }
                }
            }
            _ => return Some(HttpRequestErr::new()),
        }
        None
    }
}

impl Default for HTTPRequest {
    fn default() -> Self {
        Self {
            parts: RequestHeader::new(),
            body: Vec::new(),
        }
    }
}

impl New for HTTPRequest {}

pub struct RequestHeader {
    pub method: Method,           //Which Method
    pub uri: Uri,                 //URI
    pub version: Version,         //Version
    pub headcont: HTTPExtentions, //Headers
    pub account: Option<u16>,     //ID
}

impl New for RequestHeader {}

impl Default for RequestHeader {
    fn default() -> Self {
        Self {
            method: Method::GET,
            uri: Uri::new(),
            version: Version::new(),
            headcont: Vec::new(),
            account: None,
        }
    }
}
