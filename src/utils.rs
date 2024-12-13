use crate::{
    ferrors::VersionErr, traits::{New, WS}
};

#[cfg(feature = "log_missing_extention")]
use crate::log::{log, ERROR};


pub struct ContLength(pub u64);

pub const SERVERS: &'static str = "FivServ/2.0.0";

impl ContLength {
    pub fn as_str(&self) -> String {
        self.0.clone().to_string()
    }
}

#[derive(Clone)]
pub struct ContType(Formattype);

#[derive(Clone)]
enum Formattype {
    Plain,
    JSON,
    Xml,
    JS,
    HTML,
    CSS,
    Image(Image),
    Video
}

#[derive(Clone)]
enum Image {
    Pdf,
    Jpeg,
    Png,
}

pub const HTML: ContType = ContType(Formattype::HTML);
pub const CSS: ContType = ContType(Formattype::CSS);
pub const JAVASCRIPT: ContType = ContType(Formattype::JS);
pub const PLAIN: ContType = ContType(Formattype::Plain);
pub const JSON: ContType = ContType(Formattype::JSON);
pub const XML: ContType = ContType(Formattype::Xml);
pub const PDF: ContType = ContType(Formattype::Image(Image::Pdf));
pub const JPEG: ContType = ContType(Formattype::Image(Image::Jpeg));
pub const PNG: ContType = ContType(Formattype::Image(Image::Png));

impl ContType { 
    pub fn as_str(&self) -> &str {
        match self.0 {
            Formattype::HTML => "text/html",
            Formattype::CSS => "text/css",
            Formattype::JS => "application/javascript",
            Formattype::Plain => "text/plain",
            Formattype::JSON => "application/json",
            Formattype::Xml => "application/xml",
            Formattype::Image(Image::Pdf) => "application/pdf",
            Formattype::Image(Image::Jpeg) => "image/jpeg",
            Formattype::Image(Image::Png) => "image/png",
            Formattype::Video => "video/mp4"
        }
    }

    pub fn from_str(path: &str)->ContType {
        ContType(match path {
            "html"=>Formattype::HTML,
            "css"=>Formattype::CSS,
            "js"=>Formattype::JS,
            "txt"=>Formattype::Plain,
            "json"=>Formattype::JSON,
            "xml"=>Formattype::Xml,
            "pdf"=>Formattype::Image(Image::Pdf),
            "jpeg"=>Formattype::Image(Image::Jpeg),
            "png"=>Formattype::Image(Image::Png),
            "mp4"=>Formattype::Video,
            _=>Formattype::Plain
        })
    }
}


#[derive(Clone, Copy)]
pub struct Version(VV);


#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
enum VV {
    V0_9,
    V1_0,
    V1_1,
    V2_0,
    V3_0,
}


impl Version {
    pub const HTTP0_9: Version = Version(VV::V0_9);
    pub const HTTP1_0: Version = Version(VV::V1_0);
    pub const HTTP1_1: Version = Version(VV::V1_1);
    pub const HTTP2_0: Version = Version(VV::V2_0);
    pub const HTTP3_0: Version = Version(VV::V3_0);

    pub fn to_string(&self) -> String {
        match self.0 {
            VV::V0_9 => "HTTP/0.9".to_string(),
            VV::V1_0 => "HTTP/1.0".to_string(),
            VV::V1_1 => "HTTP/1.1".to_string(),
            VV::V2_0 => "HTTP/2.0".to_string(),
            VV::V3_0 => "HTTP/3.0".to_string(),
        }
    }

    pub fn from_string(src: String) -> Result<Version, VersionErr> {
        match src.as_str() {
            "HTTP/0.9" => Ok(Version::HTTP0_9),
            "HTTP/1.0" => Ok(Version::HTTP1_0),
            "HTTP/1.1" => Ok(Version::HTTP1_1),
            "HTTP/2.0" => Ok(Version::HTTP2_0),
            "HTTP/3.0" => Ok(Version::HTTP3_0),
            _ => Err(VersionErr::new()),
        }
    }
}


pub fn cookie_parser(str: String)->Vec<(String,String)> {
    let mut v = Vec::new();
    let mut str = str;
    str.trima();
    for src in str.split(';') {
        match src.split_once('=') {
            Some((key, value))=>v.push((key.to_string(), value.to_string())),
            None=>continue
        }
    }
    v
}

pub fn get_by_key(v: Vec<(String, String)>, s: &str)->Result<(String, String), ()> {
    for e in v.iter() {
        if e.0 == s {
            return Ok(e.clone());
        }
    }
    Err(())
}

pub type HTTPExtentions = Vec<HTTPHeader>;

#[derive(Clone)]
pub struct HTTPHeader {
    pub key: AllowedExtentions,
    pub value: String,
}


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AllowedExtentions {
    Unknown(String),
    SetCookie,
    Cookie,
    Lang,
}

impl HTTPHeader {
    pub fn key_parse(key: &str)->AllowedExtentions {
        match key {
            "Lang"=>AllowedExtentions::Lang,
            "Set-Cookie"=>AllowedExtentions::SetCookie,
            "Cookie"=>AllowedExtentions::Cookie,
            _=>{
                #[cfg(feature = "log_missing_extention")]
                log(ERROR, format!("Unknown Extention: \"{}\"", key));///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
                AllowedExtentions::Unknown(key.to_string())
            }
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::HTTP1_1
    }
}

impl New for Version {}