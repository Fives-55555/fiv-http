use crate::traits::New;
use std::{default::Default, fmt::Display, vec::Vec};

pub struct UserInfo {
    pub username: Username,
    pub password: Option<Password>,
}

pub struct Authority {
    pub creds: Option<UserInfo>,
    pub host: Host,
    pub port: Port,
}

pub struct Uri {
    pub scheme: Scheme,
    pub authority: Authority,
    pub path: Path,
    pub query: Querys,
    pub fragment: Fragment,
}

pub struct Query {
    key: String,
    value: String,
}

pub type Port = u16;

pub type Host = String;

pub type Password = String;

pub type Username = String;

pub type Path = String;

pub type Querys = Vec<Query>;

pub type Fragment = String;

pub enum Scheme {
    HTTP,
    HTTPS,
    Other(String),
    Unknown,
}

// impl Uri {
//
//     pub fn from_string(&mut self, mut src: String) {
//         //Does not check for char correctness
//         //Split Anchor
//         let mut srt = String::new();
//         match src.split_once("://") {
//             Some((scheme, span)) => {
//                 self.scheme = scheme.from_str();
//                 self.span = span.to_string();
//             }
//             None => {
//                 self.span = String::new();
//             }
//         };
//         src = srt.clone();
//         //Split Query
//         match src.split_once('?') {
//             Some((str, query)) => {
//                 srt = str.to_string();
//                 self.query = query
//                     .split('&')
//                     .filter_map(|query| match query.split_once('=') {
//                         Some((key, value)) => Some((key.to_string(), value.to_string())),
//                         None => return None,
//                     })
//                     .collect::<Vec<(String, String)>>();
//             }
//             None => {
//                 self.query = Vec::new();
//             }
//         };
//         src = srt.clone();
//         //Split the Scheme
//         match src.split_once("://") {
//             Some((scheme, str)) => {
//                 self.scheme = scheme.to_string();
//                 srt = str.to_string();
//             }
//             None => {
//                 self.scheme = String::new();
//             }
//         };
//         src = srt.clone();
//         //Split Port and Path from Auth
//         match src.split_once(':') {
//             Some((authority, portupath)) => {
//                 self.authority = authority.to_string();
//                 //Split Port from Path
//                 match portupath.split_once('/') {
//                     Some((port, path)) => {
//                         match port.parse() {
//                             Ok(port) => {
//                                 self.port = port;
//                             }
//                             Err(_) => {
//                                 self.port = Port::default();
//                             }
//                         };
//                         self.path = path.to_string();
//                     }
//                     None => {
//                         self.port = Port::default();
//                         self.path = String::new();
//                     }
//                 }
//             }
//             None => {
//                 //Default Http Port
//                 self.port = 80;
//                 match src.split_once('/') {
//                     Some((domain, path)) => {
//                         self.authority = domain.to_string();
//                         self.path = path.to_string();
//                     }
//                     None => {
//                         self.path = String::new();
//                         self.authority = src.to_string();
//                     }
//                 }
//             }
//         };
//     }
// }

impl Default for Uri {
    fn default() -> Self {
        Uri {
            scheme: Scheme::new(),
            authority: Authority::new(),
            path: Path::new(),
            query: Querys::new(),
            fragment: Fragment::new(),
        }
    }
}

impl Scheme {
    fn from_str(src: &str) -> Scheme {
        match src {
            "http" => Scheme::HTTP,
            "https" => Scheme::HTTPS,
            "" => Scheme::Unknown,
            _ => Scheme::Other(src.to_string()),
        }
    }
    fn as_str(&self) -> &str {
        match self {
            Scheme::HTTP => "HTTP",
            Scheme::HTTPS => "HTTPS",
            Scheme::Unknown => "Unknown",
            Scheme::Other(e) => e,
        }
    }
}

impl Default for Scheme {
    fn default() -> Self {
        Scheme::Unknown
    }
}

impl Default for Authority {
    fn default() -> Self {
        Self {
            creds: None,
            host: Host::new(),
            port: 80,
        }
    }
}

