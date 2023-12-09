use libc::c_char;
use std::ffi::CString;
use std::cell::RefCell;
use crate::ffi::*;

use crate::net_spx::*;
use crate::parse_util::__permutate_request__;
mod net_spx;
mod parse_util;
mod err;
mod ffi;
type BOOL = u8;

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



/*
 */
#[no_mangle]
pub extern "C" fn start_com_cycle() -> BOOL
{
    if get_state() != STATE::UNINIT
    {
        return 0;
    }

    match __start_com_cycle__()
    {
        Ok(tc) => unsafe {TLS_CLIENT = Some(tc)},
        Err(_) => return 0
    };

    set_state(STATE::INIT);

    return 1;

}



// This function will have to be called first
// This will pass the request to the gui for editing, and init the buffer which will be used as a cache to lessen passing accross ffi
#[no_mangle]
pub extern "C" fn ParseBurpRequestCache() -> Option<*mut c_char>
{

    if get_state() != STATE::INIT
    {
        return None;
    }    

    let empty: [char; 0] = [];
    let rust_string = match parse_util::parse_burp_file()
    {
        Ok(s) => s,
        Err(_) => return None
    };

    
    set_state(STATE::READY);
    return pass_to_c(&rust_string);

}


#[no_mangle]
pub extern "C" fn permutate_request(perm_string_ptr:*const c_char,) -> Option<*mut c_char>
{
    let perm_string = read_immut_string_from_c(perm_string_ptr);
    let mod_string = __permutate_request__(&perm_string);

    return pass_to_c(&mod_string);
}



/*
PARAM 1: Request String from C
RETURN: Returns null-able pointer to a struct containing infromation returned from http request 
DESTRUCT FUNC: rdealloc_http_response_data
 */
#[no_mangle]
pub extern "C" fn send_com(request_s: *const c_char) -> Option<*mut HttpResponseDataC>
{
    if get_state() != STATE::READY
    {
        return None;
    }

    let reques_rs_s: String = read_immut_string_from_c(request_s);
    let response =  unsafe { __send_comm__ (&mut TLS_CLIENT.as_mut().unwrap(),reques_rs_s)};

    match response 
    {
        Ok(hrdc) => return Some(Box::into_raw(Box::new(hrdc))),
        Err(_) => return None
    };


}




    



// ======destruct

#[no_mangle]
pub extern "C" fn rdealloc_string(string: *mut c_char) -> ()
{
    unsafe{ CString::from_raw(string) };
}


#[no_mangle]
pub extern "C" fn rdealloc_http_response_data(obj: *mut HttpResponseDataC) -> ()
{
    unsafe { Box::from_raw(obj); } 
}

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