#[cfg(target_os = "windows")]
use std::path::PathBuf;
use std::{
    arch::asm,
    ffi::{CStr, CString, c_char, c_longlong},
    io::Read,
    os::raw::c_void,
    path::Path,
    ptr::NonNull,
};

thread_local! {
    // A 64KB reusable buffer that lives for the duration of the thread
    static READ_BUF: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::with_capacity(64 << 10));
    // holds the windows win32 file path resolved from the last call to its appropriate get_cwd. supports upto 32k characters
    // #[repr(align(2))]
    static WIN32_FS_CURDIR: std::cell::RefCell<Vec<u16>> = std::cell::RefCell::new(Vec::with_capacity(64 << 10));
    // holds the result of win32 chdir calls
    static WIN32_FS_CHDIR: std::cell::RefCell<Vec<u16>> = std::cell::RefCell::new(Vec::with_capacity(64 << 10));


}

// not that ninja was 'slow' here but they chose A variant functions that fail on long paths
// and prevent skia from building, which is the whole point at the time of this comment
// we also allocate a static heap buffer here because I believe that the buffer reuse will beat
// a 64kb buffer and it will grow if it needs to. this will technically allow arbitrary size file loads

// the caller must ensure that NonNull is upheld for raw performance. any of those ptrs CANNOT BE NULL
// err is not nonnull here yet because it may be init to null
pub fn read_file(path: &str, mut cb: impl FnMut(&mut Vec<u8>, usize)) -> Result<(), String> {
    // I decided to use canonicalize here because it handles \\?\\ on windows for the operating system
    // I will perf test this eventually, but the goal is to see how rust allows us to build skia on windows
    // if neccesary I will also test prepending with \\?\\ for long paths
    let mut resolved: String = String::with_capacity(path.len());
    #[cfg(windows)]
    {
        // attempt to smartly call ntdll if able
        if !unsafe { is_long_path_aware_runtime() } {
            unsafe { rs_absolute_path_win32_ntdll(&path) };
            WIN32_FS_CHDIR.with_borrow(|buf| {
                let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());

                resolved = String::from_utf16_lossy(&buf[..len]);
            });
        } else if path.len() > 260 {
            println!("WARN: NOT LONG PATH AWARE");
        }
    }

    let mut pathb = std::path::Path::new(path).to_path_buf();
    if pathb.is_relative() {
        pathb = match pathb.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return Err(format!(
                    "ReadFile Failed to canonicalize path {path} because: {e}"
                ));
            }
        };
    }
    let result = std::fs::OpenOptions::new().read(true).open(&pathb);
    let mut file = match result {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "ReadFile failed to open file after canonicalizing path because: {e}"
            ));
        }
    };
    // let mut buf = Vec::new();
    // let data = file.read_to_end(&mut buf).unwrap();
    // let len = buf.len();
    // cb(&mut buf, len);
    READ_BUF.with(|cell| {
        let mut buf = cell.borrow_mut();
        let cap = buf.capacity();
        unsafe { buf.set_len(cap) };
        // buf.fill(u8::MAX);
        loop {
            let r = file.read(&mut buf);
            // unsafe {buf.set_len(cap);}
            let bytes_read = match r {
                Ok(bytes_read) => bytes_read,
                Err(e) => {
                    println!("ReadFile failed to read chunk because {e}");
                    0
                }
            };
            // let dump_pos = buf.iter().position(|&b| b == 0xFF).unwrap_or(bytes_read);
            // unsafe {buf.set_len(bytes_read);}
            // println!("CHUNK {bytes_read}",);
            // hexdump_colon_highlight(&buf[0..dump_pos]);

            cb(&mut buf, bytes_read);
            if bytes_read == 0 {
                break;
            }
        }
    });

    Ok(())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_read_file(
    path: NonNull<c_char>,
    err: *mut *mut c_char,
    string_ctx: NonNull<c_void>,
    cb: unsafe extern "C" fn(NonNull<c_void>, NonNull<c_char>, c_longlong),
) {
    let path = unsafe { CStr::from_ptr(path.as_ptr() as *const _) };
    let path_str = match path.to_str() {
        Ok(p) => p,
        Err(e) => {
            if err.is_null() {
                // panic cannot be used here since ninja 'attmpts' to catch faults but crashes
                // unwinding across ffi is UB
                println!("RS FATAL ERROR. ReadFile recieved non utf8 path and err ptr is null.");
                std::process::exit(-1);
            }
            if !err.is_aligned() {
                println!(
                    "RS FATAL ERROR. ReadFile recieved an non utf8 path unalinged err ptr and the error cannot be written"
                );
                std::process::exit(-1);
            }
            let msg = format!("ReadFile: Path was not valid utf8 string. Error: {e}");
            // this used here because rust guarentees no nuls and valid utf8;
            let msg = unsafe { CString::from_vec_unchecked(msg.into_bytes()) };
            unsafe { *err = msg.into_raw() };
            return;
        }
    }.to_string();

    let r = read_file(&path_str, |data, len| {
        // rust must produce valid bytes here or all hell will break loose
        // not sure if rust will push zero here or not
        // data.push(0);
        // unchecked used here because rust 'should' guarentee nonnull from above
        // even though rust is u8, the bytes will be treated as i8, there may be data loss
        let nonnull: NonNull<i8> = unsafe { NonNull::new_unchecked(data.as_ptr() as *mut _) };
        unsafe { cb(string_ctx, nonnull, len as _) }
    });
    if let Err(e) = r {
        if err.is_null() {
            // panic cannot be used here since ninja 'attmpts' to catch faults but crashes
            // unwinding across ffi is UB
            println!("RS FATAL ERROR. ReadFile recieved non utf8 path and err ptr is null.");
            std::process::exit(-1);
        }
        if !err.is_aligned() {
            println!(
                "RS FATAL ERROR. ReadFile recieved an non utf8 path unalinged err ptr and the error cannot be written"
            );
            std::process::exit(-1);
        }
        let msg = unsafe { CString::new(e).unwrap_unchecked() };
        unsafe {
            *err = msg.into_raw();
        }
    }
}
#[cfg(target_os = "windows")]
#[link(name = "ntdll")]
unsafe extern "system" {
    // This is the "Native" version of chdir
    pub fn RtlSetCurrentDirectory_U(
        path: *const windows::Win32::Foundation::UNICODE_STRING,
    ) -> windows::Win32::Foundation::NTSTATUS;
    fn RtlGetFullPathName_U(
        FileName: *const u16,
        BufferLength: u32,
        Buffer: *mut u16,
        FileNamePart: *mut *mut u16,
    ) -> u32;
    fn RtlGetCurrentDirectory_U(BufferLength: u32, Buffer: *mut u16) -> u32;
    // Note: The 'str' version of the API takes the struct by pointer
    // fn RtlGetFullPathName_Ustr(
    //     FileName: *const windows::Win32::Foundation::UNICODE_STRING,
    //     StaticString: *mut windows::Win32::Foundation::UNICODE_STRING, // Optional internal buffer
    //     DynamicString: *mut windows::Win32::Foundation::UNICODE_STRING, // The one we want
    //     StringInDynamicString: *mut bool,
    //     FilePartPrefixCch: *mut u32,
    //     FileNameUsage: *mut u32,
    // ) -> i32; // Returns length in bytes
    pub fn RtlAreLongPathsEnabled() -> bool;
}
#[cfg(windows)]
// type RtlAreLongPathsEnabledFn = unsafe extern "system" fn() -> bool;
#[cfg(windows)]
pub unsafe extern "C" fn is_long_path_aware_runtime() -> bool {
    unsafe {
        // 1. Get a handle to ntdll.dll (already loaded in your process)

        // use windows::core::PCSTR;
        use windows::{Win32::System::LibraryLoader::GetModuleHandleA};
        if let Ok(h_ntdll) = GetModuleHandleA(windows::core::s!("ntdll.dll")) {
            // 2. Find the address of the "Long Path Aware" check

            use windows::Win32::System::LibraryLoader::GetProcAddress;
            if let Some(addr) = GetProcAddress(h_ntdll, windows::core::s!("RtlAreLongPathsEnabled"))
            {
                // 3. Transmute the raw pointer into a callable function
                // It returns a BOOLEAN (u8) where 0 is False and != 0 is True
                let rtl_are_long_paths_enabled: unsafe extern "system" fn() -> u8 =
                    std::mem::transmute(addr);

                return rtl_are_long_paths_enabled() != 0;
            }
        }
        // Fallback: If the function doesn't exist (older Windows), you're definitely not aware.
        false
    }
}
#[cfg(target_os = "windows")]
pub unsafe fn rs_absolute_path_win32_ntdll(path: &str) -> PathBuf {
    use std::os::windows::ffi::OsStrExt;

    let wide_input: Vec<u16> = std::ffi::OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // 2. Allocate a "Long Path" buffer (NT limit is ~32,767 chars)
    // let mut buffer = vec![0u16; 32768];
    let mut file_part: *mut u16 = std::ptr::null_mut();
    let s = WIN32_FS_CHDIR.with_borrow_mut(|buffer| {
        buffer.clear();
        unsafe {
            // 3. Call the NT Native engine
            let bytes_needed = RtlGetFullPathName_U(
                wide_input.as_ptr(),
                buffer.capacity() as _, // Length in bytes
                buffer.as_mut_ptr(),
                &mut file_part,
            );
            if bytes_needed == 0 {
                panic!("NT Native resolution failed!");
            }
            buffer.set_len(bytes_needed as _);
            // Return the raw wide string for direct API calls
            String::from_utf16(buffer.as_ref()).expect("ninja: fatal bad string data during rs_absolute_path_win32_ntdll")
        }
        
    });
    Path::new(&s).to_path_buf()
}
/// asks the NT subsystem what it believes the current directory is
/// attempts to bypass the win32 subsystem in case it has MAX PATH checks
/// you must also be long path aware as a best effort to use this function
/// be aware that this is not known to allow giant paths on its own
/// and may require tracking the current directory independantly of the operating system
pub fn rs_getcwd_ntdll() -> String {
    WIN32_FS_CURDIR.with_borrow_mut(|buffer| unsafe {
        buffer.clear();
        // buffer.reserve(512);
        let buffer_ptr = buffer.as_mut_ptr() as *mut u16;
        let buffer_len_bytes = buffer.capacity() as u32;

        let bytes_needed = RtlGetCurrentDirectory_U(buffer_len_bytes, buffer_ptr);

        if bytes_needed == 0 {
            return String::new();
        }
        buffer.set_len(bytes_needed as usize);
        let len_chars = (bytes_needed / 2) as usize;
        let wide_slice = std::slice::from_raw_parts(buffer_ptr, len_chars);
        // println!(
        //     "ptr{buffer_ptr:p} wide slice {wide_slice:?} buffer bytes: {}",
        //     &buffer[0..32]
        //         .iter()
        //         .map(|b| format!("{b:016b} "))
        //         .collect::<String>()
        // );
        String::from_utf16_lossy(wide_slice)
    })
}

