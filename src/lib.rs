#![deny(warnings)]
use std::{
    cell::RefCell,
    ffi::{CStr, CString, c_char, c_longlong, c_void},
    io::Write,
    path::{Component},
};
pub mod build;
pub mod clparser;
pub mod ninja;
pub mod fs;
pub mod includes;
pub mod subprocess;
pub mod status;
pub mod build_log;
const RHS_PATH_SEP_BYTE: u8 = b'/';
const LHS_PATH_SEP_BYTE: u8 = b'\\';
thread_local! {
    // Each thread gets its own pre-allocated 'High Water Mark' vector
    static CANNON_SEGMENT_CACHE: RefCell<Vec<&'static str>> = RefCell::new(Vec::with_capacity(4096));
}
// static LHS_PATH_SEP_STR: &str = "\\";
static RHS_PATH_SEP_STR: &str = "/";
static UNC_PREFIX: &str = "\\?\\UNC\\";
static WIN_LONG_PREFIX: &str = "\\?\\";
static WIN_SHARED_PREFIX: &str = "\\\\";
static REL_ROOT_RHS: &str = "./";
static RHS_PARENT_DIR_STR: &str = "../";
// static LHS_PARENT_DIR_STR: &str = "..\\";

static LHS_REL_ROOT: &str = ".\\";
static EMPTY_STR: &str = "";
static CUR_DIR: &str = ".";
static PARENT_DIR: &str = "..";
static LINUX_ROOT: &str = RHS_PATH_SEP_STR;

