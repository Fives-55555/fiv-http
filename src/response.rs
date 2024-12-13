use crate::{date::Date, http::{request::HTTPRequest, utils::{ContType, Version, PLAIN, SERVERS}}, traits::New, traits::LogUnwrap};

use super::server::Client;


pub struct HTTPResponse {
    pub rsheader: ResponseHeader,
    pub body: Vec<u8>,
}


impl HTTPResponse {
    pub fn ct(mut self, ct: ContType) -> HTTPResponse {
        self.rsheader.fields.push((String::from("Content-Type:"), ct.as_str().to_string()));
        self
    }
    pub fn version(mut self, ver: Version) -> HTTPResponse {
        self.rsheader.version = ver;
        self
    }
    pub fn status_code(mut self, ver: StatusCode) -> HTTPResponse {
        self.rsheader.status_code = ver;
        self
    }
    pub fn body(mut self, ver: Vec<u8>) -> HTTPResponse {
        self.body = ver;
        self
    }
    pub fn gen_len(mut self) -> HTTPResponse {
        self.rsheader.fields.push((String::from("Content-Length"), self.body.len().to_string()));
        self
    }
    pub fn build<'a>(mut self) -> Vec<u8> {
        let mut str: Vec<u8> = format!(
            "{} {}\r\n{}\r\n\r\n",
            self.rsheader.version.to_string(),
            self.rsheader.status_code.as_str(),
            self.rsheader.fields.iter().map(|header| format!("{}: {}", header.0, header.1)).collect::<Vec<String>>().join("\r\n")
        )
        .into_bytes();
        str.extend_from_slice(&mut self.body);
        str
    }
    pub fn wrong_method(req: &HTTPRequest)->Vec<u8> {
        HTTPResponse::new()
            .version(req.parts.version)
            .status_code(StatusCode::METHODNOTALLOWED)
            .ct(PLAIN)
            .body("405 Method Not Allowed".to_string().into_bytes())
            .gen_len().build()
    }
    pub fn misdirreq(req: &HTTPRequest)->Vec<u8> {
        let response = HTTPResponse::new()
            .version(req.parts.version)
            .status_code(StatusCode::MISDIRECTEDREQUEST)
            .ct(PLAIN)
            .body("421 Misdirected Request: Wrong Uri".to_string().into_bytes())
            .gen_len();
        response.build()
    }
    pub fn get_run(req: &HTTPRequest, res: &Client, auth: &str)-> Vec<u8> {
        if req.parts.uri.authority.host == auth || req.parts.uri.authority.host == "" {
            match res.paths.binary_search(&req.parts.uri.path.clone().into_boxed_str()) { //////////////////////////////////// Könnte Probleme machen wenn Path und Sites nicht gleich sind
                Ok(index) => {
                    let site = res.sites.iter().nth(index).unwrap_log();
                    HTTPResponse::new()
                        .status_code(StatusCode::OK)
                        .version(req.parts.version)
                        .body(site.get_page(req, res))
                        .ct(site.file_type.clone())
                        .gen_len()
                        .build()
                }
                Err(_) => {
                    return HTTPResponse::misdirreq(req)
                }
            }
        } else {
            return HTTPResponse::misdirreq(req)
        }
    }
    pub fn post_run(req: &HTTPRequest, res: &Client, auth: &str)-> Vec<u8> {
        if req.parts.uri.authority.host == auth || req.parts.uri.authority.host == "" {
            match res.api_paths.binary_search(&req.parts.uri.path.clone().into_boxed_str()) { //////////////////////////////////// Könnte Probleme machen wenn Path und Api nicht gleich sind
                Ok(index) => {
                    let api = res.api.iter().nth(index).unwrap_log();
                    HTTPResponse::new()
                        .status_code(StatusCode::OK)
                        .version(req.parts.version)
                        .body(api.get_resp(req, res))
                        .ct(api.filetype.clone())
                        .gen_len()
                        .build()
                }
                Err(_) => {
                    return HTTPResponse::misdirreq(req)
                }
            }
        } else {
            return HTTPResponse::misdirreq(req)
        }
    }
}

impl Default for HTTPResponse {
    fn default() -> Self {
        HTTPResponse {
            rsheader: ResponseHeader {
                version: Version::HTTP1_1,
                status_code: StatusCode::INTERNALSERVERERROR,
                date: Date::new().now().to_http_string(),
                server: SERVERS,
                fields: Vec::new()
            },
            body: Vec::new(),
        }
    }
}

impl New for HTTPResponse{}

pub struct ResponseHeader {
    pub version: Version,
    pub status_code: StatusCode,
    pub date: String,
    pub server: &'static str,
    pub fields: Vec<(String,String)>
}

