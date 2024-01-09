
use ffi::{HttpResponseDataC, RequestandPermutation};
use log::dbg_log_progress;
use parse_util::parse_host_from_cache_data;
use std::cell::RefCell;
use std::num::IntErrorKind;
use std::sync::Arc;
use crate::net_spx::*;
use crate::parse_util::__permutate_request__;

mod net_spx;
mod parse_util;
mod log;
mod ffi_wrp;
mod async_net_spx;






#[cxx::bridge]
pub mod ffi {
    
    #[derive(Clone)]
    pub struct RequestandPermutation
    {
        pub request: Vec<String>,
        pub permutation: Vec<String>
    }
    
    pub struct HttpHeadersC
    { // holds pointers to immutable data passed to C
        pub header: [String; 64],
        pub value:  [String; 64],
        pub init:   bool
    }

    
    pub struct HttpResponseDataC
    {
        pub headers: HttpHeadersC,
        pub full_response: String,
        pub body: String,
        pub status_code: u16,
        pub total_response_bytes: u32 
    }

    pub struct HttpResponseDataKeepAliveC
    {
        pub len: usize,
        pub http_response_data_c: Vec<HttpResponseDataC>,
    }

    extern "Rust"
    {
        fn start_com_cycle()                                                -> u8;
        fn send_com_keep_alive(request_s: String)                           -> HttpResponseDataKeepAliveC;
        fn send_com(request_s: String)                                      -> HttpResponseDataC;
        fn permutate_request(perm_string_ptr: String, perm_mod_ptr: String) -> String;
        fn parse_burp_request_cache()                                       -> String;
        fn parse_burp_file_export()                                         -> String;

    }

    
}
/*
    There is no support for passing CXX function pointers to rust,
    so we'll manage the function pointer for the ui call back ourself
 */
thread_local!{ static DOMAIN_BUF: Arc<RefCell<String>> = Arc::new(RefCell::new(String::new())); }
pub type Callback = Option< extern "C" fn(hrdc: HttpResponseDataC, permuation: String, row_num: u16) -> bool>;

#[allow(improper_ctypes_definitions)] // Regardless if CXX does not manage the function, CXX's ::rust::Vec types should still work 
#[no_mangle]
pub extern "C" fn send_com_async(request_permutation_buffer: RequestandPermutation, cb: Callback) -> bool
{
    if request_permutation_buffer.request.len() == 0
    {
        return false;
    }


    let tk_rt = tokio::runtime::Runtime::new()
    .unwrap();

    tk_rt.block_on(async move 
    {
        let tsk_d_str = DOMAIN_BUF.with(|d: &Arc<RefCell<String>>| { d.borrow_mut().clone() } );
    
        async_net_spx::start_taskmaster(tsk_d_str, request_permutation_buffer, cb).await;
    });

    return true;
}



pub fn parse_burp_file_export() -> String
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

    DOMAIN_BUF.with(|d: &Arc<RefCell<String>>| 
        {
            let inp_str = parse_host_from_cache_data(&parsed_string).unwrap();
            d.borrow_mut().push_str(&inp_str);
        });


    println!("[+] Request parsed from BurpSuite request cache:");
    print!("{}", parsed_string);
    return parsed_string;
}
    



impl ffi::HttpResponseDataKeepAliveC
{
    pub fn new(hrd_v: Vec<ffi::HttpResponseDataC>, len: usize, empty: bool) -> ffi::HttpResponseDataKeepAliveC
    {

        let mut r = ffi::HttpResponseDataKeepAliveC 
        {
            http_response_data_c: Vec::new().into(),
            len: len
            
        };
        
        if empty == false
        {
            r.http_response_data_c = hrd_v.try_into()
                .unwrap_or(Vec::new().into());
        }

        return r;
    }   
}

impl ffi::HttpHeadersC
{
    fn new() -> ffi::HttpHeadersC
    {
        let empt_v  = vec![String::new(); 64];
        let empt_v2 = vec![String::new(); 64];

        return ffi::HttpHeadersC
        {
            header: empt_v.try_into().unwrap(),
            value: empt_v2.try_into().unwrap(),
            init: false,
       };

    }

}



impl ffi::HttpResponseDataC
{
    pub fn new(response_tp: (Option<httparse::Response>, Option<String>), bytes_from_server: usize, full_response_string: String) -> ffi::HttpResponseDataC
    {

      
       if response_tp.0.is_none() && response_tp.1.is_none()
       {
            dbg_log_progress("[!] Failed to parse into HTTPResponseDataC, no response or body found...");
            return ffi::HttpResponseDataC 
            {
                headers:              ffi::HttpHeadersC::new(),
                full_response:        String::new(),
                body:                 String::new(),  
                status_code:          0,
                total_response_bytes: 0
            };
       }
 

        let r = ResponseFFITransformer(response_tp.0.unwrap());
        let code = r.0.code.unwrap_or(0);

        let mut http_response_data = ffi::HttpResponseDataC 
        {
            headers:              r.transform(),
            full_response:        full_response_string,
            body:                 String::new(),  
            status_code:          code,
            total_response_bytes: bytes_from_server as u32
        };

        match response_tp.1
        {
            Some(b) => http_response_data.body = b,
            None => ()
        }

        return http_response_data;
    }

}



