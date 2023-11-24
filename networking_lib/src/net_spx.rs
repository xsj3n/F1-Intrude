use std::fmt::format;
use std::{net::*};
use std::io::{self, Write, Read, stdout};
use webpki_roots::*;
use std::collections::HashMap;
use native_tls::*;
use std::sync::Arc;
use std::fs;

#[path ="parse_util.rs"]
mod parse_util;
use self::parse_util::URI_COMPONENTS;

#[path = "err.rs"]
mod err;
use self::err::*;

enum opt_spacing 
{
    no,
    fin,
    space,
}

struct RequestMetaData
{
    url: URI_COMPONENTS,
    headers: HashMap<(u16,String), String>,
    verb: String,
    version: String
}

#[repr(C)]
pub struct HttpResponse
{
    status_code: u16,
    content_length: u32,
    
}


#[repr(C)]
pub struct TlsClient
{
    uri: URI_COMPONENTS,
    clean_close: bool,
    closing: bool,
    tls_conn: TlsStream<TcpStream>
}

impl TlsClient
{
    fn new(uri: URI_COMPONENTS, tls_conn: TlsStream<TcpStream>) -> Self
        {
            Self 
            {
                uri: uri,
                closing: false,
                clean_close: false,
                tls_conn:  tls_conn
            }
        }

}

impl RequestMetaData
{
    fn new(url: URI_COMPONENTS, headers: HashMap<String, String>, verb: String) -> Self
    {
        Self 
        {
            url: url , headers: HashMap::new() , verb: verb, version: "HTTP/1.1".to_string()
        }
    }
}

pub fn __send_comm__(tlsclient_st: &mut TlsClient, request_string: String) -> String
{
    // dont see why id need to handle the error for this- that's not stupid right?
    tlsclient_st.tls_conn.write_all(request_string.as_bytes())
    .unwrap();

    let mut response_buffer = Vec::<u8>::new();
    tlsclient_st.tls_conn.read_to_end(&mut response_buffer)
    .unwrap();

    let response = String::from_utf8_lossy(&response_buffer)
    .to_string();

    return response;
}



pub fn __start_com_cycle__() -> std::result::Result<TlsClient, String>
{

    //let url_parts = parse_util::parse_uri(url);
    let request_string = match parse_util::parse_burp_file()
    {
      Ok(rs) => rs,
      Err(e) => { return Err(e.details) }
    };

    let host = match parse_util::parse_host_from_cache_data(request_string)
    {
        Ok(hs) => hs + ":443",
        Err(e) => return Err(e.details)
    };
 

    let connector = TlsConnector::new().unwrap();
    let socket_addr = match host.to_socket_addrs()
    {
        Ok(mut it) => it.next().unwrap(),
        Err(e) => return Err(e.to_string())
    };
    
    let socket = TcpStream::connect(socket_addr).unwrap();
    let t_client = connector.connect(&host, socket).unwrap();

    let url_parts = URI_COMPONENTS { scheme: "1.1".to_string(),
     host: host, port: Some(443),
     path: "".to_string(), query: Some("".to_string()) };
    return Ok(TlsClient::new(url_parts, t_client));
    
}

pub fn tls_set(tc: TlsClient,  tc_global: &mut TlsClient) -> ()
{
    unsafe 
    { 
      *tc_global = tc;
    };
}


/* 
fn construct_request(mut request_data: RequestMetaData) -> Vec<u8>
{
    let mut h_o = 0; 
    let mut mpv = format!("{} {} {}\r\n", request_data.verb, request_data.url.path, request_data.version);
    
    request_data.headers.insert((h_o,"Host".to_string()), request_data.url.host);
    h_o += 1;

    for (k,v) in &request_data.headers
    {
        let head = (k.1.to_owned() + ": ") + (v.to_owned() + "\r\n").as_str();
        h_o += 1;
        mpv.push_str(&head);
    }

    mpv.push_str("Connection: close\r\n");
    mpv.push_str("\r\n");
    println!("Request:\n{}", mpv);
    return mpv.as_bytes().to_owned();

}

*/