#[cfg(target_os = "windows")]
pub unsafe fn rs_chdir_ntdll_longpath(path: &str) -> i32 {
    // use windows::core::PCWSTR;
    // use windows::Win32::System::WindowsProgramming::RtlSetCurrentDirectory_U;
    #[cfg(debug_assertions)]
    if path.len() > u16::MAX as usize {
        eprintln!(
            "Chdir WARN: Path > u16::MAX, path will appear truncated to ntdll and will not be able to see all of it"
        );
    }
    // todo verify number
    // #[cfg(debug_assertions)]
    else if path.len() > 32727 {
        eprintln!("Chdir WARN: path > ntdll max path support, will fail or crash");
    }

    unsafe {
        use windows::{Win32::Foundation::UNICODE_STRING, core::PWSTR};

        let mut wide_path: Vec<u16> = path.encode_utf16().collect();
        let bytes: &[u16] = wide_path.as_ref();
        let len = bytes.len() as u16;
        // 2. Direct jump into ntdll.
        // It ignores MAX_PATH and moves the process "anchor" directly.
        let unicode_string = UNICODE_STRING {
            Length: len,
            MaximumLength: wide_path.capacity() as _,
            Buffer: PWSTR(wide_path.as_mut_ptr().cast()),
        };
        let status = RtlSetCurrentDirectory_U(&raw const unicode_string);
        // println!("Chdir Win32 RtlSetCurrentDirectoryU return status {status:?}");
        status.is_ok() as _
    }
}