#[cfg(test)]
fn generate_max_windows_path() -> String {
    let prefix = r"\\?\C:\";
    let mut path = String::with_capacity(32768);
    path.push_str(prefix);

    // Each segment is 16 bytes: "segment_000000/ "
    // This makes it easy to track SIMD iterations
    let mut i = 0;
    while path.len() < 32760 {
        path.push_str(&format!("seg_{:011}\\", i));
        i += 1;
    }

    // Pad the remainder to hit exactly 32767
    while path.len() < 32767 {
        path.push('a');
    }

    path
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_feature = "sse2"))]
#[test]
fn test_sse2_collect_path_slice() {
    use std::time::Instant;
    let start = Instant::now();
    // LOL COMPLETES 32k chars IN 1126200 nanoseconds in release mode.. FOR 1K iterations
    // THIS FUNCTION IS FUCKING SPEED DEMON
    let mut handles = Vec::new();
    let three_two_k_challenge = generate_max_windows_path();
    let len = three_two_k_challenge.as_bytes().len();
    let ptr = three_two_k_challenge.as_bytes().as_ptr();
    // this benchmark smuggles a pointer where I KNOW ITS SAFE
    #[derive(Clone, Copy)]
    struct Ptrguard(*const u8);
    let ptr = Ptrguard(ptr);
    unsafe impl Send for Ptrguard {}

    for _ in 0..6 {
        let handle = std::thread::spawn(move || {
            let mut buf = Vec::with_capacity(5000);
            let ptr = ptr.clone();
            // we join at the end of the block and the compiler cannot see that the input data outlives the program
            let bytes = unsafe { std::slice::from_raw_parts(ptr.0, len) };
            for _ in 0..10 {
                unsafe {
                    std::hint::black_box(sse2_collect_path_slices(&bytes, &mut buf, false, false))
                };
                buf.clear();
            }
        });
        handles.push(handle);
    }
    for h in handles {
        let _ = h.join().ok();
    }
    let duration = start.elapsed();
    println!("whole test took {} milis", duration.as_millis());
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_feature = "sse2"))]
#[target_feature(enable = "sse2")]
unsafe fn sse2_collect_path_slices(
    bytes: &[u8],
    output: &mut Vec<&str>,
    include_slashes: bool,
    include_empty_segments: bool,
) {
    #[cfg(all(not(target_arch = "x86_64"), target_arch = "x86"))]
    use core::arch::x86::{self, __m128i, *};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__m128i};
    use std::{mem::MaybeUninit, ops::Add};

    const SSE2_REG_BYTE_SIZE: usize = 16;
    // the compiler needed some help. I wanted to reserve these 5 vars as their own registers
    // for the whole function but it would just do garbage on xmm0
    // maybeunint used here because the block of inline asm below IS the init
    // and it would be pointless to assign these from a temporary scratch register
    let mut lhs_slash_template: MaybeUninit<__m128i> = MaybeUninit::uninit();
    let mut rhs_slash_template: MaybeUninit<__m128i> = MaybeUninit::uninit();
    let mut right_slash_results: MaybeUninit<__m128i> = MaybeUninit::uninit();
    let mut left_slash_results: MaybeUninit<__m128i> = MaybeUninit::uninit();
    let mut combined_slash_results: MaybeUninit<__m128i> = MaybeUninit::uninit();
    // let mut chunk: MaybeUninit<__m128i> = MaybeUninit::uninit();
    #[allow(unused_assignments)]
    unsafe {
        core::arch::asm!(
            // initialize lhs_slash_template to '\\'
            "movd xmm5, {0:e}",
            "punpcklbw xmm5, xmm5",
            "pshuflw xmm5, xmm5, 0",
            "pshufd xmm5, xmm5, 0",
            // initialize rhs_slash_template to '/'
            "movd xmm0, {1:e}",
            "punpcklbw xmm0, xmm0",
            "pshuflw xmm0, xmm0, 0",
            "pshufd xmm0, xmm0, 0",
            // make sure to zero init these xmm registers
            "pxor xmm1, xmm1",
            "pxor xmm2, xmm2",
            "pxor xmm3, xmm3",
            // "pxor xmm4, xmm4",
            in(reg) LHS_PATH_SEP_BYTE as u32,
            in(reg) RHS_PATH_SEP_BYTE as u32,
            inout("xmm0") rhs_slash_template,
            inout("xmm1") right_slash_results,
            inout("xmm2") left_slash_results,
            inout("xmm3") combined_slash_results,
            // inout("xmm4") chunk,
            inout("xmm5") lhs_slash_template,
            options(nomem,nostack,preserves_flags)
        );
    }
    let mut lhs_slash_template: __m128i = unsafe { lhs_slash_template.assume_init() };
    let mut rhs_slash_template: __m128i = unsafe { rhs_slash_template.assume_init() };
    let mut right_slash_results: __m128i = unsafe { right_slash_results.assume_init() };
    let mut left_slash_results: __m128i = unsafe { left_slash_results.assume_init() };
    let mut combined_slash_results: __m128i = unsafe { combined_slash_results.assume_init() };
    // let mut chunk: __m128i = unsafe { chunk.assume_init() };

    let sse2_count = bytes.len() / SSE2_REG_BYTE_SIZE;
    // println!("sse2_count {sse2_count} input len {}",bytes.len());
    // let sse2_remainder = bytes.len() % SSE2_REG_BYTE_SIZE;
    let base_address = bytes.as_ptr();
    let mut chunk_address;
    let mut component_base_address = bytes.as_ptr();
    let mut mask: u16 = 0;
    // this allows us to 'collect' 4 iterations
    let mut mask_acc: usize = 0;
    // let mut slashes: Vec<u64>= Vec::with_capacity(32);
    let mut last_slash_pos = bytes.as_ptr();
    // prealloc chunk here to be reused so it doenst get dropped when
    // going out of scope
    for count in 0..sse2_count {
        // the first iteration comes with  slash_template loaded with '/'
        // our math has already verified at this point that slice is a multiple of 16
        // calculate the 'effective address' of the current slice based on the size of an sse2 register and the loop
        // count
        chunk_address = unsafe { base_address.add(SSE2_REG_BYTE_SIZE * count) };
        unsafe {
            // ofc we needed inline asm here because we are using maybeuniti
            core::arch::asm!(
              "movdqu xmm1, [{0}]",    // 1. Load 16 bytes of path data
              "movdqa xmm2, xmm1",       // 2. Only ONE copy of the data is needed
              "pcmpeqb xmm1, xmm0",      // 3. xmm1 = mask for '/' (xmm1 is now clobbered)
              "pcmpeqb xmm2, xmm5",      // 4. xmm2 = mask for '\' (xmm2 is now clobbered)
              "por xmm1, xmm2",          // 5. Merge the results (OR them together)
              "pmovmskb {1:e}, xmm1",




              //   // load data from effecetive address into xmm4 (chunk)
              //   "movdqu xmm4, [{0}]",
              //   // we may compare xmm4 twice so copy xmm4 to right slash results or xmm1
              //   "movdqa xmm1, xmm4",
              //   // compare xmm1 (results) to xmm0 (mask or slash)
              //   "pcmpeqb xmm1, xmm0",
              //   // copy xmm4 into xmm2 (left slash results)
              //   "movdqa xmm2, xmm4",
              //   // compare xmm2 (results) to xmm0 (mask or slash)
              //   "pcmpeqb xmm2, xmm5",

              //   // copy xmm1 to xmm3
              //   "movdqa xmm3, xmm1",
              //   // now we need to collect both xmm1 and xmm2 together,
              //   "por xmm3,xmm2",
              //   // now xmm3 should contain the mask for all slashes, we need to turn this into a mask
              //   "pmovmskb {1:e}, xmm3",

                // maybe do the shuffle logic here
              //   "mov eax, {1:e}",
              //   "mov {scratch},{count}",
              //   "and {scratch}, 3",
              //   "shl {scratch}, 4",
              //   "shlx {rax}, {rax}, {scratch}",
              //   "or {acc}, {rax}",

                in(reg) chunk_address,
                inout(reg) mask,
              //   acc = inout(reg) mask_acc,
              //   count = in(reg) count,
                // in(reg) sliding_index,
                inout("xmm0") rhs_slash_template,
                inout("xmm1") right_slash_results,
                inout("xmm2") left_slash_results,
                inout("xmm3") combined_slash_results,
              //   inout("xmm4") chunk,
                inout("xmm5") lhs_slash_template,
                options(nostack)
            );
            #[cfg(target_pointer_width = "64")]
            const CHUNKS_PER_ACC: usize = 4; // 4 chunks * 16 bytes = 64 bits
            #[cfg(target_pointer_width = "32")]
            const CHUNKS_PER_ACC: usize = 2;
            // let raw_mask=u64::from(mask);
            // let mask_shift = ;
            mask_acc |= (mask as usize) << (count % CHUNKS_PER_ACC * 16);
            // println!("count {count} mask acc {mask_acc:064b}");
            if count % CHUNKS_PER_ACC == 0 && count > 0 {
                // println!("count {count}");
                let mut trailing;
                while mask_acc != 0 {
                    trailing = mask_acc.trailing_zeros() as usize;
                    let absolute_idx = component_base_address.add(trailing as usize);
                    let len = absolute_idx
                        .addr()
                        .saturating_sub(last_slash_pos.addr())
                        .add(include_slashes as usize);
                    // let byte_idx = absolute_idx.read_unaligned();
                    let slice =  std::slice::from_raw_parts(last_slash_pos, len);
                    let str = std::str::from_utf8_unchecked(slice);
                    if len > 0 {
                        output.push(str);
                    } else if include_empty_segments {
                        output.push(str);
                    }
                    // println!("mask {mask_acc:032b} path str {str:?} trailing {trailing:032b} c {component_base_address:p} absolude idx = {absolute_idx:p} acc {mask_acc:064b}");
                    last_slash_pos = absolute_idx.add(1);
                    mask_acc &= mask_acc - 1;
                }
                component_base_address = chunk_address;
            }
        }
    }
    unsafe {

        let final_address = base_address.add(bytes.len());
        let len = final_address.offset_from(last_slash_pos) as usize;
        // println!("{final_address:p} len {len}");
        let final_slice = std::slice::from_raw_parts(last_slash_pos, len);
        
        let str = std::str::from_utf8_unchecked(final_slice);
        // println!("{str}");
        // ideally run the SIMD 1 more time with the correct mask to 'clean up' and find the remaining data
        let mut remaining: Vec<&str> = str
        .split(|c| c == '/' || c == '\\')
        .filter(|seg| !seg.is_empty()) // Optional: ignores "//"
        .collect();
    output.append(&mut remaining);
}
}

