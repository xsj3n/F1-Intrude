#[path ="err.rs"]
mod err;

use std::ffi::CStr;
use std::ffi::c_char;
use std::ffi::CString;
use std::ptr;
use std::str::Utf8Error;
use crate::err::CResult;


macro_rules! ret2c {
    ($type:ident) => 
    {
        Box::into_raw(Box::new($type))
    };
}

pub fn to_c_string(rust_str: String) -> CResult
{


    let cr = match CResult::new(false, &rust_str)
    {
        Ok(c_r) => c_r,
        Err(e) => return   CResult::new(true, &String::new()).unwrap()    
    };

    return cr;
     
}

pub unsafe fn from_c_string(c_str_ptr: *const c_char) -> Result<String, u8>
{
    let cs = CStr::from_ptr(c_str_ptr);
    let rust_str = match cs.to_str()
    {
        Ok(s) => s,
        Err(e) => return Err(0)
    };

    return Ok(rust_str.to_owned().to_string())
}