pub struct ResponseFFITransformer<'h, 'b>(httparse::Response<'h, 'b>);
impl<'h, 'b> ResponseFFITransformer<'h, 'b>
{
    // ill check the results on cstring creations if it causes problems 
    fn transform(self) -> ffi::HttpHeadersC
    {   
        let mut http_struct = ffi::HttpHeadersC::new();
        
        // why would null bytes appear... right, right?!
        let mut i = 0;
        for h in self.0.headers
        {
            let bf = h.value.to_vec();
            let strs = String::from_utf8_lossy(&bf).to_string();

            http_struct.value[i] = strs;
            http_struct.header[i] = h.name.to_string();

            i += 1;
        }

        return http_struct;

    }
}


#[derive(PartialEq)]
#[derive(Copy, Clone)]
enum STATE 
{
    UNINIT,
    INIT,
    //LOCKED,
    READY
}

thread_local! {static STATE_T: RefCell<STATE> = RefCell::new(STATE::UNINIT);}
static mut TLS_CLIENT: Option<net_spx::TlsClient> = None;


fn start_com_cycle() -> u8
{
    
    if get_state() != STATE::UNINIT
    {
        dbg_log_progress("[!] Failed due to state lock");
        return 0;
    }

    dbg_log_progress("[*] __start_com__cycle INIT START");
    match __start_com_cycle__()
    {
        Ok(tc) => unsafe {TLS_CLIENT = Some(tc)},
        Err(_) => return 0
    };


    dbg_log_progress("[*] __start_com__cycle INIT DONE");
    set_state(STATE::INIT);

    return 1;

}




fn parse_burp_request_cache() -> String
{

    if get_state() != STATE::INIT
    {
        return String::new();
    }    

    let rust_string = parse_util::parse_burp_file();

    set_state(STATE::READY);
    return rust_string;

}



fn permutate_request(perm_string_c: String, perm_mod_c: String) -> String
{
    if perm_string_c.is_empty() || perm_mod_c.is_empty()
    {
        return String::new();
    }

    //let dbg_s: String = "[+] Original String:\n".to_string() + &perm_string_c + "\n[+] Permuatation to insert:  " + &perm_mod_c;
    //dbg_log_progress(&dbg_s);

    let permutation = __permutate_request__(&perm_string_c, &perm_mod_c);

    return permutation;
}




fn send_com_keep_alive(request_s: String) -> ffi::HttpResponseDataKeepAliveC
{
    if get_state() != STATE::READY
    {
        dbg_log_progress("Send_Com failure: state not ready");
        return ffi::HttpResponseDataKeepAliveC::new(Vec::new(), 0, true);
    }


    dbg_log_progress("Reading request from C...");
    let reques_rs_s: std::string::String = request_s.to_string();

    
    let response =  unsafe { __send_comm_keepalive__ (&mut TLS_CLIENT.as_mut().unwrap(),reques_rs_s)};

    dbg_log_progress("Response generated, transferring to C...");
    match response 
    {
        Ok(hrdc) => return hrdc,
        Err(HTTPResult::WRITTING_STILL_INTO_BUFFER) => return ffi::HttpResponseDataKeepAliveC::new(Vec::new(), 1, true),
        Err(_) => return ffi::HttpResponseDataKeepAliveC::new(Vec::new(), 0, true)
    };

}

fn send_com(request_s: String) -> ffi::HttpResponseDataC
{
    if get_state() != STATE::READY
    {
        dbg_log_progress("[!] Send_Com failure: state not ready");
        return ffi::HttpResponseDataC::new((None, None), 0, request_s);
    }

    dbg_log_progress("[+] Reading request from C...");


    let response =  unsafe { __send_comm__ (request_s.clone())};

    dbg_log_progress("[+] Response generated, transferring to C...");
    match response
    {
        Ok(hrdc) => return hrdc,
        Err(_) => return ffi::HttpResponseDataC::new((None, None), 0, request_s)
    };

}




    



/* ======destruct

fn rdealloc_string(string: String) -> ()
{
    unsafe{ CString::from_raw(string) };
}


fn rdealloc_http_response_data(obj: *mut HttpResponseDataC) -> ()
{
    unsafe { Box::from_raw(obj); } 
}
*/ 
// ======STATE
fn get_state() -> STATE
{
    let st: STATE = STATE_T.with(|state: &RefCell<STATE> | 
        {
            *state.borrow()
        });

    return st;
}

fn set_state(state_set: STATE) -> ()
{
    STATE_T.with(|state: &RefCell<STATE> | 
        {
            *state.borrow_mut() = state_set;
        });

}
/*
June 1, 172; first surviving instance of megistou kai megistou
theou megalou Hermou

API USAGE ORDER:
1. Init tls client: start_com_cycle
1. Parase burp file
2. SEND REQUEST - send_com 
2. PERMUTATE REQUEST - permutate_request
3. INIT - start_com_cycle 


*/