// SIGH. this function exists because of limitations inside the WIN32 api layer. in order to
// actually be able to handle long paths we have to bypass the win32 api compatibility layer and talk to the kernel directly
// this also supports linux in the same function, but for now calls rust's own chdir
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_chdir(path: NonNull<c_char>) -> bool {
    let result = unsafe { CStr::from_ptr(path.as_ptr() as *const _) }.to_str();
    let path = match result {
        Ok(p) => p,
        Err(e) => {
            println!("chdir recieved non utf8 input for path: {e}");
            return false;
        }
    };
    let mut pb = std::path::Path::new(path).to_owned();
    #[cfg(target_os = "windows")]
    {
        let is_long_path_aware = unsafe {
            is_long_path_aware_runtime()
        };
        if !is_long_path_aware {
            println!("WARN: Not path aware, may fail for {}", pb.display());
        }
        // this path was chosen to attempt to bypass the windows api limitations. may or may not be permanent,
        // but the real issues is that the PEB cannot hold long file paths
        unsafe { rs_absolute_path_win32_ntdll(pb.to_str().unwrap()) };
        let s = WIN32_FS_CHDIR.with_borrow(|buf| {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            let s = String::from_utf16_lossy(&buf[..len]);
            s
        });
        // 2. Convert from UTF-16 to a standard Rust String
        // "lossy" ensures that if there's an invalid sequence, it won't crash
        println!("ntdll resolved path as {s}");
        pb = std::path::Path::new(&s).to_path_buf()
    }

    if pb.is_relative() {
        let result = std::fs::canonicalize(&pb);
        pb = match result {
            Ok(p) => p,
            Err(e) => {
                println!("Failed to canonicalize path {e}, set curdir may fail");
                pb
            }
        };
    }
    #[cfg(windows)]
    {
        let is_long_path_aware = unsafe {
            is_long_path_aware_runtime()
        };
        if !is_long_path_aware {
            println!("CHDIR: WARN: Not path aware, may fail for {}", pb.display())
        }
    }
    match std::env::set_current_dir(&pb) {
        Ok(_) => return true,
        Err(e) => {
            println!("Chdir failed to change directory because {e}. trying for long paths if available");
        }
    }
    #[cfg(target_os = "windows")]
    {
        let path = pb.to_str().unwrap().replace("\\\\?\\", "\\??\\");
        return unsafe { rs_chdir_ntdll_longpath(&path) != 0 };
    }
    #[cfg(target_os = "linux")]
    {
        println!("TODO LINUX: CHDIR can we handle more than 4k chars?");
        return false;
    }
}
#[test]
#[cfg(target_os = "windows")]
pub fn test_win32_chdir_long() {
    if !unsafe { is_long_path_aware_runtime() } {
        panic!("the exe must be long path aware to run this test");
    }
    // let mut args = std::env::args();
    // let _ = args.next();
    // let n = args.next().unwrap();
    let limit = 30;
    // if n == "chdirlonglimit" {
    //     limit = args.next().unwrap_or("20".to_string()).parse().unwrap();
    // }
    println!("Testing up to {limit} iters of path");
    let mut input = create_essay_sized_directory(limit);
    input.push_str("../../../../../../");
    let path = std::ffi::CString::new(input).unwrap();
    // let path = c"C:\\skia_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa_aaaaaaaaaaa_aaaaaaaaaa_aaaaaaaaaa\\skiaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\\skia";
    let ptr = unsafe { NonNull::new_unchecked(path.as_ptr() as *mut _) };
    let b = unsafe { rs_chdir(ptr) };
    // assert_eq!(b, true);
    println!("is success {b}");
    // attempt to get the current dir
    println!("ntdll current dir {}", rs_getcwd_ntdll());

    let p = c"../ABC1234567890ABC123/ABC1234567890ABC123";
    let p = unsafe { NonNull::new_unchecked(p.as_ptr() as *mut _) };
    let b = unsafe { rs_chdir(p) };
    assert_eq!(b, true);
    let cur_dir = std::env::current_dir().unwrap();
    println!("cur_dir is {}", cur_dir.display());
}
// #[test]
#[cfg(test)]
fn create_essay_sized_directory(limit: usize) -> String {
    use std::fs;
    use std::path::PathBuf;

    // 1. Get an absolute path to a temp workspace
    let mut base_path = {
        if cfg!(target_os = "windows") {
            let cur_dir = rs_getcwd_ntdll();
            std::path::Path::new(&cur_dir).to_path_buf()
        } else {
            std::env::current_dir().unwrap()
        }
    };
    base_path.push("mithril_test_zone");
    println!("{}", base_path.display());

    // 2. Prepend the "Nuclear" Verbatim Prefix
    // This tells Windows to stop looking for a 260-char limit
    let mut long_path_str = format!(r"\\?\{}", base_path.display());

    // 3. Chain segments until we hit ~600-800 characters
    // Using 20-char segments to avoid individual filename limits (max 255)
    for _ in 0..limit {
        long_path_str.push_str(r"\ABC1234567890ABC123");
    }

    let final_path = PathBuf::from(&long_path_str);

    // 4. Create the directory tree
    // Rust's create_dir_all is "Handled with Love" and respects the prefix
    fs::create_dir_all(&final_path).expect("Failed to create the Essay-Sized path");

    // 5. Verification: Check if it exists
    assert!(
        final_path.exists(),
        "The path exists but Windows is lying to us"
    );

    println!("Mithril Path Created. Length: {}", long_path_str.len());
    println!("Path: {}", long_path_str);
    return long_path_str;
}
pub unsafe fn dump_peb_table() {
    // 1. Get PEB address from GS:[0x60]
    let mut peb_ptr: *const u8 = std::ptr::null_mut();
    unsafe {
        asm!(
            "mov {}, gs:[0x60]",
            out(reg) peb_ptr,
        );
    }

    println!("PEB Address: {:p}", peb_ptr);
    println!("Offset | 00 01 02 03 04 05 06 07 | ASCII");
    println!("-------|-------------------------|---------");

    // 2. Dump the first 64 bytes
    for i in (0..64).step_by(8) {
        let chunk = unsafe {std::slice::from_raw_parts(peb_ptr.add(i), 8)};

        // Hex part
        let hex = chunk
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join(" ");

        // ASCII part (to spot those leaked path fragments)
        let ascii = chunk
            .iter()
            .map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' })
            .collect::<String>();

        println!("0x{:02X}   | {} | {}", i, hex, ascii);
    }
}
// use std::arch::asm;

