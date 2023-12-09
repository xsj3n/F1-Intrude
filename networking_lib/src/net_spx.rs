
use std::net::*;
use std::io::{Write, Read};
use core::result::Result;
use httparse::Response;
use native_tls::*;


#[path ="parse_util.rs"]
mod parse_util;
use crate::ffi::HttpResponseDataC;
use crate::parse_util::URICOMPONENTS;

#[path = "err.rs"]
mod err;

pub struct TlsClient
{
    pub uri: URICOMPONENTS,
    pub clean_close: bool,
    pub closing: bool,
    pub tls_conn: TlsStream<TcpStream>
}


impl TlsClient
{
    fn new(uri: URICOMPONENTS, tls_conn: TlsStream<TcpStream>) -> Self
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

pub enum HTTPResult
{
    MALFORMED,
    OK
}

pub struct ResponseString(pub String);
impl ResponseString
{
    pub fn parse_response(&self) -> Result<HttpResponseDataC, HTTPResult> 
    {
        let mut header_buffer = [httparse::EMPTY_HEADER; 64];
        let response_data_st: (Option<Response>, Option<String>) = match __recv_parse_comm__(&self.0, &mut header_buffer)
        {
            (None, Some(_)) => return Err(HTTPResult::MALFORMED),
            (Some(h), None) => (Some(h), None),
            (Some(h), Some(s)) => (Some(h), Some(s)),
            (None, None) => return Err(HTTPResult::MALFORMED)
        };


        return Ok(HttpResponseDataC::new(response_data_st));
        

    }
}

// this is unreliable
struct BodyString<'a>(&'a str);
impl BodyString<'_>
{
    fn is_body(&self) -> Option<String>
    {
        match self.0.split("\r\n\r\n").nth(1)
        {
            Some(s) => 
            {
                if !s.is_empty() 
                { 
                    return Some(s.to_string()); 
                }
                else {return None;}
            },
            None => { return None; }
        }
    }
}



pub fn __send_comm__(tlsclient_st: &mut TlsClient, request_string: String) -> Result<HttpResponseDataC, HTTPResult>
{
    // dont see why id need to handle the error for this- that's not stupid right?
    tlsclient_st.tls_conn.write_all(request_string.as_bytes())
    .unwrap();

    let mut response_buffer = Vec::<u8>::new();
    tlsclient_st.tls_conn.read_to_end(&mut response_buffer)
    .unwrap();

    let response = String::from_utf8_lossy(&response_buffer)
    .to_string();

    return ResponseString(response).parse_response();
}


pub fn __recv_parse_comm__<'a, 'b>(response_s: &'a str, http_header_buffer: &'b mut [httparse::Header<'a>]) -> (Option<httparse::Response<'a, 'b>>, Option<String>)
{

    let mut response_headers: Response<'b, 'b> = httparse::Response::new(http_header_buffer);

    match response_headers.parse(response_s.as_bytes())
    {
        Ok(_) => (),
        Err(_) => return (None, Some("MALFORMED RESPONSE BELOW:\n".to_string() + &response_s))
    };

   match BodyString(response_s).is_body()
   {
       Some(s) => return (Some(response_headers), Some(s)),
       None => return (Some(response_headers), None)
   };




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

    let url_parts = URICOMPONENTS { scheme: "1.1".to_string(),
     host: host, port: Some(443),
     path: "".to_string(), query: Some("".to_string()) };
    return Ok(TlsClient::new(url_parts, t_client));
    
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
struct RequestMetaData
{
    url: URI_COMPONENTS,
    headers: HashMap<(u16,String), String>,
    verb: String,
    version: String
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

*/