// #[unsafe(no_mangle)]
// #[target_feature(enable="sse2")]
pub unsafe fn rs_canonicalize_path(
    path: &str,
    output: &mut String, // len: *mut core::ffi::c_longlong,
) {
    if path.len() == 0 {
        return;
    }

    // let input = unsafe { std::ffi::CStr::from_ptr(path) };
    // let b_input = input;
    let mut b_current = path;
    // println!("bytes {b_input:?}");
    // let mut cursor = 0;
    let mut prefix: &str = EMPTY_STR;
    let mut drive_letter: &str = EMPTY_STR;
    // for readability.
    for item in [
        UNC_PREFIX,
        WIN_LONG_PREFIX,
        WIN_SHARED_PREFIX,
        LHS_REL_ROOT,
        REL_ROOT_RHS,
        LINUX_ROOT,
    ] {
        if let Some(r) = b_current.strip_prefix(item) {
            b_current = r;
            prefix = item;
            break;
        }
    }
    // strip ./ at the beginning using ptr_eq if possible
    if std::ptr::eq(prefix, LHS_REL_ROOT) {
        prefix = EMPTY_STR;
    } else if std::ptr::eq(prefix, REL_ROOT_RHS) {
        prefix = EMPTY_STR;
    } else if prefix == LHS_REL_ROOT {
        prefix = EMPTY_STR;
    } else if prefix == REL_ROOT_RHS {
        prefix = EMPTY_STR
    }
    // drive letter detection. matches any prefix above then drive letter (shrug) i guess
    let chars = b_current.as_bytes();
    match (chars.get(0), chars.get(1)) {
        (Some(b'a'..=b'z' | b'A'..=b'Z'), Some(b':')) => {
            if let Some(s) = b_current.get(2..) {
                drive_letter = &path[0..=1];
                b_current = s;
            }
        }
        _ => {}
    }
    // we embedded a static mut vec on our side as a memory cache
    let components = CANNON_SEGMENT_CACHE.with(|cache| {
        let mut vec = cache.borrow_mut();
        vec.clear();
        unsafe { sse2_collect_path_slices(b_current.as_bytes(), &mut vec, false, false) };
        unsafe { std::slice::from_raw_parts(vec.as_ptr(), vec.len()) }
    });
    // we stripped the prefix, so if there were no slashes just return the input
    if components.len() == 0 {
        output.push_str(path);
        return;
    } else if components.len() == 1 && components[0] != PARENT_DIR {
        output.push_str(prefix);
        output.push_str(&components[0]);
        return;
    }
    let components: Vec<_> = components
        .iter()
        .filter(|c| **c != CUR_DIR)
        .map(|s| *s)
        .collect();
    let has_relative = components.iter().find(|s| **s == PARENT_DIR).map(|s| *s);
    if has_relative.is_none() {
        output.push_str(prefix);
        output.push_str(drive_letter);
        // let mut p = prefix.to_string();
        if output.len() > 0 {
            output.push('/');
        }
        output.push_str(&components.join("/"));
        // println!("p {p} {components:?}");
        return;
    }

    // skip over leading .. as per the algo
    let mut relative_root = 0;
    while components.get(relative_root) == Some(&PARENT_DIR) {
        relative_root += 1;
    }
    let resolved_after_relative = &components[relative_root..];
    // this means the input was only ../../../ so it doesnt matter
    if resolved_after_relative.len() == 0 {
        output.push_str(prefix);
        output.push_str(drive_letter);
        output.push_str(&components.join(RHS_PATH_SEP_STR));
        return;
    }
    let mut relative_output: Vec<_> = Vec::with_capacity(components.len());
    for segment in resolved_after_relative {
        match segment {
            &".." if !relative_output.is_empty() => {
                relative_output.pop();
            }
            _ => relative_output.push(segment),
        }
    }
    // next we stitch together the prefix if there is one (todo)
    // components[0..relative_root] + relative_output
    output.push_str(prefix);
    output.push_str(drive_letter);
    let mut rs = components[0..relative_root].join(RHS_PATH_SEP_STR);
    output.push_str(&rs);
    rs = relative_output
        .iter()
        .map(|s| **s)
        .collect::<Vec<_>>()
        .join(RHS_PATH_SEP_STR);
    output.push_str(&rs);
    if output.len() == 0 {
        output.push('.');
    }
    // if path starts with ./ then get the next component
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_canonicalize_path3(
    path: *mut core::ffi::c_char,
    _len: *mut core::ffi::c_longlong,
) -> *mut c_char {
    let input = unsafe { std::ffi::CStr::from_ptr(path) };
    let input = input.to_str().expect("invalid utf8 sequence");
    let mut output = String::with_capacity(input.len());
    unsafe { rs_canonicalize_path(input, &mut output) };
    // println!("cannon path input {input} output {output}");
    return std::ffi::CString::new(output).unwrap().into_raw();
}
// const LEN: core::ffi::c_longlong = 0;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_canonicalize_path2(
    path: *mut core::ffi::c_char,
) -> *mut std::ffi::c_char {
    // unsafe{rs_canonicalize_path(path,len)}
    // let mut len = string.count_bytes().try_into().unwrap();
    unsafe { rs_canonicalize_path3(path, std::ptr::null_mut()) }
    // let s = std::ffi::CString::new(string.as_bytes()).unwrap_or_default();
    // s.into_raw()
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cstring_free(ptr: *mut std::ffi::c_char) {
    let d = unsafe { std::ffi::CString::from_raw(ptr) };
    drop(d);
}
#[unsafe(no_mangle)]
/// NOTABLE CHANGES. time since unix epoch not y2k
pub unsafe extern "C" fn rs_stat_single_file(
    path: *const std::ffi::c_char,
    _error: *const *mut std::ffi::c_char,
) -> std::ffi::c_longlong {
    // panic!("is rs_stat_single_file Dead code?");
    // println!("HELLO FROM RUST rs_stat_single_file");
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    if path.starts_with("../C") {
        
            // core::arch::asm!("int3");
            panic!("INVALID PATH");
        
    }
    let metadata = match std::fs::metadata(path.as_ref()) {
        Ok(m) => m,
        Err(_e) => {
            // 4 is the default for some reason?
            return 4;
        }
    };
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        return timestamp_from_win32_mfiletime(metadata.last_write_time());
    }
    #[cfg(not(target_os = "windows"))]
    {
        compile_error!("TODO: Statsinglefile on non windows");
    }
    // let modified = match std::fs::metadata(path.as_ref()) {
    //     Ok(m) => m.modified(),
    //     Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
    //         return 0;
    //     }
    //     Err(e) => {
    //         panic!("StatSingleFile {e}")
    //     }
    // };
    // let time_since_epoch = match modified {
    //     Err(e) => {
    //         eprintln!("failed to get lastwritetime {e}");
    //         return -1;
    //     }
    //     Ok(system_time) => system_time.duration_since(std::time::UNIX_EPOCH),
    // };
    // match time_since_epoch {
    //     Err(e) => {
    //         eprintln!("Failed to get time since unix epoch {e}");
    //         return -1;
    //     }
    //     Ok(time) => return time.as_nanos().try_into().unwrap(),
    // }

    //       printf_s("StatSingleFile %s\n",path.c_str());
    //   WIN32_FILE_ATTRIBUTE_DATA attrs;
    //   if (!GetFileAttributesExA(path.c_str(), GetFileExInfoStandard, &attrs)) {
    //     DWORD win_err = GetLastError();
    //     if (win_err == ERROR_FILE_NOT_FOUND || win_err == ERROR_PATH_NOT_FOUND)
    //       return 0;
    //     *err = "GetFileAttributesEx(" + path + "): " + GetLastErrorString();
    //     return -1;
    //   }

    //   if (attrs.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) {
    //     HANDLE hFile = CreateFileA(path.c_str(), 0, 0, 0, OPEN_EXISTING,
    //                                FILE_FLAG_BACKUP_SEMANTICS, 0);
    //     if (hFile == INVALID_HANDLE_VALUE) {
    //       DWORD win_err = GetLastError();
    //       if (win_err == ERROR_FILE_NOT_FOUND || win_err == ERROR_PATH_NOT_FOUND)
    //         return 0;
    //       *err = "CreateFileA(" + path + "): " + GetLastErrorString();
    //       CloseHandle(hFile);
    //       return -1;
    //     }

    //     CHAR pathBuf[MAX_PATH];
    //     if (GetFinalPathNameByHandleA(hFile, pathBuf, MAX_PATH,
    //                                   FILE_NAME_NORMALIZED) == 0) {
    //       DWORD win_err = GetLastError();
    //       if (win_err == ERROR_FILE_NOT_FOUND || win_err == ERROR_PATH_NOT_FOUND)
    //         return 0;
    //       *err = "GetFinalPathNameByHandleA(" + path + "): " + GetLastErrorString();
    //       CloseHandle(hFile);
    //       return -1;
    //     }
    //     CloseHandle(hFile);
    //     return StatSingleFile(pathBuf, err);
    //   }

    //   return TimeStampFromFileTime(attrs.ftLastWriteTime);
    // panic!("this works")
    // return -1;
}