pub unsafe fn dump_process_parameters() {
    let peb_ptr: usize;

    // 1. Snatch the PEB address again
    unsafe {asm!("mov {}, gs:[0x60]", out(reg) peb_ptr)};

    // 2. Read the pointer at PEB + 0x20 (ProcessParameters)
    // We cast to *const usize to read the 8-byte address stored there
    let proc_params_ptr = unsafe {*((peb_ptr + 0x20) as *const usize) as *const u8};

    println!(
        "--- PEB -> ProcessParameters (0x{:X}) ---",
        proc_params_ptr as usize
    );
    println!("Offset | 00 01 02 03 04 05 06 07 | ASCII");
    println!("-------|-------------------------|---------");

    // 3. Dump the ProcessParameters structure
    // We go to 0x80 to ensure we hit the CURDIR structure at 0x38
    for i in (0..128).step_by(8) {
        let chunk = unsafe {
            std::slice::from_raw_parts(proc_params_ptr.add(i), 8)
            // testing123
        };

        let hex = chunk
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join(" ");

        let ascii = chunk
            .iter()
            .map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' })
            .collect::<String>();

        println!("0x{:02X}   | {} | {}", i, hex, ascii);
    }
}

#[test]
pub fn test_dump_peb_table() {
    let has_long_paths = unsafe { is_long_path_aware_runtime() };
    println!("Process has long paths enabled: {has_long_paths}");
    unsafe {
        dump_peb_table();
    }
    unsafe {
        dump_process_parameters();
    }
}

