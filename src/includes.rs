use std::{
    ffi::c_char,
    str::FromStr,
};

use crate::CUR_DIR;

pub fn win32_includes_normalized(
    input: &str,
    relative_to: &str,
    output: &mut String,
) -> Result<bool, String> {
    // let mut cannon_output = String::with_capacity(input.len());
    // unsafe {rs_canonicalize_path(input, &mut cannon_output)}

    let abs_input = match std::path::absolute(input) {
        Ok(p) => p,
        Err(e) => {
            return Err(format!(
                "IncludesNormalized failed to absolutize input path {input} because: {e}"
            ));
        }
    };
    let Ok(mut p_relative_to) = std::path::PathBuf::from_str(relative_to);

    // this is the most common case
    if relative_to.len() > 0 {
        match std::path::absolute(p_relative_to) {
            Ok(p) => p_relative_to = p,
            Err(e) => {
                return {
                    Err(format!(
                        "IncludesNormalize Failed to absolutize the relative_to_ variable because: {e}"
                    ))
                };
            }
        }
    } else {
        p_relative_to = match std::env::current_dir() {
            Ok(current_dir) => current_dir,
            Err(e) => {
                return Err(format!(
                    "IncludesNormalized failed to obtain the current directory because: {e}"
                ));
            }
        };
    }
    if p_relative_to == abs_input {
        output.push_str(CUR_DIR);
        return Ok(true);
    }
    let mut input_comps = abs_input.components();
    let mut base_comps = p_relative_to.components();
    let mut common_comps = 0;
    while let (Some(a), Some(b)) = (input_comps.next(), base_comps.next()) {
        // Windows: Case-insensitive comparison of components
        if a.as_os_str().to_string_lossy().to_lowercase()
            == b.as_os_str().to_string_lossy().to_lowercase()
        {
            common_comps += 1;
        } else {
            break;
        }
    }
    let base_remaining = p_relative_to.components().skip(common_comps).count();
    for _ in 0..base_remaining {
        output.push_str(crate::RHS_PARENT_DIR_STR);
    }
    let input_remaining: Vec<_> = abs_input
        .components()
        .skip(common_comps)
        .map(|c| c.as_os_str().to_string_lossy())
        .collect();
    output.push_str(&input_remaining.join(crate::RHS_PATH_SEP_STR));

    // println!("relativze: input {input:} output:{output} relative_to {relative_to:?}");
    Ok(true)
}

#[allow(unused_assignments)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_includes_normalize(
    input: *const c_char,
    relative_to: *const c_char,
    output: *mut *mut c_char,
    err: *mut *mut c_char,
) -> bool {
    let input = unsafe { std::ffi::CStr::from_ptr(input) }
        .to_str()
        .expect("Failed to convert input string");
    let relative_to = unsafe { std::ffi::CStr::from_ptr(relative_to) }
        .to_str()
        .expect("failed to convert relativeto");
    let mut rust_string = String::with_capacity(input.len());
    let b = match win32_includes_normalized(input, relative_to, &mut rust_string) {
        Ok(b) => b,
        Err(e) => {
            unsafe { *err = std::ffi::CString::new(e).unwrap().into_raw() };
            false
        }
    };
    let cstr = std::ffi::CString::new(rust_string)
        .expect("rust gave null string")
        .into_raw();
    unsafe { std::ptr::write(output, cstr) };
    // unsafe {*output = cstr};
    b
}
// #[test]
// pub fn normalize_relative_to(
//     input: &str,
//     relative_to: &str,
//     output: &mut String,
//     expected: &str,
//     expected_bool: bool,
// ) {
//     let b = win32_includes_normalized(input, relative_to, &mut *output);
//     assert_eq!(b, expected_bool);
//     assert_eq!(expected, output);
// }
// #[test]
// pub fn test_normalize() {}
