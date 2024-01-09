use std::{num::IntErrorKind, slice::from_raw_parts, str::from_boxed_utf8_unchecked};
use unicode_segmentation::UnicodeSegmentation;


use crate::log::dbg_log_progress;

#[derive(Debug)]
pub struct CacheReadError
{
    pub details: String
}

impl CacheReadError
{
    pub fn new(msg: &str) -> CacheReadError
    {
        CacheReadError { details: msg.to_string()}
    }
}

pub struct URICOMPONENTS
{
    pub scheme: String,
    pub host: String,
    pub port: Option<u32>,
    pub path: String,
    pub query: Option<String>,
}
/* 
pub fn parse_uri(full_uri: String) -> URICOMPONENTS
{
    let uri_comps = Url::parse(&full_uri).unwrap();
    
    return URICOMPONENTS
    {
        scheme: uri_comps.scheme().to_string(),
        host: uri_comps.host().unwrap().to_string(),
        port:  match uri_comps.port() 
        {
            Some(p) => Some(p as u32),
            None => None
            
        },
        path: uri_comps.path().to_string(),
        query: match uri_comps.query() 
        {
            Some(q) => Some(q.to_string()),
            None => None
        }
    };

}
*/

pub fn parse_host_from_cache_data(request_string: &str) -> Result<String, CacheReadError>
{
    let mut host = String::new();
    let lines = request_string.split("\r\n");
    
    for line in lines 
    {
        if line.contains("Host:")
        {
            host = line.replace("Host: ", "")
            .replace("\r\n", "");
        
        }
    }

    let log_s = "[+] Host parsed: ".to_string() + &host;
    dbg_log_progress(&log_s);

    if host.is_empty() == true { return Err(CacheReadError::new("[!] Unable to parse host from the request in request cache")); }
    return Ok(host);
}


pub fn parse_burp_file() -> String
{
    dbg_log_progress("[+] parse_burp_file started");
    let req_byte_string = match std::fs::read_to_string("/Users/xis31/tmp/req_cache.dat")
    {
      Ok(s) => s, 
      Err(_) => 
      {
        dbg_log_progress("[!] Unable to read cache file");
        return String::new();
      },
    };

    let req_byte_string_iterator = req_byte_string.split("\n");
    let mut bytes: Vec<u8> = Vec::new();

    for strings in req_byte_string_iterator
    {
        match strings.parse::<u8>()  
        {
            Ok(i) => bytes.push(i),
            Err(e) => 
            {
                if e.kind() == &IntErrorKind::Empty 
                {
                    println!("[+] Reached end of Burp Suite request cache");
                }
            }
        };
    }
    

    let parsed_string = String::from_utf8_lossy(&bytes)
        .to_string();

    println!("[+] Request parsed from BurpSuite request cache:");
    print!("{}", parsed_string);
    return parsed_string;

}

pub fn __permutate_request__(perm_src: &str, perm_mod: &str) -> String
{
    
   
    let n_index = perm_src.find("†").unwrap_or(0);
    let n_e_index = perm_src.find("‡").unwrap_or(perm_src.len());
    
    let mut ns = perm_src.to_string();
    ns.replace_range(n_index..n_e_index + 3, perm_mod);
    return ns;
    

}