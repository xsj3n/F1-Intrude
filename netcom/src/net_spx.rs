
use std::cell::RefCell;
use std::net::*;
use std::io::{Write, Read};
use core::result::Result;
use httparse::Response;
use native_tls::*;



#[path ="parse_util.rs"]
mod parse_util;
use crate::ffi::*;
use crate::log::dbg_log_progress;


pub struct TlsClient
{
    pub clean_close: bool,
    pub closing: bool,
    pub tls_conn: TlsStream<TcpStream>
}


impl TlsClient
{
    pub fn new(tls_conn: TlsStream<TcpStream>) -> Self
        {
            Self 
            {
                closing: false,
                clean_close: false,
                tls_conn:  tls_conn
            }
        }


}

#[derive(Debug)]
pub enum HTTPResult
{
    MALFORMED,
    TLS_READ_ERROR,
    WRITTING_STILL_INTO_BUFFER,
    OK
}

pub struct ResponseString(pub String);
impl ResponseString
{
    pub fn parse_response(self) -> Result<HttpResponseDataC, HTTPResult> 
    {
        let mut header_buffer = [httparse::EMPTY_HEADER; 64];
        let response_data_st: (Option<Response>, Option<String>) = match __recv_parse_comm__(&self.0, &mut header_buffer)
        {
            (None, Some(_)) => return Err(HTTPResult::MALFORMED),
            (Some(h), None) => (Some(h), None),
            (Some(h), Some(s)) => (Some(h), Some(s)),
            (None, None) => return Err(HTTPResult::MALFORMED)
        };

        let out = self.0.to_owned();
        return Ok(HttpResponseDataC::new(response_data_st, self.0.len(), out));
        

    }
}

