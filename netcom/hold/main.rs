use std::{net::TcpStream, io::{Write, Read}, ptr::null_mut};

use ffi::{HttpResponseDataKeepAliveC, HttpResponseDataC};
use log::dbg_log_progress;
use native_tls::TlsConnector;
use net_spx::{TlsClient, HTTPResult};
use net_spx::__send_comm_keepalive__;
use std::ffi::CString;

mod err;
mod ffi;
mod net_spx;
mod parse_util;
mod log;
mod async_net_spx;


fn main()
{
    send1();
}
fn send1()
{
    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect("google.com:443").unwrap();
    let stream = connector.connect("google.com", stream)
    .unwrap();
    let mut t_cli = TlsClient::new(stream);

    let req_close = "GET / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0\r\nConnection: close\r\n\r\n".to_string();
    let req = "GET / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0\r\nConnection: keep-alive\r\n\r\n".to_string();

    match __send_comm_keepalive__(&mut t_cli, req.clone())
    {
        Ok(_) => println!("ret 1"),
        Err(HTTPResult::MALFORMED) => println!("1st malformed"),
        Err(HTTPResult::WRITTING_STILL_INTO_BUFFER) => println!("Writting still..."),
        Err(HTTPResult::TLS_READ_ERROR) => println!("TLS read error "),
        Err(_) => ()
    }
    match __send_comm_keepalive__(&mut t_cli, req.clone())
    {
        Ok(_) => println!("ret 1"),
        Err(HTTPResult::MALFORMED) => println!("1st malformed"),
        Err(HTTPResult::WRITTING_STILL_INTO_BUFFER) => println!("Writting still..."),
        Err(HTTPResult::TLS_READ_ERROR) => println!("TLS read error "),
        Err(_) => ()
    }
    match __send_comm_keepalive__(&mut t_cli, req.clone())
    {
        Ok(_) => println!("ret 1"),
        Err(HTTPResult::MALFORMED) => println!("1st malformed"),
        Err(HTTPResult::WRITTING_STILL_INTO_BUFFER) => println!("Writting still..."),
        Err(HTTPResult::TLS_READ_ERROR) => println!("TLS read error "),
        Err(_) => ()
    }
    match __send_comm_keepalive__(&mut t_cli, req_close.clone())
    {
        Ok(r) =>
        {


        },
        Err(HTTPResult::MALFORMED) => println!("1st malformed"),
        Err(HTTPResult::WRITTING_STILL_INTO_BUFFER) => println!("Writting still..."),
        Err(HTTPResult::TLS_READ_ERROR) => println!("TLS read error "),
        Err(_) => ()
    }

}


fn send()
{
    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect("google.com:443").unwrap();
    let stream = connector.connect("google.com", stream)
    .unwrap();
    let mut t_cli = TlsClient::new(stream);

    let req_close = "GET / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0\r\nConnection: close\r\n\r\n".to_string();
    let req = "GET / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/118.0\r\nConnection: keep-alive\r\n\r\n".to_string();

    t_cli.tls_conn.write_all(req.as_bytes()).unwrap();
    t_cli.tls_conn.write_all(req_close.as_bytes()).unwrap();

    let mut v: Vec<u8> = Vec::new();
    t_cli.tls_conn.read_to_end(&mut v).unwrap();

    let s = String::from_utf8_lossy(&v).to_string();
    println!("{}", s);
}
  

#[cxx::bridge]
mod ffi {

    
    pub struct HttpHeadersC
    { // holds pointers to immutable data passed to C
        pub header: [String; 64],
        pub value:  [String; 64],
        pub init:   bool
    }

    
    pub struct HttpResponseDataC
    {
        pub headers: HttpHeadersC,
        pub body: String,
        pub status_code: u16,
        pub total_response_bytes: u32 
    }

    pub struct HttpResponseDataKeepAliveC
    {
        pub len: usize,
        pub http_response_data_c: Vec<HttpResponseDataC>,
    }

    fn start_com_cycle() -> u8;
    fn send_com_keep_alive(request_s: String) -> HttpResponseDataKeepAliveC;
    fn permutate_request(perm_string_ptr: String, perm_mod_ptr: String) -> String;

    
}
    