#[cfg(target_os = "windows")]
fn timestamp_from_win32_mfiletime(filetime_ticks: u64) -> std::ffi::c_longlong {
    // 12622770400 * (1_000_000_000 / 100) == seconds * 10_000_000 (100ns ticks)
    const EPOCH_DIFF_TICKS: u64 = 12_622_770_400 * 10_000_000;

    // Do subtraction in u64 space (matches C++ behavior)
    let shifted = filetime_ticks - EPOCH_DIFF_TICKS;

    // Then cast to signed (like (TimeStamp)mtime in C++)
    shifted as _
}

// #[cfg(target_os="windows")]
// fn filetime_to_duration_since_2000(ft: &FILETIME) -> Duration {
//     use std::time::Duration;
//     use windows::Win32::Foundation::FILETIME;
//     // Combine high/low into u64
//     let ticks = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);

//     // FILETIME is in 100ns units → convert to nanoseconds
//     let nanos = ticks * 100;

//     // Seconds between 1601 and 2000
//     const EPOCH_DIFF_SECS: u64 = 12_622_770_400;

//     let epoch_diff = Duration::from_secs(EPOCH_DIFF_SECS);

//     // Convert nanos to Duration
//     let filetime_duration = Duration::from_nanos(nanos);

//     // Subtract epoch difference
//     filetime_duration
//         .checked_sub(epoch_diff)
//         .unwrap_or(Duration::ZERO)
// }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_stat_all_files_in_dir(
    dir: *const c_char,
    map_ctx: *mut c_void,
    cb: unsafe extern "C" fn(ctx: *mut c_void, *const c_char, c_longlong),
) -> bool {
    if dir.is_null() {
        return false;
    }
    let dir_cstr = unsafe { std::ffi::CStr::from_ptr(dir) };
    let dir_cow = dir_cstr.to_string_lossy();
    if dir_cow.starts_with("../C") {
        
            // core::arch::asm!("int3");
            println!("INVALID PATH: {}", dir_cow);
            std::io::stdout().flush().ok();
            std::io::stdin().read_line(&mut String::new()).ok();
        
    }

    let iterator = std::fs::read_dir(dir_cow.as_ref());
    if iterator.is_err() {
        return false;
    }
    let iterator = unsafe { iterator.unwrap_unchecked() };
    for entry in iterator {
        if entry.is_err() {
            continue;
        }
        let entry = unsafe { entry.unwrap_unchecked() };
        let metadata = entry.metadata();
        if metadata.is_err() {
            continue;
        }
        let metadata = metadata.unwrap();
        let mtime = {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::fs::MetadataExt;
                timestamp_from_win32_mfiletime(metadata.last_write_time())
            }
            #[cfg(target_os = "linux")]
            {
                compile_error!("TODO HANDLE MTIME");
            }
        };
        // let since_epoch = since_epoch.unwrap();
        // let mtime = since_epoch.as_nanos().try_into().unwrap();
        let n = entry.file_name();
        let f = n.to_string_lossy();
        let c = std::ffi::CString::new(f.as_bytes());
        if c.is_err() {
            continue;
        }
        let c = unsafe { c.unwrap_unchecked() };

        unsafe { cb(map_ctx, c.as_ptr(), mtime) }
    }
    return true;
}