use std::fs::File;
use std::io::{BufWriter, Write};
// use std::path::Path;
#[allow(unused)]
fn generate_64kb_random_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
    const FILE_SIZE: usize = 64 * 1024 * 2; // 64KB
    let mut buffer = vec![0u8; FILE_SIZE];

    // Using the system's time as a very basic "poor man's seed"
    // for pseudo-randomness if you truly can't use the 'rand' crate.
    // Otherwise, for true random, you'd usually use `rand::thread_rng().fill()`.
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut x = seed;
    for i in 0..FILE_SIZE {
        // Simple Xorshift random number generator
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        buffer[i] = (x % 256) as u8;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&buffer)?;
    writer.flush()?;

    Ok(buffer)
}

#[cfg(windows)]
#[test]
pub fn test_fs_crud_longpath() {
    assert_eq!(unsafe { is_long_path_aware_runtime() }, true);
    let input = create_essay_sized_directory(50);
    std::fs::create_dir_all(&input).unwrap();
    let path = std::ffi::CString::new(input.as_bytes()).unwrap();
    // let path = c"C:\\skia_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa_aaaaaaaaaaa_aaaaaaaaaa_aaaaaaaaaa\\skiaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\\skia";
    let ptr = unsafe { NonNull::new_unchecked(path.as_ptr() as *mut _) };
    let b = unsafe { rs_chdir(ptr) };
    assert_eq!(b, true);
    let current_dir_string = rs_getcwd_ntdll();
    assert_eq!(current_dir_string, input);
    let current_dir = std::path::Path::new(&current_dir_string);
    const FILENAME: &str = "test_file.txt";
    let input_buf = generate_64kb_random_file(current_dir.join(FILENAME)).unwrap();
    let mut stdlib_readbuf = Vec::with_capacity(64 * 1024 * 2);
    std::fs::OpenOptions::new()
        .read(true)
        .open(current_dir.join(FILENAME))
        .unwrap()
        .read_to_end(&mut stdlib_readbuf)
        .unwrap();
    assert_eq!(input_buf, stdlib_readbuf);
    let mut rs_read_buf = Vec::with_capacity(64 << 10);
    let path = current_dir.join(FILENAME);
    read_file(path.to_str().unwrap(), |vec, _len| {
        rs_read_buf.append(&mut *vec);
    })
    .unwrap();
    assert_eq!(rs_read_buf, input_buf);

    std::fs::remove_file(current_dir.join(FILENAME)).unwrap();
}
// use std::io::{self, Write};

