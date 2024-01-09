/* use std::ffi::CStr;
use std::ffi::c_char;
use std::ffi::CString;
use std::ptr::null;
use httparse::Response;

use crate::log::dbg_log_progress;


#[path ="err.rs"]
mod err;

pub static NULL_PTR: usize = 0x0;

//=== FFI structs to pass to C


















//===

 this is used to store strings to keep their lifetime going... dont see a way around this for now  
thread_local!(pub static FFI_RESPONSESTRING_BUFFER: RefCell<Vec<CString>> = RefCell::new(Vec::new()));
pub struct CFFIString(pub String);
impl CFFIString
{
    // add string to buffer 
    pub fn lend_immut_string2c(&self) -> *const c_char
    {
        let rf: &str = &(self).0;
        let cs_s: CString = match CString::new(rf)
        {
            Ok(s) => s,
            Err(_) => CString::new("").unwrap()
        };

        let mut len = 0;
        let mut ptr_bind = ptr::null();

        FFI_RESPONSESTRING_BUFFER.with( |text_cv: &RefCell<Vec<CString>> | 
            {
                (*text_cv.borrow_mut()).push(cs_s);
                len = (*text_cv.borrow_mut()).len();
                ptr_bind = (*text_cv.borrow_mut())[len - 1].as_ptr() as *const c_char;
            } ); 

        return ptr_bind;
    }


}



pub fn read_immut_string_from_c( request_string_ptr: *const c_char ) -> String
{
    let rust_request_c_ref: &CStr = unsafe { CStr::from_ptr(request_string_ptr) };
    return rust_request_c_ref.to_str().unwrap().to_string();
}

pub fn pass_to_c( string: &str ) -> *const c_char
{
    match CString::new(string)
    {
        Ok(cs) => return cs.into_raw(),
        Err(_) => return NULL_PTR as *const c_char
    };

}*/