impl ResponseHeader {
    pub fn as_str(&self) -> String {
        let r: String = format!(
            "{} {}\r\n{}\r\nServer: {}\r\n{}\r\n",
            self.version.clone().to_string(),
            self.status_code.as_str(),
            self.date,
            self.server,
            self.fields.iter().map(|header|format!("{}: {}", header.0, header.1)).collect::<Vec<String>>().join("\r\n")
        );
        r
    }
}


pub struct StatusCode(Codes);

enum Codes {
    SC100,
    SC101,
    SC200,
    SC201,
    SC202,
    SC203,
    SC204,
    SC205,
    SC206,
    SC300,
    SC301,
    SC302,
    SC303,
    SC304,
    SC307,
    SC308,
    SC400,
    SC401,
    SC402,
    SC403,
    SC404,
    SC405,
    SC406,
    SC407,
    SC408,
    SC409,
    SC410,
    SC411,
    SC412,
    SC413,
    SC414,
    SC415,
    SC416,
    SC417,
    SC418,
    SC421,
    SC422,
    SC423,
    SC424,
    SC425,
    SC426,
    SC428,
    SC429,
    SC431,
    SC451,
    SC500,
    SC501,
    SC502,
    SC503,
    SC504,
    SC505,
    SC506,
    SC507,
    SC508,
    SC510,
    SC511,
}

impl StatusCode {
    pub const CONTINUE: StatusCode = StatusCode(Codes::SC100);
    pub const SWITCHINGPROTOCOLS: StatusCode = StatusCode(Codes::SC101);
    pub const OK: StatusCode = StatusCode(Codes::SC200);
    pub const CREATED: StatusCode = StatusCode(Codes::SC201);
    pub const ACCEPTED: StatusCode = StatusCode(Codes::SC202);
    pub const NONAUTHORITATIVEINFO: StatusCode = StatusCode(Codes::SC203);
    pub const NOCONTENT: StatusCode = StatusCode(Codes::SC204);
    pub const RESETCONTENT: StatusCode = StatusCode(Codes::SC205);
    pub const PARTIALCONTENT: StatusCode = StatusCode(Codes::SC206);
    pub const MULTIPLECHOICES: StatusCode = StatusCode(Codes::SC300);
    pub const MOVEDPERMANENTLY: StatusCode = StatusCode(Codes::SC301);
    pub const FOUND: StatusCode = StatusCode(Codes::SC302);
    pub const SEEOTHER: StatusCode = StatusCode(Codes::SC303);
    pub const NOTMODIFIED: StatusCode = StatusCode(Codes::SC304);
    pub const TEMPORARYREDIRECT: StatusCode = StatusCode(Codes::SC307);
    pub const PERMANENTREDIRECT: StatusCode = StatusCode(Codes::SC308);
    pub const BADREQUEST: StatusCode = StatusCode(Codes::SC400);
    pub const UNAUTHORIZED: StatusCode = StatusCode(Codes::SC401);
    pub const PAYMENTREQUIRED: StatusCode = StatusCode(Codes::SC402);
    pub const FORBIDDEN: StatusCode = StatusCode(Codes::SC403);
    pub const NOTFOUND: StatusCode = StatusCode(Codes::SC404);
    pub const METHODNOTALLOWED: StatusCode = StatusCode(Codes::SC405);
    pub const NOTACCEPTABLE: StatusCode = StatusCode(Codes::SC406);
    pub const PROXYAUTHENTICATIONREQUIRED: StatusCode = StatusCode(Codes::SC407);
    pub const REQUESTTIMEOUT: StatusCode = StatusCode(Codes::SC408);
    pub const CONFLICT: StatusCode = StatusCode(Codes::SC409);
    pub const GONE: StatusCode = StatusCode(Codes::SC410);
    pub const LENGHREQUIRED: StatusCode = StatusCode(Codes::SC411);
    pub const PRECONDITIONFAILED: StatusCode = StatusCode(Codes::SC412);
    pub const PAYLOADTOOLARGE: StatusCode = StatusCode(Codes::SC413);
    pub const URITOOLONG: StatusCode = StatusCode(Codes::SC414);
    pub const UNSUPPORTEDMEDIATYPE: StatusCode = StatusCode(Codes::SC415);
    pub const RANGENOTSATISFIABLE: StatusCode = StatusCode(Codes::SC416);
    pub const EXPECTATIONFAILED: StatusCode = StatusCode(Codes::SC417);
    pub const IAMATEAPOT: StatusCode = StatusCode(Codes::SC418);
    pub const MISDIRECTEDREQUEST: StatusCode = StatusCode(Codes::SC421);
    pub const UNPROCESSABLEENTITY: StatusCode = StatusCode(Codes::SC422);
    pub const LOCKED: StatusCode = StatusCode(Codes::SC423);
    pub const FAILEDDEPENDENCY: StatusCode = StatusCode(Codes::SC424);
    pub const TOOEARLY: StatusCode = StatusCode(Codes::SC425);
    pub const UPGRADEREQUIRED: StatusCode = StatusCode(Codes::SC426);
    pub const PRECONDITIONREQUIRED: StatusCode = StatusCode(Codes::SC428);
    pub const TOOMANYREQUEST: StatusCode = StatusCode(Codes::SC429);
    pub const REQUESTHEADERFIELDTOOLARGE: StatusCode = StatusCode(Codes::SC431);
    pub const UNAVAILABLEFORLEGALREASONS: StatusCode = StatusCode(Codes::SC451);
    pub const INTERNALSERVERERROR: StatusCode = StatusCode(Codes::SC500);
    pub const NOTIMPLEMENTED: StatusCode = StatusCode(Codes::SC501);
    pub const BADGATEWAY: StatusCode = StatusCode(Codes::SC502);
    pub const SERVICEUNAVAILABLE: StatusCode = StatusCode(Codes::SC503);
    pub const GATEWAYTIMEOUT: StatusCode = StatusCode(Codes::SC504);
    pub const HTTPVERSIONNOTSUPPORTED: StatusCode = StatusCode(Codes::SC505);
    pub const VARIANTALSONEGOTIATES: StatusCode = StatusCode(Codes::SC506);
    pub const INSUFFICIENTSTORAGE: StatusCode = StatusCode(Codes::SC507);
    pub const LOOPDETECTED: StatusCode = StatusCode(Codes::SC508);
    pub const NOTEXTENDED: StatusCode = StatusCode(Codes::SC510);
    pub const NETWORKAUTHENTICATIONREQUIRED: StatusCode = StatusCode(Codes::SC511);

