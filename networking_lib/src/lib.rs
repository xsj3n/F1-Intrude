use core::panic;
use std::{ffi::{CStr, CString, c_char}, ptr, str::{from_utf8, from_boxed_utf8_unchecked}};
use std::sync::Mutex;
use err::{CResult, FFI_STRING_ERROR_C};
use ffi::from_c_string;
use libc::strlen;
use net_spx::{__start_com_cycle__, TlsClient, __send_comm__};
use parse_util::__permutate_request__;



mod net_spx;
mod parse_util;
mod err;
mod ffi;

/*
Error handling: 
*/
macro_rules! ret2c {
    ($type:ident) => 
    {
        Box::into_raw(Box::new($type))
    };
}

static mut TLS_CLIENT: Option<TlsClient> = None;
static mut RequestCache: String = String::new();

// This function will have to be called first
// This will pass the request to the gui for editing, and init the buffer which will be used as a cache to lessen passing accross ffi
#[no_mangle]
pub extern "C" fn ParseBurpRequestCache() -> *mut CString
{
    let rust_string = match parse_util::parse_burp_file()
    {
        Ok(s) => s,
        Err(_) => return ptr::null_mut()
    };



    let c_str = match CString::new(rust_string)
    {
        Ok(s) => s,
        Err(_) => return ptr::null_mut()
    };

    return ret2c!(c_str);

}


// iterate over permutations 
#[no_mangle]
pub extern "C" fn StartCOMCycle() -> CResult
{
    match __start_com_cycle__()
    {
        Ok(tc) => unsafe {TLS_CLIENT = Some(tc)},
        Err(e) => return CResult::new(true, e).unwrap()
    };

    return CResult::new(false, String::new()).unwrap();

}

#[no_mangle]
pub extern "C" fn send_com(request_s: *const c_char) -> CResult
{

    let reques_rs_s: String = match  unsafe { from_c_string(request_s) }
    {
        Ok(s) => s,
        Err(_) => return CResult::new(true, FFI_STRING_ERROR_C).unwrap()
    };


    let response = __send_comm__ (unsafe { &mut TLS_CLIENT.unwrap()} , reques_rs_s);
}

#[no_mangle]
pub extern "C" fn permutate_request(perm:*const c_char,) -> CResult
{
    let perm_string = match unsafe { from_c_string(perm) }
    {
        Ok(s) => s,
        Err(_) => return CResult::new(true, FFI_STRING_ERROR_C).unwrap()
    };


    return CResult::new(false, __permutate_request__(perm_string)).unwrap();
}

/*
June 1, 172; first surviving instance of megistou kai megistou
theou megalou Hermou
*/