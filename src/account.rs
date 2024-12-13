use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fiv::{encrypt::encrypt::AesKey, openfile::openfile, traits::{FileUtils, WS}};

pub type AuthLevel = u8;

pub struct Account {
    pub id: u16,
    pub level: u8,
    pub username: Box<str>,
    pub password: Box<str>,
}

pub struct SessionId {
    sid: [u8;16],
    aid: u16,
    t: SystemTime
}

impl Account {
    pub fn new(v: &mut Vec<Account>, lvl: u8, username: String, password: String) -> Result<(),()> {
        if username.len() <= 24 && password.len() <= 24 {
            let i = 0;
            while i < v.len() && i < u16::MAX.into(){
                if v[i].id != i as u16 {
                    v.insert(i, Account {
                        id: i as u16,
                        level: lvl,
                        username: username.into_boxed_str(),
                        password: password.into_boxed_str()
                    });
                    return Ok(());
                }
            }
            if i < u16::MAX.into() {
                v.push(Account {
                    id: i as u16,
                    level: lvl,
                    username: username.into_boxed_str(),
                    password: password.into_boxed_str()
                })
            }else {
                return Err(());
            }
        }
        Err(())
    }
    pub fn sort(v: &mut Vec<Account>) {
        v.sort_by(|a, b|a.id.cmp(&b.id));
    }
    pub fn from_file(src: &str, key: &AesKey) -> Vec<Account> {
        let mut f = openfile(&src, false, true, false, false);
        let v = f.cry_save_read(1, 65, key).unwrap();
        unsafe{
            let str = String::from_utf8_unchecked(v);
            str.lines()
                .filter_map(|line| {
                    let id =
                        u16::from_be_bytes([line.chars().nth(0)? as u8,line.chars().nth(1)? as u8]);
                    let (un, ps) = line[3..].split_once(';')?;
                    let username = un.to_string().into_boxed_str();
                    let password = ps.to_string().into_boxed_str();
                    Some(Account {
                        id: id,
                        level: line.chars().nth(2)? as u8,
                        username: username,
                        password: password,
                    })
                })
                .collect::<Vec<Account>>()
        }
    }
    pub fn to_file(v: &Vec<Account>, key: &AesKey, path: &str) {
        let x = v.iter().map(|acc| {
            unsafe{
            let x = acc.id.to_be_bytes();
            format!("{}{}{}{};{}", char::from_u32_unchecked(x[0] as u32), char::from_u32_unchecked(x[1] as u32), char::from_u32_unchecked(acc.level as u32), acc.username, acc.password)
    }}).collect::<String>().into_bytes();
        openfile(&path, true, false, false, true).cry_save_write(1, 65, x, key);
    }
}

impl SessionId {
    pub fn new(a: u16)->SessionId {
        let mut b = [0;16];
        for i in 0..17 {
            let x = random();
            if x != '\n' as u8 && x != ':' as u8 {
                b[i] = x;
            }else {
                
            }
        }
        SessionId {
            sid: b,
            aid: a,
            t: SystemTime::now()
        }
    }
    pub fn from_file(src: &str, key: &AesKey)->Vec<SessionId> {
        let mut f = openfile(&src, false, true, false, false);
        let v = f.cry_save_read(1, 66, key).expect("Session File is corrupted");
        unsafe {
            let str = String::from_utf8_unchecked(v);
            return str.lines().filter_map(|line|{
                if line.len() == 26 {
                    let mut x: [u8;2] = [0;2];
                    let mut y: [u8;8] = [0;8];
                    let mut z: [u8;16] = [0;16];
                    for i in 0..2 {
                        x[i] = line.chars().nth(i).unwrap() as u8;
                    }
                    for i in 2..10 {
                        y[i-2] = line.chars().nth(i).unwrap() as u8;
                    }
                    for i in 10..26 {
                        z[i-10] = line.chars().nth(i).unwrap() as u8;
                    }
                    Some(SessionId {
                        aid: u16::from_be_bytes(x),
                        sid: z,
                        t: UNIX_EPOCH + Duration::from_secs(u64::from_be_bytes(y))
                    })
                }else {
                    None
                }
            }).collect::<Vec<SessionId>>()
        }
    }
    pub fn to_file(v: &Vec<SessionId>, key: &AesKey, path: &str) {
        let x = v.iter().map(|s| {
            let mut v = [0;26];
            let x = s.aid.to_be_bytes();
            let y = s.t.duration_since(UNIX_EPOCH).unwrap().as_secs().to_be_bytes();
            for i in 0..27 {
                match i {
                    0..2=>v[i]=x[i],
                    2..10=>v[i]=y[i-2],
                    10..27=>v[i]=s.sid[i-10],
                    _=>unreachable!()
                }
            }
            v
        }).collect::<Vec<[u8;26]>>();
        let v = x.join(&b'\n');
        openfile(&path, true, false, false, true).cry_save_write(1, 66, v, key);
    }
    pub fn from_id(sess: &Vec<SessionId>, mut str: String)->Option<u16> {
        str.trima();
        if str.len() != 16 {
            return None;
        }
        let b = str.as_bytes();
        let mut x: [u8; 16] = [0;16];
        for i in 0..17 {
            x[i]=b[i];
        }
        match sess.binary_search_by(|sess| sess.sid.cmp(&x)) {
            Ok(index)=>{
                return Some(sess[index].aid)
            },
            Err(_)=>return None
        }
    }
    pub fn check_exp(v: &mut Vec<SessionId>, x: u64) {
        let n = SystemTime::now();
        let mut i = 0;
        while i < v.len() {
            if Duration::as_secs(&n.duration_since(v[i].t).unwrap()) / 86400 >= x {
                v.remove(i);
            }
            i+=1;
        }
    }
}