pub struct ResponseBlotTransformer(String);
impl ResponseBlotTransformer
{
    fn transform(self) -> HttpResponseDataKeepAliveC
    {
        let s = self.0;
        let mut l = s.split("HTTP/1.1");
        
        let c = l.count() - 1;
        l = s.split("HTTP/1.1");

        let dbg_s = format!("[+] Parsing {} requests from buffer...", c);
        dbg_log_progress(&dbg_s);


        let mut response_buffer: Vec<HttpResponseDataC> = Vec::new();
        let mut ns = String::new();

        let mut i = 0;
        for single_request in l
        {
            if i != 0
            {
                ns = single_request.to_string();
                ns.insert_str(0, "HTTP/1.1");
    
                let header_c = ResponseString(ns.clone()).parse_response();
    
                match header_c
                {
                    Ok(h) => response_buffer.push(h),
                    Err(_) => response_buffer.push(HttpResponseDataC::new((None, None), 0, ns))
                }
            }

            i += 1;
        }

        let len = response_buffer.len();
        
        let dbg_s = format!("[+] Returning {} HRDs...", len);
        dbg_log_progress(&dbg_s);

        return HttpResponseDataKeepAliveC::new(response_buffer, len, false);
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

pub fn __recv_parse_comm__<'a, 'b>(response_s: &'a str, http_header_buffer: &'b mut [httparse::Header<'a>]) -> (Option<httparse::Response<'a, 'b>>, Option<String>)
{

    let mut response_headers: Response<'b, 'b> = httparse::Response::new(http_header_buffer);

    match response_headers.parse(response_s.as_bytes())
    {
        Ok(_) => (),
        Err(e) => 
        {
            let err = e.to_string();
            let mut err_s = "[!] HTTParse lib returned error on parsing a response: ".to_string() + &err;
            err_s.push_str(". Troublesome request below:\n");
            err_s.push_str(&response_s);

            dbg_log_progress(&err_s);

            return (None, Some("MALFORMED RESPONSE BELOW:\n".to_string() + &response_s));
        }
    };

   match BodyString(response_s).is_body()
   {
       Some(s) => return (Some(response_headers), Some(s)),
       None => return (Some(response_headers), None)
   };

}


pub enum KeepAlive
{
    TRUE,
    END,
}


pub fn __send_comm__(request_string: String) -> Result<HttpResponseDataC, HTTPResult>
{

    let dbg_s = "[+] Starting request, writting all bytes...: ".to_string();
    dbg_log_progress(&(dbg_s + &request_string));

    let socket_info: SocketInfo = SOCK_ADDR.with(|sock: &RefCell<SocketInfo>|
        {
            sock.borrow_mut().clone()
        });

    let connector = match TlsConnector::new()
    {
        Ok(c) => c,
        Err(_) => 
        {
            dbg_log_progress("[!] native_tls: TlsConnector init failed!");
            return Err(HTTPResult::MALFORMED);
        }
    };

    let socket = match TcpStream::connect(socket_info.Addr)
    {
        Ok(s) => s,
        Err(_) => 
        {
            dbg_log_progress("[!] Failure to connect to socket!");
            return Err(HTTPResult::MALFORMED);
        }
    };

    let hs: String = SOCK_ADDR.with(|sock: &RefCell<SocketInfo> | 
        {
            sock.borrow().host.clone()
            
        });

    let mut t_client = match connector.connect(&hs, socket)
    {
        Ok(t_cli) => t_cli,
        Err(e) => 
        {
            let err_s = ("[!] TLS Handshake error!".to_string(), e.to_string());
            dbg_log_progress(&(err_s.0 + &err_s.1));
            return Err(HTTPResult::TLS_READ_ERROR)
        }
    };


    match t_client.write_all(request_string.as_bytes())
    {
            Ok(_) => dbg_log_progress("[+] Write to socket success!"),
            Err(e) => 
            {
                let err_s = e.to_string();
                let wre = "[!] Write_all Error: ".to_string() + &err_s;
                dbg_log_progress( &wre);
                return Err(HTTPResult::MALFORMED);
            } 
    };

    let mut buffer: Vec<u8> = Vec::new();
    match t_client.read_to_end(&mut buffer)
    {
        Ok(bytes_read) => 
        {
            let n = bytes_read.to_string();
            let mut s = "[+] Read ".to_string();
            s.push_str(&n);
            s.push_str(" bytes from the server!");
            dbg_log_progress(&s);
        },
        Err(e) =>
        {
            let err_s = e.to_string();
            let wre = "[!] read_to_end Error: ".to_string() + &err_s;
            dbg_log_progress( &wre);
            return Err(HTTPResult::TLS_READ_ERROR);
        }
    }

    match ResponseString(String::from_utf8_lossy(&buffer).to_string()).parse_response()
    {
        Ok(hdrc) => return Ok(hdrc),
        Err(e) => return Err(e)
    }

}



thread_local! {static RESP_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new()); }
pub fn __send_comm_keepalive__(tlsclient_st: &mut TlsClient, request_string: String) -> Result<HttpResponseDataKeepAliveC, HTTPResult>
{
    let dbg_s = "[+] Keep-Alive: Starting request, writting all bytes...: ".to_string();
    dbg_log_progress(&(dbg_s + &request_string));

    if request_string.is_empty()
    {
        dbg_log_progress("[!] Request empty...");
        return Err(HTTPResult::MALFORMED);
    }

    
    if !request_string.contains("Connection: close")
    {
        match tlsclient_st.tls_conn.write_all(request_string.as_bytes())
        {
            Ok(_) => 
            {
                dbg_log_progress("[+] Write to socket success!");
                return Err(HTTPResult::WRITTING_STILL_INTO_BUFFER);
            },
            Err(e) => 
            {
                let err_s = e.to_string();
                let wre = "[!] Write_all Error: ".to_string() + &err_s;
                dbg_log_progress( &wre);
                return Err(HTTPResult::MALFORMED);
            } 
        }

    } 

    match tlsclient_st.tls_conn.write_all(request_string.as_bytes())
    {
        Ok(_) => dbg_log_progress("[+] Write to socket success!"),
        Err(e) => 
        {
            let err_s = e.to_string();
            let wre = "[!] Write_all Error: ".to_string() + &err_s;
            dbg_log_progress( &wre);
            return Err(HTTPResult::MALFORMED);
        } 
    }

    let mut response: String = String::new();
    let mut tls_err_flag = false;
    RESP_BUFFER.with(|buffer: &RefCell<Vec<u8>>| 
        {     
            match tlsclient_st.tls_conn.read_to_end(&mut (*buffer.borrow_mut()))
            {
                Ok(bytes_read) => 
                {
                    let n = bytes_read.to_string();
                    let mut s = "[+] Read ".to_string();
                    s.push_str(&n);
                    s.push_str(" bytes from the server!");
                    dbg_log_progress(&s);
                },
                Err(e) =>
                {
                    let err_s = e.to_string();
                    let wre = "[!] read_to_end Error: ".to_string() + &err_s;
                    dbg_log_progress( &wre);
                    tls_err_flag = true;
                }
            }
            response = String::from_utf8_lossy(&(*buffer.borrow())).to_string();
        });



    if response.is_empty() 
    {
        dbg_log_progress("[!] Request empty from keep-alive buffer...");
        return Err(HTTPResult::MALFORMED);
    }

    if tls_err_flag == true
    {
        return Err(HTTPResult::TLS_READ_ERROR);
    }



   let response_v = ResponseBlotTransformer(response).transform();
   return Ok(response_v);


}

#[derive(Clone)]
pub struct SocketInfo
{
    pub Addr: SocketAddr,
    pub port: u16,
    pub r_string: String,
    pub host: String
}


thread_local! {pub static SOCK_ADDR: RefCell<SocketInfo> = RefCell::new(SocketInfo 
    {
        Addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 443),
        port: 443,
        r_string: String::new(),
        host: String::new()
    });}
pub fn __start_com_cycle__() -> std::result::Result<TlsClient, String>
{


    //let url_parts = parse_util::parse_uri(url);
    let request_string = parse_util::parse_burp_file();
  
    _ = dbg_log_progress("[+] parsed burp file...");

    let mut host: String = match parse_util::parse_host_from_cache_data(&request_string)
    {
        Ok(hs) =>
        {
            crate::DOMAIN_BUF.with(|d: &std::sync::Arc<RefCell<String>>| 
                {
                    d.borrow_mut().push_str(&hs);
                });
        
            SOCK_ADDR.with(|sock: &RefCell<SocketInfo>|
                {
                    sock.borrow_mut().host = hs.clone();
                });
                hs
        },
        Err(e) =>
        {
            let err_s = "[!] Failure in parse_host_from_cache_data: ".to_string() + &e.details;
            dbg_log_progress(&err_s);
            return Err(e.details);
        }
    };

    let host_no_port = host.clone();
    host += ":443";

    _ = dbg_log_progress("[+] Server hostname parsed from burp file contents...");
 
    
    let connector = match TlsConnector::new()
    {
        Ok(c) => c,
        Err(_) => 
        {
            dbg_log_progress("[!] native_tls: TlsConnector init failed!");
            return Err("".to_string())
        }
    };
    _ = dbg_log_progress("[+] TlsConnector constructed");

    let socket_addr: SocketAddr = match host.to_socket_addrs()
    {
        Ok(mut it) => match it.next()
        {
            Some(it) =>
            {
                SOCK_ADDR.with(|sock: &RefCell<SocketInfo>|
                    {
                        sock.borrow_mut().Addr = it.clone();
                    });
                    it

            }, None => return Err("".to_string())
        },
        Err(e) => 
        {
            dbg_log_progress("[!] Failure gathering socket address from host");
            return Err(e.to_string())
        }
    };
    _ = dbg_log_progress("[+] Socket addrs constructed");
    
    let socket = match TcpStream::connect(socket_addr)
    {
        Ok(s) => s,
        Err(_) => 
        {
            dbg_log_progress("[!] Failure to connect to socket!");
            return Err("".to_string())
        }
    };
    _ = dbg_log_progress("[+] TCP Stream opened");

    let t_client = match connector.connect(&host_no_port, socket)
    {
        Ok(t_cli) => t_cli,
        Err(e) => 
        {
            let err_s = ("[!] TLS Handshake error!".to_string(), e.to_string());
            dbg_log_progress(&(err_s.0 + &err_s.1));
            return Err("Handshake error".to_string())
        }
    };

    _ = dbg_log_progress("[+] TlsStream Connected");


    return Ok(TlsClient::new(t_client));
    
}


