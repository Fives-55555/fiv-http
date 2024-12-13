#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Method(MV);


#[derive(Clone, PartialEq, Eq, Hash)]
enum MV {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Other(String)
}


impl Method {
    pub const GET: Method = Method(MV::Get);
    pub const HEAD: Method = Method(MV::Head);
    pub const POST: Method = Method(MV::Post);
    pub const PUT: Method = Method(MV::Put);
    pub const DELETE: Method = Method(MV::Delete);
    pub const CONNECT: Method = Method(MV::Connect);
    pub const OPTIONS: Method = Method(MV::Options);
    pub const TRACE: Method = Method(MV::Trace);

    pub fn as_str(&self) -> &str {
        match &self.0 {
            MV::Get => "GET",
            MV::Head => "HEAD",
            MV::Post => "POST",
            MV::Put => "PUT",
            MV::Delete => "DELETE",
            MV::Connect => "CONNECT",
            MV::Options => "OPTIONS",
            MV::Trace => "TRACE",
            MV::Other(str)=>str
        }
    }

    pub fn from_str(src: &str) -> Method {
        match src {
            "GET" => Method(MV::Get),
            "HEAD" => Method(MV::Head),
            "POST" => Method(MV::Post),
            "PUT" => Method(MV::Put),
            "DELETE" => Method(MV::Delete),
            "CONNECT" => Method(MV::Connect),
            "OPTIONS" => Method(MV::Options),
            "TRACE" => Method(MV::Trace),
            _ => Method(MV::Other(src.to_string())),
        }
    }
}