    pub fn as_str(&self) -> &str {
        match self.0 {
            Codes::SC100 => "100 Continue",
            Codes::SC101 => "101 Switching Protocols",
            Codes::SC200 => "200 OK",
            Codes::SC201 => "201 Created",
            Codes::SC202 => "202 Accepted",
            Codes::SC203 => "203 Non-Authoritative Information",
            Codes::SC204 => "204 No Content",
            Codes::SC205 => "205 Reset Content",
            Codes::SC206 => "206 Partial Content",
            Codes::SC300 => "300 Multiple Choices",
            Codes::SC301 => "301 Moved Permanently",
            Codes::SC302 => "302 Found",
            Codes::SC303 => "303 See Other",
            Codes::SC304 => "304 Not Modified",
            Codes::SC307 => "307 Temporary Redirect",
            Codes::SC308 => "308 Permanent Redirect",
            Codes::SC400 => "400 Bad Request",
            Codes::SC401 => "401 Unauthorized",
            Codes::SC402 => "402 Payment Required",
            Codes::SC403 => "403 Forbidden",
            Codes::SC404 => "404 Not Found",
            Codes::SC405 => "405 Method Not Allowed",
            Codes::SC406 => "406 Not Acceptable",
            Codes::SC407 => "407 Proxy Authentication Required",
            Codes::SC408 => "408 Request Timeout",
            Codes::SC409 => "409 Conflict",
            Codes::SC410 => "410 Gone",
            Codes::SC411 => "411 Length Required",
            Codes::SC412 => "412 Precondition Failed",
            Codes::SC413 => "413 Payload Too Large",
            Codes::SC414 => "414 URI Too Long",
            Codes::SC415 => "415 Unsupported Media Type",
            Codes::SC416 => "416 Range Not Satisfiable",
            Codes::SC417 => "417 Expectation Failed",
            Codes::SC418 => "418 I'm a teapot",
            Codes::SC421 => "421 Misdirected Request",
            Codes::SC422 => "422 Unprocessable Entity",
            Codes::SC423 => "423 Locked",
            Codes::SC424 => "424 Failed Dependency",
            Codes::SC425 => "425 Too Early",
            Codes::SC426 => "426 Upgrade Required",
            Codes::SC428 => "428 Precondition Required",
            Codes::SC429 => "429 Too Many Requests",
            Codes::SC431 => "431 Request Header Fields Too Large",
            Codes::SC451 => "451 Unavailable For Legal Reasons",
            Codes::SC500 => "500 Internal Server Error",
            Codes::SC501 => "501 Not Implemented",
            Codes::SC502 => "502 Bad Gateway",
            Codes::SC503 => "503 Service Unavailable",
            Codes::SC504 => "504 Gateway Timeout",
            Codes::SC505 => "505 HTTP Version Not Supported",
            Codes::SC506 => "506 Variant Also Negotiates",
            Codes::SC507 => "507 Insufficient Storage",
            Codes::SC508 => "508 Loop Detected",
            Codes::SC510 => "510 Not Extended",
            Codes::SC511 => "511 Network Authentication Required",
        }
    }
}