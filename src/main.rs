use std::ffi::{CString, c_char};

#[link(name="ninja",kind="static")]
unsafe  extern "C" {
    fn ninja_main(argc: std::ffi::c_int,argv: *const *const  std::ffi::c_char) -> std::ffi::c_int;
} 

pub fn main() {

    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect();
    
    let arg_ptrs: Vec<*const c_char> = args.iter()
        .map(|arg| arg.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();


    let result = unsafe {ninja_main(args.len().try_into().unwrap(), arg_ptrs.as_ptr())};
    std::process::exit(result);
}   