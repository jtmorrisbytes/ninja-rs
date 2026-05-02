use std::ffi::c_char;
#[unsafe(no_mangle)]
#[allow(non_snake_case,unsafe_op_in_unsafe_fn,unused_variables)]
pub unsafe extern "C" fn rs_clparser__filter_show_includes(path: *const c_char, prefix: *const c_char) -> *mut c_char{
    let p = std::ffi::CStr::from_ptr(path).to_string_lossy();
    let prefix = std::ffi::CStr::from_ptr(prefix).to_string_lossy();
    let out = p.strip_prefix(prefix.as_ref()) // 1. Matches prefix and "slides" forward
        .map(|rest| rest.trim_start()) // 2. Skips leading spaces
        .unwrap_or("");
    // println!("clparser_show_includes result:{out}");
    let r = std::ffi::CString::new(out).unwrap_or_else(|_|c"".to_owned());
    return r.into_raw();
    // println!("clparser show includes {p:?} {b:?}");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_clparser__parse(output: *const c_char, deps_prefix: *const c_char, filtered_output: *mut c_char, err:*mut c_char ) -> bool {
    let o = unsafe {std::ffi::CStr::from_ptr(output)};
    let d = unsafe {std::ffi::CStr::from_ptr(deps_prefix)};
    let f = unsafe {std::ffi::CStr::from_ptr(filtered_output)};
    let e= unsafe {std::ffi::CStr::from_ptr(err)};

    false
}