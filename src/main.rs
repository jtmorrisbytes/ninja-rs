use std::ffi::{CString, c_char};

use clap::Parser;

#[link(name = "ninja", kind = "static")]
unsafe extern "C" {
    fn ninja_main(argc: std::ffi::c_int, argv: *const *const std::ffi::c_char) -> std::ffi::c_int;
}
// #[cfg(windows)]
// #[link(name="ninja_rs",kind="static")]
// unsafe extern "C" {
//     unsafe fn is_long_path_aware_runtime() -> bool;
// }

pub fn main() {
    #[cfg(windows)]
    {
        // if unsafe {!is_long_path_aware_runtime()} {
        //     println!("Ninja is not Long Path Aware. Builds may fail on paths > 260 or exhibit buggy behavior");

        // }
    }
    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect();

    let arg_ptrs: Vec<*const c_char> = args
        .iter()
        .map(|arg| arg.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();

    println!("WARN: Current phase is reimplementing Ninja main in rust. there may be reduced null build performance");
    // yes we are sandwiching rust and ninda init here for right now
    let _config: ninja_rs::build::BuildConfig = Default::default();
    let mut options: ninja_rs::ninja::Options = Default::default();
    // we use clap here. clap is probably overkill but has HUGE support
    // and we could get fancier with it
    let clap_args = ninja_rs::ninja::NinjaCLIArgs::parse();
    let _staus = ninja_rs::status::StatusFactory::create(&_config);

    println!("CLAP parsed args successfully");
    println!("CLAP ARGS\r\n{clap_args:?}");
    options.input_file.push_str("build.ninja");



    let result = unsafe { ninja_main(args.len().try_into().unwrap(), arg_ptrs.as_ptr()) };
    std::process::exit(result);
}