pub fn hexdump(buf: &[u8]) {
    let width = term_width().unwrap_or(120); // fallback if detection fails

    // Each byte takes "XX " = 3 chars, plus offset + spacing
    let offset_width = 8; // 8 hex digits
    let spacing = 2; // ": "
    let ascii_spacing = 3; // " | "
    let per_byte = 3;

    // Compute how many bytes per row fit
    let usable = width.saturating_sub(offset_width + spacing + ascii_spacing + 1);
    let bytes_per_row = (usable / per_byte).max(1);

    let mut stdout = std::io::stdout();

    for (i, chunk) in buf.chunks(bytes_per_row).enumerate() {
        let offset = i * bytes_per_row;

        // Print offset
        write!(stdout, "{:08x}: ", offset).unwrap();

        // Hex section
        for b in chunk {
            write!(stdout, "{:02x} ", b).unwrap();
        }

        // Padding if last line is short
        let missing = bytes_per_row - chunk.len();
        for _ in 0..missing {
            write!(stdout, "   ").unwrap();
        }

        // ASCII section
        write!(stdout, " | ").unwrap();
        for &b in chunk {
            let c = if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            };
            write!(stdout, "{}", c).unwrap();
        }

        writeln!(stdout).unwrap();
    }
}
fn term_width() -> Option<usize> {
    use terminal_size::{Width, terminal_size};

    terminal_size().map(|(Width(w), _)| w.saturating_sub(10) as usize)
}

pub fn hexdump_colon_highlight(buf: &[u8]) {
    const BYTES_PER_LINE: usize = 32;

    // ANSI colors
    const YELLOW: &str = "\x1b[33m";
    const RESET: &str = "\x1b[0m";

    for (i, chunk) in buf.chunks(BYTES_PER_LINE).enumerate() {
        let offset = i * BYTES_PER_LINE;

        // Offset
        print!("{:08x}: ", offset);

        // Hex section
        for &b in chunk {
            if b == b':' {
                print!("{}{:02x}{} ", YELLOW, b, RESET);
            } else {
                print!("{:02x} ", b);
            }
        }

        // Padding for short lines
        for _ in 0..(BYTES_PER_LINE - chunk.len()) {
            print!("   ");
        }

        print!(" | ");

        // ASCII section
        for &b in chunk {
            let ch = if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            };

            if b == b':' {
                print!("{}{}{}", YELLOW, ch, RESET);
            } else {
                print!("{}", ch);
            }
        }

        println!();
    }
}

#[test]
fn spawn_with_very_long_absolute_path() {
    // Build a very long path like:
    // \\?\C:\temp\aaaa...\aaaa\tool.exe
    assert_eq!(unsafe { is_long_path_aware_runtime() }, true);
    let mut path = String::from(r"\\?\C:\temp");
    // Make it absurdly long
    for _ in 0..200 {
        path.push_str(r"\aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    }
    std::fs::create_dir_all(&path).unwrap();
    unsafe { rs_chdir_ntdll_longpath(&path) };
    path.push_str(r"\tool.exe");

    println!("Attempting to spawn:\n{}", path);
    println!("Length: {}", path.len());

    let result = std::process::Command::new(&path).arg("test").spawn();

    match result {
        Ok(child) => {
            println!("Spawn succeeded unexpectedly: {:?}", child);
        }
        Err(e) => {
            println!("Spawn failed as expected: {}", e);
        }
    }

    // Don't assert success—we just want to observe behavior
    assert!(true);
}
