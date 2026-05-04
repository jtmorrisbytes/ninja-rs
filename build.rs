use std::{io::{BufRead, BufReader, Cursor}, path::PathBuf, process::Command, time::UNIX_EPOCH};

use embed_resource::ParamsIncludeDirs;

fn main() {
    if std::env::var("IS_LIB_SUBBUILD").is_ok() {
        return;
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/fs.rs");
    println!("cargo:rerun-if-changed=src/includes.rs");



    let target_triple = std::env::var("TARGET").expect("TARGET env var not set");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir_path = std::path::Path::new(&manifest_dir);
    let is_windows = std::env::var_os("CARGO_CFG_WINDOWS").is_some();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir_path = std::path::Path::new(&out_dir);
    let target_dir = std::path::Path::new(&out_dir)
        .join("../../../deps");

    // we need a second scratch directory for us to build the lib target in 
    let sub_target_dir = out_dir_path.join("ninja-rs-lib");
    if !sub_target_dir.exists() {
        std::fs::create_dir_all(&sub_target_dir).unwrap();
    }
    // we must run cargo from here
        let status = Command::new("cargo")
        .env("IS_LIB_SUBBUILD","1")
        .args([
            "build",
            "--manifest-path",manifest_dir_path.join("Cargo.toml").to_str().unwrap(),
            "--package", "ninja-rs",
            "--lib",
            "--target-dir", sub_target_dir.to_str().unwrap(), // CRITICAL: Separate target dir
            "--target", &target_triple,
            "--release"
        ])
        .status()
        .expect("Failed to spawn sub-cargo build");
    
    if !status.success() {
        use std::io::Read;
        // let stdout = unsafe {String::from_utf8_unchecked(status)};
        // let mut reader = Cursor::new(stdout);
        
        // let mut line = String::new();
        // while let Ok(_bytes) = reader.read_line(&mut line) {
        //     println!("cargo:error={line}");
        //     line.clear();
        //     if _bytes == 0 {
        //         break;
        //     }
        // }
        // let mut reader = Cursor::new(status.stderr);
        // while let Ok(_bytes) = reader.read_line(&mut line) {
        //     line.clear();
        //     println!("cargo::warning={line}");
        //     if _bytes == 0 {
        //         break;
        //     }
        // }
        panic!("Build failed: Cargo exited with {:?}",status);
    }
    // I know ninja loved using itself to bootstrap itself
    // but its dependency hell for right now
    let build_root = manifest_dir_path.join("build");
    // create the build root if not exists
    if !build_root.exists() {
        std::fs::create_dir_all(&build_root).unwrap();
    }
    let mut build = cc::Build::new();
    build.cpp(true);
    build.std("c++17");
    build.include("src");
    build.include(&build_root);

    build.define("NOMINMAX", None);
    build.define("_CRT_SECURE_NO_WARNINGS", None);
    build.define("_HAS_EXCEPTIONS", "0");
    build.define("NINJA_PYTHON", "\"python.exe\"");

    // set up windows specific configurations for the build
    if is_windows {
        // defines
        build.define("WIN32", "1");
        build.define("_WIN32","1");

        // compiler flags
        build.flag("/nologo");
        build.flag("/utf-8");
        build.flag("/GR-"); // Disable RTTI
        build.flag("/Zc:__cplusplus");
        build.flag("/Zi");
        build.flag("/FS");
        build.flag("/Wall");
        build.flag("/W4");
        // build.flag("/WX");
        build.flag("/EHsc");
        build.flag("/analyze");
        build.flag(format!("/LIBPATH:{out_dir}"));
        // build.flag("/RTC1");
                // Disable the specific warnings Ninja ignores
        let disabled_warnings = [
            "4530", "4100", "4706", "4244", "4512", 
            "4800", "4702", "4127", "4355", "4091",
            "4820", "4514"
        ];
        for wd in disabled_warnings {
            build.flag(&format!("/wd{}", wd));
        }
    }


    // attempt to build ninja using cc
    let mut src_files = vec![
        "depfile_parser.cc",
        "lexer.cc",
        "build.cc",
        "build_log.cc",
        "clean.cc",
        "clparser.cc",
        "debug_flags.cc",
        "deps_log.cc",
        "disk_interface.cc",
        "dyndep.cc",
        "dyndep_parser.cc",
        "edit_distance.cc",
        "elide_middle.cc",
        "eval_env.cc",
        "explanations.cc",
        "graph.cc",
        "graphviz.cc",
        "jobserver.cc",
        "json.cc",
        "line_printer.cc",
        "manifest_parser.cc",
        "metrics.cc",
        "missing_deps.cc",
        "parser.cc",
        "real_command_runner.cc",
        "state.cc",
        "status_printer.cc",
        "string_piece_util.cc",
        "util.cc",
        "version.cc",
        "ninja.cc"
    ];

    // include these files only for windows
    if is_windows {
        src_files.extend([
            "subprocess-win32.cc",
            "includes_normalize-win32.cc",
            "jobserver-win32.cc",
            "msvc_helper-win32.cc",
            "msvc_helper_main-win32.cc",
            "minidump-win32.cc",
            "getopt.c", // Note: this is .c, not .cc
        ]);
    }

    // we must include the crate's library target
    if is_windows {
        build.object(sub_target_dir.join(target_triple).join("release").join("ninja_rs.lib"));
    } else {
        build.object(sub_target_dir.join(target_triple).join("release").join("libninja_rs.a"));
    }

    let src_paths:Vec<_> = src_files.iter().map(|f|format!("src/{}",f)).collect();

    for file in src_paths.iter() {
        build.file(file);
        // make sure cargo knows to watch these files
        println!("cargo:rerun-if-changed={file}");
    }
    // scan the out_dir
    let readdir = std::fs::read_dir(&out_dir).unwrap();

    let entries:Vec<_> = 
        readdir.filter(|f|f.is_ok())
        .map(|r|r.unwrap())
        .filter(|e| {
            let f = match e.file_type() {
                Ok(f)=>f,
                Err(_)=>return false
            };
            return f.is_file();
        })
        .collect();
    //  dbg!(&entries);
    //  todo!();
    for file in src_files.iter() {
        let metadata = std::fs::metadata(format!("src/{}",file)).unwrap();
        let modified = metadata.modified().unwrap();
        // let now = std::time::SystemTime::now();
        // let elapsed = now.duration_since(modified)?;
        let o_filename = file.replace(".cc", ".o").replace(".c", ".o");
        // dbg!(o_filename);
        // panic!();
        let entry = entries.iter().find(|direntry|direntry.file_name().into_string().unwrap().contains(&o_filename));
        if entry.is_none() {
            build.compile("ninja");
            break;
        }
        let entry = entry.unwrap();
        // dbg!(entry);
        let m = entry.metadata().unwrap();
        let o_modified = m.modified().unwrap();
        if modified > o_modified {
             build.compile("ninja");
             break;
        }
        // panic!();
        // let o_modified = entry.metadata().unwrap().modified().unwrap();
        // if modified > o_modified {
        //     break;
        // }
        

        
    }
    println!("cargo:rustc-link-search={out_dir}");
    // todo!();
    // build.compile("ninja");
    // generate the resource file
    if is_windows {
        let mut res_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        res_path.push("app.rc");
        // let include_dirs = ParamsIncludeDirs (,
        // );
        embed_resource::compile(&res_path,vec![res_path.to_str().unwrap()])
        .manifest_required().unwrap();
    } 
}