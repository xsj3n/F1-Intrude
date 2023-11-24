use std::{ffi::{CString, c_char}, os::raw::c_void, ptr, fmt::Display,};



pub const FFI_STRING_ERROR_C: &'static str = "Error converting C string to Rust lib";
pub const NULL_UTF8_ERROR: &'static str = "Null byte encountered in UTF8 translation";


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




#[repr(C)]
pub struct CResult
{
    pub error_present: bool,
    pub return_content: Option<*const c_void>
}


impl CResult
{
    pub fn new<T: Display>(err_present: bool, return_content: T) -> Result<CResult, &'static str>
    {
        let r_c_s = return_content.to_string();

        let cr = CResult
        {   
            error_present: err_present,
            return_content: match CString::new(r_c_s)
            {
             Ok(cs) => Some(cs.into_raw() as *const _) ,
             Err(_) => return Err("UTF8 FFI error")
            }
        };
        
        return Ok(cr);
    }
}