use std::ffi::CStr;
use std::ffi::c_char;
use std::ffi::CString;
use httparse::Response;


#[path ="err.rs"]
mod err;

//=== FFI structs to pass to C
#[repr(C)]
pub struct HttpHeadersC
{ // holds pointers to immutable data passed to C
    header: [Option<*mut c_char>; 64],
    value:  [Option<*mut  c_char>; 64],
    init:   bool
}

#[repr(C)]
pub struct HttpResponseDataC
{
    headers: HttpHeadersC,
    body: Option<*mut c_char>,
    status_code: u16,
    total_response_bytes: u32 
}
//===
impl HttpHeadersC
{
    fn new() -> HttpHeadersC
    {
        const INIT: Option<*mut c_char> = None;
        let ptr_empty_init: [Option<*mut c_char>; 64] = [INIT; 64];
        return HttpHeadersC
        {
            header: ptr_empty_init,
            value: ptr_empty_init,
            init: false,
       };
    }

}


impl HttpResponseDataC
{
    pub fn new(response_tp: (Option<Response>, Option<String>) ) -> HttpResponseDataC
    {
 

        let r = ResponseFFITransformer(response_tp.0.unwrap());
        let code = r.0.code.unwrap();

        let mut http_response_data = HttpResponseDataC 
        {
            headers: r.transform(),
            body: None,  
            status_code: code,
            total_response_bytes: 0
        };

        match response_tp.1
        {
            Some(b) => 
            {
                http_response_data.total_response_bytes = b.len() as u32;

                let cs = CString::new(b)
                    .unwrap_or(CString::new("<FFI ERROR: NULL DETECTED IN BODY>").unwrap());
                
                http_response_data.body = Some(cs.into_raw());
            }
            None => ()
        }

        return http_response_data;
    }

}





pub struct ResponseFFITransformer<'h, 'b>(Response<'h, 'b>);
impl<'h, 'b> ResponseFFITransformer<'h, 'b>
{
    // ill check the results on cstring creations if it causes problems 
    fn transform(self) -> HttpHeadersC
    {   
        let mut http_struct = HttpHeadersC::new();
        
        // why would null bytes appear... right, right?!
        let mut i = 0;
        for h in self.0.headers
        {
            http_struct.header[i] = Some(pass_to_c(h.name).unwrap());
            http_struct.value[i] = Some(CString::from_vec_with_nul(h.value.to_vec())
            .unwrap().into_raw());
            i += 1;
        }

        return http_struct;

    }
}

/*  this is used to store strings to keep their lifetime going... dont see a way around this for now  
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
*/


pub fn read_immut_string_from_c( request_string_ptr: *const c_char ) -> String
{
    let rust_request_c_ref: &CStr = unsafe { CStr::from_ptr(request_string_ptr) };
    return String::from_utf8_lossy(rust_request_c_ref.to_bytes()).to_string();
}

pub fn pass_to_c( string: &str ) -> Option<*mut c_char>
{
    match CString::new(string)
    {
        Ok(cs) => return Some(cs.into_raw()),
        Err(_) => return None
    };

}