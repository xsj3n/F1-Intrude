/*
use std::{ffi::{CString, c_char}, os::raw::c_void, ptr, fmt::Display,};
use std::cell::RefCell;




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
    pub return_content: Option<*const c_void>,
    pub index: u32
}

thread_local!(pub static CR_BUFFER: RefCell<Vec<CString>> = RefCell::new(Vec::new()));
impl CResult
{
    pub fn new<'a> (err_present: bool, return_content: String) -> Result<CResult, &'static str>
    {

        let ptr_bind = CFFIString(return_content).lend_immut_string2c();
        
        let mut cr = CResult
        {   
            error_present: err_present,
            return_content: None,
            index: 0
        };
        

            
        cr.return_content = Some(ptr_bind as *const c_void);
        
        return Ok(cr);
    }
}
 */