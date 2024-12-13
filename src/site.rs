use std::{fmt::Display, io::Read};

use crate::{
    http::{
        request::HTTPRequest,
        server::Client,
        utils::{ContType, HTML},
        account::AuthLevel
    },
    openfile::openfile,
    traits::{New, OpttoString},
};

pub struct Site {
    pub file_type: ContType,
    pub path: Box<str>,
    pub auth: AuthLevel,
    pub site: SiteType,
}

pub enum SiteType {
    StaticSite(SSite),
    ServerSideRenderedSite(SSRSite),
}

pub struct SSite {
    pub file_path: Box<str>,
    pub cache: bool,
    pub cached: Option<Box<[u8]>>,
}

pub struct SSRSite {
    pub genfunc: fn(&HTTPRequest, &Client) -> Vec<u8>,
}

pub struct SiteConf {
    auth_level: u8,
    ct: ContType,
    online_path: String,
    ss: bool,
    
    gen: Option<fn(&HTTPRequest, &Client) -> Vec<u8>>,

    file_path: Option<String>,
    cache: Option<bool>,
}

impl Site {
    fn ss_inner(&self) -> &SSite {
        match &self.site {
            SiteType::StaticSite(ss) => ss,
            _ => unreachable!(),
        }
    }
    fn ss_mut_inner(&mut self) -> &mut SSite {
        match &mut self.site {
            SiteType::StaticSite(ss) => ss,
            _ => unreachable!(),
        }
    }
}

impl SSite {
    fn cache(site: &mut Site) {
        if site.ss_inner().cache {
            let mut file = openfile(
                &site.ss_inner().file_path,
                false,
                true,
                false,
                false,
            );
            let mut v = Vec::with_capacity(file.metadata().expect("Idk I hate Windows. ITs it fault. Metadata.").len() as usize);
            let _ = file.read_to_end(&mut v);
            site.ss_mut_inner().cached = Some(v.into_boxed_slice());
        } else {
            site.ss_mut_inner().cached = None;
        };
    }
}

impl Site {
    pub fn read_all(sites: &Vec<SiteConf>) -> Vec<Site> {
        let mut sites = sites
            .iter()
            .map(|site| {
                if site.ss {
                    let mut site = Site {
                        file_type: site.ct.clone(),
                        path: site.online_path.clone().into_boxed_str(),
                        auth: site.auth_level,
                        site: SiteType::StaticSite(SSite {
                            file_path: site.file_path.clone().unwrap().into_boxed_str(),
                            cache: site.cache.unwrap(),
                            cached: None,
                        }),
                    };
                    SSite::cache(&mut site);
                    site
                } else {
                    Site {
                        file_type: site.ct.clone(),
                        path: site.online_path.clone().into_boxed_str(),
                        auth: site.auth_level,
                        site: SiteType::ServerSideRenderedSite(SSRSite {
                            genfunc: site.gen.unwrap(),
                        }),
                    }
                }
            })
            .collect::<Vec<Site>>();
        sites.sort_by(|sitea, siteb| sitea.path.cmp(&siteb.path));
        sites
    }
    pub fn get_page(&self, req: &HTTPRequest, data: &Client) -> Vec<u8> {
        match &self.site {
            SiteType::StaticSite(ss) => {
                if ss.cache {
                    ss.cached.clone().unwrap().to_vec()
                } else {
                    let mut v = Vec::new();
                    let _ = openfile(&ss.file_path, false, true, false, false).read_to_end(&mut v);
                    v
                }
            }
            SiteType::ServerSideRenderedSite(ssrs) => (ssrs.genfunc.clone())(req, data),
        }
    }
}

impl Default for Site {
    fn default() -> Self {
        Self {
            file_type: HTML,
            path: "".to_string().into_boxed_str(),
            auth: 255,
            site: SiteType::StaticSite(SSite {
                file_path: "index.html".to_string().into_boxed_str(),
                cache: false,
                cached: None,
            }),
        }
    }
}

impl New for Site {}

impl Display for SiteConf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ss {
            write!(
                f,
                "Real Path: {}; ContentType: {}; Is Static: {}; Auth Level: {}; is_cached: {}; FilePath: {}", self.online_path, self.ss, self.auth_level, self.cache.unwrap(), self.ct.as_str(), self.file_path.clone().unwrap()
            )
        } else {
            write!(
                f,
                "Real Path: {}; ContentType: {}; Is Static: {}; Auth Level: {}; ",
                self.online_path,
                self.ct.as_str(),
                self.ss,
                self.auth_level
            )
        }
    }
}

impl SiteConf {
    pub fn from_tuple(
        tup: &(
            /*ContentType:*/ &str,
            /*OnlinePath:*/ &str,
            /*AuthorityLevel:*/ u8,
            /*Is Static:*/ bool,
            /*FilePath:*/ Option<&str>,
            /*Should Cached:*/ Option<bool>,
            /*OnlinePath:*/ Option<fn(&HTTPRequest, &Client) -> Vec<u8>>,
        ),
    ) -> SiteConf {
        SiteConf {
            ct: ContType::from_str(tup.0),
            online_path: tup.1.to_string(),
            auth_level: tup.2,
            ss: tup.3,
            gen: tup.6,
            file_path: tup.4.to_string(),
            cache: tup.5,
        }
    }

    pub fn new(
        im: Vec<(
            /*ContentType:*/ &str,
            /*OnlinePath:*/ &str,
            /*AuthorityLevel:*/ u8,
            /*Is Static:*/ bool,
            /*FilePath:*/ Option<&str>,
            /*Should Cached:*/ Option<bool>,
            /*OnlinePath:*/ Option<fn(&HTTPRequest, &Client) -> Vec<u8>>,
        )>,
    ) -> Vec<SiteConf> {
        im.iter()
            .map(|site| SiteConf::from_tuple(site))
            .collect::<Vec<SiteConf>>()
    }
}

pub struct Api {
    pub path: Box<str>,
    pub fnp: fn(&HTTPRequest, &Client)->Vec<u8>,
    pub filetype: ContType,
    pub auth: AuthLevel,
}

impl Api {
    pub fn get_resp(&self, req: &HTTPRequest, res: &Client)->Vec<u8>  {
        (self.fnp)(req, res)
    }
}