impl Default for UserInfo {
    fn default() -> Self {
        Self {
            username: Username::new(),
            password: None,
        }
    }
}

impl New for Uri {}

impl New for Authority {}

impl New for Port {}

impl New for Scheme {}

impl Uri {
    #[rustfmt::skip]
    pub const VALIDCHARS: [u8; 256] = [
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,  b'!',     0,  b'#',  b'$',     0,  b'&', b'\'',
     b'(',  b')',  b'*',  b'+',  b',',  b'-',  b'.',  b'/',  b'0',  b'1',
     b'2',  b'3',  b'4',  b'5',  b'6',  b'7',  b'8',  b'9',  b':',  b';',
        0,  b'=',     0,  b'?',  b'@',  b'A',  b'B',  b'C',  b'D',  b'E',
     b'F',  b'G',  b'H',  b'I',  b'J',  b'K',  b'L',  b'M',  b'N',  b'O',
     b'P',  b'Q',  b'R',  b'S',  b'T',  b'U',  b'V',  b'W',  b'X',  b'Y',
     b'Z',  b'[',     0,  b']',     0,  b'_',     0,  b'a',  b'b',  b'c',
     b'd',  b'e',  b'f',  b'g',  b'h',  b'i',  b'j',  b'k',  b'l',  b'm',
     b'n',  b'o',  b'p',  b'q',  b'r',  b's',  b't',  b'u',  b'v',  b'w',
     b'x',  b'y',  b'z',     0,     0,     0,  b'~',     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
        0,     0,     0,     0,     0,     0
    ];
    pub fn from_string(&mut self, mut src: String) {
        if let Some((scheme, str)) = src.split_once("://") {
            self.scheme = Scheme::from_str(scheme);
            src = str.to_string();
        } else {
            self.scheme = Scheme::Unknown
        };
        if let Some((authority, exe)) = src.split_once('/') {
            let mut auth = authority.to_string().clone();
            if let Some((userinfo, hostport)) = auth.split_once('@') {
                let auth_without_port = hostport.to_string();
                self.authority.creds = match userinfo.split_once(':') {
                    Some((username, password)) => Some(UserInfo {
                        username: username.to_string(),
                        password: Some(password.to_string()),
                    }),
                    None => Some(UserInfo {
                        username: userinfo.to_string(),
                        password: None,
                    }),
                };
                auth = auth_without_port;
            } else {
                self.authority.creds = None
            };
            if let Some((host, port)) = auth.split_once(':') {
                self.authority.port = match port.parse::<u16>() {
                    Ok(port) => port,
                    Err(_) => Port::default(),
                };
                self.authority.host = host.to_string();
            } else {
                self.authority.port = Port::default();
                self.authority.host = auth
            };
            src = exe.to_string().clone();
        } else {
            self.authority = Authority::new()
        };

        if let Some((str, fragment)) = src.split_once('#') {
            self.fragment = fragment.to_string();
            src = str.to_string();
        } else {
            self.fragment = String::new()
        };

        if let Some((path, query)) = src.split_once('?') {
            self.path = path.to_string();
            self.query = query
                .split('&')
                .filter_map(|query| match query.split_once('=') {
                    Some((key, value)) => Some(Query {
                        key: key.to_string(),
                        value: value.to_string(),
                    }),
                    None => None,
                })
                .collect::<Querys>()
        } else {
            self.path = src
        };
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let userinfo = match &self.authority.creds {
            Some(e) => {
                format!("{}", e)
            }
            None => "Contains no Userinfo".to_string(),
        };
        write!(
            f,
            "Protokoll: {}\n{}\nHost: {}\nPort: {}\nPath: {}\nQuery: {}\nFragment: {}",
            self.scheme,
            userinfo,
            self.authority.host,
            self.authority.port,
            self.path,
            self.query
                .iter()
                .map(|query| { format!("{query}") })
                .collect::<String>(),
            self.fragment
        )
    }
}

impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for UserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self.password {
            Some(ref p) => p,
            None => "None",
        };
        write!(f, "    Username: {}\n    Password: {}", self.username, str)
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}: {}]", self.key, self.value)
    }
}
//At char at Email