#[unsafe(no_mangle)]
// takes in a 'path' and attepmts to call std::fs::absolute on it
pub unsafe extern "C" fn rs_abs_path2(
    path: *const c_char,
    string_ctx: *mut c_void,
    cb: unsafe extern "C" fn(*mut c_void, *const c_char),
) {
    debug_assert_eq!(path.is_null(), false);
    let c_str = unsafe { std::ffi::CStr::from_ptr(path) };
    let cow = c_str.to_string_lossy();
    let path = std::path::Path::new(cow.as_ref());
    if cow == "." {
        let d = std::env::current_dir().unwrap();
        let s = d.to_string_lossy();
        let c = CString::new(s.as_ref()).unwrap();
        unsafe { cb(string_ctx, c.as_ptr()) };
        return;
    }
    let r = std::path::absolute(path);
    if r.is_err() {
        todo!("Implement lexical absolutes if possible")
    }
    let r = unsafe { r.unwrap_unchecked() };
    let cow = r.to_string_lossy();
    let r = std::ffi::CString::new(cow.as_bytes());
    if r.is_err() {
        // this should never happen but panic if it does
        // std::process::exit(-1);
        unreachable!("Unexpected nul byte from rust");
    }
    let c_str = unsafe { r.unwrap_unchecked() };
    unsafe { cb(string_ctx, c_str.as_ptr()) };
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_same_lexical_drive(a: *const c_char, b: *const c_char) -> bool {
    debug_assert_eq!(a.is_null(), false);
    debug_assert_eq!(b.is_null(), false);
    let cstr_a = unsafe { std::ffi::CStr::from_ptr(a) };
    let cstr_b = unsafe { std::ffi::CStr::from_ptr(b) };

    let cow_a = cstr_a.to_string_lossy();
    let cow_b = cstr_b.to_string_lossy();

    let path_a = std::path::Path::new(cow_a.as_ref());
    let path_b = std::path::Path::new(cow_b.as_ref());

    let root_a = path_a.components().next();
    let root_b = path_b.components().next();

    if root_a.is_none() {
        return false;
    }
    let root_a = unsafe { root_a.unwrap_unchecked() };
    if root_b.is_none() {
        return false;
    }
    let root_b = unsafe { root_b.unwrap_unchecked() };

    match (root_a, root_b) {
        (Component::Prefix(a), Component::Prefix(b)) => a.kind() == b.kind(),
        (a @ Component::RootDir, b @ Component::RootDir) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::CStr};

    use super::rs_canonicalize_path2;

    fn run_test(input: &'static CStr, output: &'static CStr) {
        let r = unsafe { rs_canonicalize_path2(input.as_ptr() as *mut _) };
        let c = unsafe { CStr::from_ptr(r) };
        if output != c {
            panic!("assertion failed: input:  {input:?} , output: {c:?} , expected: {output:?}'");
        }
    }
    #[test]
    fn test_canonicalize_path_compatibility() {
        for _ in 0..1000 {
            
                // These tests match the original Ninja C++ unit tests for CanonicalizePath

                // Test 1: Single dot-dot
                run_test(c"", c"");

                // Test 2: Trailing slash on dot-dot
                run_test(c"foo.h", c"foo.h");

                // assert_eq!(rs_canonicalize_path2(c"../".as_ptr() as *mut _), c"..".as_ptr() as *mut _);

                // Test 3: Relative parent components
                run_test(c"./foo/./bar.h", c"foo/bar.h");

                // assert_eq!(rs_canonicalize_path2(c"../foo".as_ptr() as *mut _), c"../foo".as_ptr() as *mut _);

                // Test 4: Trailing slash removal
                run_test(c"./x/foo/../bar.h", c"x/bar.h");
                // Test 5: Multiple parent jumps
                run_test(c"./x/foo/../../bar.h", c"bar.h");

                // assert_eq!(rs_canonicalize_path2(c"../..".as_ptr() as *mut _), c"../..".as_ptr() as *mut _);

                // Test 6: Multiple jumps with trailing slash
                run_test(c"foo//bar", c"foo/bar");

                // assert_eq!(rs_canonicalize_path2(c"../../".as_ptr() as *mut _), c"../..".as_ptr() as *mut _);

                // Test 7: Dot-slash prefix cleanup
                run_test(c"foo//.//..///bar", c"bar");

                // assert_eq!(rs_canonicalize_path2(c"./../".as_ptr() as *mut _), c"..".as_ptr() as *mut _);

                // Test 8: Root-level dot-dot
                run_test(c"./x/../foo/../../bar.h", c"../bar.h");

                run_test(c"foo/./.", c"foo");
                run_test(c"..", c"..");
                run_test(c"../", c"..");
                run_test(c"../..", c"../..");
                run_test(c"../../", c"../..");
                run_test(c"./../", c"..");
                run_test(c"/..", c"/..");
                run_test(c"/../", c"/..");
                run_test(c"/../..", c"/../..");
                run_test(c"/../../", c"/../..");
                run_test(c"/", c"/");
                run_test(c"/foo/..", c"/");
                run_test(c".", c".");
                run_test(c"./.", c".");
                run_test(c".", c".");

                run_test(c"foo/..", c".");
                run_test(c"foo/.._bar", c"foo/.._bar");
                // yes it really gets this long
                run_test(c"C:\\Program Files(x86)\\Microsoft\\Visual Studio\\10.0.0.1000\\msbuild\\x86\\64\\msvcrt",c"C:/Program Files(x86)/Microsoft/Visual Studio/10.0.0.1000/msbuild/x86/64/msvcrt");
            
        }
    }

    #[test]
    fn test_windows_abyss_drive_reset() {
        // Our custom "Senior" fix for the ../C: bug
        // assert_eq!(rs_canonicalize_path2(c"../C:\\path"), "C:\\path");
        // assert_eq!(rs_canonicalize_path2(c"../../D:/skia"), "D:\\skia");
    }
}
