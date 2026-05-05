#![allow(dead_code)]

// #[path="parse/avx2.rs"]
pub mod parse;


/// Trait for objects that can answer questions about the manifest
/// for the BuildLog (usually implemented by your `State` or `Graph` struct).
pub trait BuildLogUser {
    /// Return if a given output is no longer part of the build manifest.
    ///
    /// This is used during log recompaction to prune entries that no longer
    /// exist in the current .ninja files.
    ///
    /// # Performance
    /// As the C++ comment notes, this doesn't have to be fast because
    /// recompaction is a rare maintenance task.
    fn is_path_dead(&self, path: &str) -> bool;
}
// use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

// use memmap2::Mmap;

use crate::build::Edge;

/// The status of a BuildLog load operation.
pub enum LoadStatus {
    LoadSuccess,
    LoadNotFound,
    LoadError,
}

// /// A single entry in the build log representing a previously executed command.
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct LogEntry {
//     pub start_time: i32,
//     pub end_time: i32,
//     pub
//     pub output: String, // We'll intern this or use slices in the "Beast" version
//     pub command_hash: String,
//     pub mtime: i64, // TimeStamp equivalent
// }

// impl LogEntry {
//     pub fn new(output: String) -> Self {
//         Self {
//             output,
//             command_hash: 0,
//             start_time: 0,
//             end_time: 0,
//             mtime: 0,
//         }
//     }

//     /// Your Beast Mode GxHash or xxHash goes here!
//     pub fn hash_command(_command: &str) -> u64 {
//         // Implementation using your preferred SIMD hasher
//         0
//     }
// }

pub struct BuildLog {
    /// Maps output paths to their last successful build metadata.
    /// Using your Agnostic Hashing here makes it slash-insensitive!
    // entries: HashMap<String, LogEntry>,

    /// The handle for appending new entries.
    log_file: Option<BufWriter<File>>,
    log_file_path: PathBuf,

    needs_recompaction: bool,
}

impl BuildLog {
    pub fn new() -> Self {
        Self {
            // entries: HashMap::new(),
            log_file: None,
            log_file_path: PathBuf::new(),
            needs_recompaction: false,
        }
    }

    /// Prepares the log for writing.
    pub fn open_for_write(
        &mut self,
        path: PathBuf,
        _user: &dyn BuildLogUser,
    ) -> Result<(), String> {
        self.log_file_path = path;
        // In Ninja, we don't open the file until we actually have an entry to write.
        let _ = self.needs_recompaction;
        Ok(())
    }

    /// The "Hot" function: Records a successful command execution.
    pub fn record_command(
        &mut self,
        _edge: &Edge,
        _start_time: i32,
        _end_time: i32,
        _mtime: i64,
    ) -> Result<(), String> {
        self.open_for_write_if_needed()?;
        // 1. Create LogEntry
        // 2. Hash command
        // 3. WriteEntry to disk
        // 4. Update internal HashMap
        Ok(())
    }

    /// Load the on-disk log using your SSE4 scanner logic.
    pub fn load(&mut self, _path: &Path, _err: &mut String) -> LoadStatus {
        // BEAST MODE: Use MapViewOfFile here!
        LoadStatus::LoadSuccess
    }

    // pub fn lookup_by_output(&self, path: &str) -> Option<&LogEntry> {
    //     // Thanks to your Agnostic DAG logic, this lookup
    //     // doesn't care about / vs \
    //     self.entries.get(path)
    // }

    /// Rewrites the log to remove "dead" entries.
    pub fn recompact(&mut self, _path: &Path, _user: &dyn BuildLogUser) -> Result<(), String> {
        // Use your BuildLogUser::is_path_dead here to filter self.entries
        Ok(())
    }

    fn open_for_write_if_needed(&mut self) -> Result<(), String> {
        if self.log_file.is_some() {
            return Ok(());
        }
        // Open file in append mode using your Long Path Aware logic
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct LogEntyV7<'p, 'h> {
    start_time: i32,
    end_time: i32,
    mtime: i64,
    path: &'p str,
    command_hash: &'h str,
}

// helper functions for converting bytes into integers
#[inline(always)]
pub fn fast_atoi_i32(bytes: &[u8]) -> i32 {
    let mut val = 0i32;
    for &b in bytes {
        // b - b'0' converts ASCII '0'-'9' (0x30-0x39) to 0-9
        // We use wrapping operations to keep it fast
        val = val.wrapping_mul(10).wrapping_add((b - b'0') as i32);
    }
    val
}

#[inline(always)]
pub fn fast_atoi_i64(bytes: &[u8]) -> i64 {
    let mut val = 0i64;
    for &b in bytes {
        val = val.wrapping_mul(10).wrapping_add((b - b'0') as i64);
    }
    val
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
pub fn parse_ninja_log_v7_text_using_avx2(b: &[u8]) -> Result<(), String> {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    use std::collections::HashMap;
    let b = b.strip_prefix(b"# ninja log v7\n").unwrap_or(b);
    let len = b.len();
    let v_tab = _mm256_set1_epi8(b'\t' as i8);
    let v_nl = _mm256_set1_epi8(b'\n' as i8);
    let mut cursor: usize = 0;
    let base_ptr = b.as_ptr();
    let mut output = HashMap::<&str,LogEntyV7>::with_capacity(100);
    // let unalingment = base_ptr.addr() % 32;
    // let bytes_to_align = if unalingment == 0 {
    //     0
    // } else {
    //     32 - unalingment
    // };
    // let aligned = unsafe { base_ptr.add(bytes_to_align) };

    // let _loader = {
    //     if base_ptr.addr() & 31 == 0 {
    //         println!("using aligned instruction");
    //         _mm256_load_si256
    //     }
    //     else {
    //         println!("using unaligned instructions. check alignment 64 bytes, {}", base_ptr.addr() %64);

    //         _mm256_loadu_si256
    //     }
    // };
    // let mut mask = 0;
    let mut e = LogEntyV7 {
    start_time:0,
    end_time:0,
    mtime:0,
    path:crate::EMPTY_STR,
    command_hash:crate::EMPTY_STR,
    };
    let mut field_index= 0;
    let mut field_start = 0;
    let mut mask = 0;
    while cursor + 32 <= len {
        unsafe {
            // 2. Load 32 bytes (unaligned)
            let block = _mm256_loadu_si256(base_ptr.add(cursor) as *const __m256i);
            // println!("block ptr {aligned:?}");
            // 3. Compare for both delimiters
            let match_tab = _mm256_cmpeq_epi8(block, v_tab);
            let match_nl = _mm256_cmpeq_epi8(block, v_nl);

            // 4. Combine masks (logic: (is_tab OR is_nl))
            let combined = _mm256_or_si256(match_tab, match_nl);

            // 5. Move to u32 mask
            mask = _mm256_movemask_epi8(combined) as u32;
            // dbg!(mask);

            if mask != 0 {
                let block_base = cursor;
                while mask != 0 {
                    let index = mask.trailing_zeros() as usize;
                    let found_at = block_base + index;

                    // BEAST MODE: Slice from field_start, not block_base!
                    let field = b.get_unchecked(field_start..found_at);
                    field_start = found_at + 1;
                    match field_index {
                        0 => {e.start_time = fast_atoi_i32(field); field_index+=1;},
                        1 => {e.end_time = fast_atoi_i32(field); field_index+=1;}
                        2 => {e.mtime = fast_atoi_i64(field); field_index+=1;}
                        3 => {e.path = std::str::from_utf8_unchecked(field);  field_index+=1;}
                        4 => {
                            e.command_hash = std::str::from_utf8_unchecked(field);
                            // println!("field {e:?}");
                            output.insert(e.path,e.clone());
                            field_index = 0;
                            // continue;
                        }
                        _=> {field_index=0;
                            // field_index+=1;
                            // continue;
                        }
                    }
                   
                    // process_field(field);
                    // The NEXT field starts after this delimiter

                    mask &= mask - 1;
                }
                // Move the scanner cursor to where the LAST field ended
                cursor = field_start;
            } else {
                // No delimiter in this 32-byte block.
                // Just move the scanner, but DON'T move field_start!
                cursor += 32;
            }
        }
        // cursor += 32;
    }
    let remaining = unsafe {b.get_unchecked(cursor..len)};
    println!("cursor {cursor} {len} {field_start} field_index: {field_index} {} {:?} {mask:032b} {remaining:?}\n{output:?}",len - cursor, char::from_u32(b[cursor-1] as u32));

    Ok(())
}

// #[cfg(target_feature = "sse4.2")]
// #[cfg(target_feature = "sse4")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse4.2")]
pub fn parse_ninja_log_v7_text_using_sse_4_2(_s: &[u8]) -> Result<(), String> {
    let s = _s.strip_prefix(b"# ninja log v7\n").unwrap_or(_s);
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    use std::mem::MaybeUninit;

    let mut cursor = 0;
    let mut offset = cursor;
    let needles = _mm_setr_epi8(
        b'\t' as i8,
        b'\n' as i8,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    let mut start_time: MaybeUninit<i32> = MaybeUninit::uninit();
    let mut end_time: MaybeUninit<i32> = MaybeUninit::uninit();
    let mut mtime: MaybeUninit<i64> = MaybeUninit::uninit();
    let mut path: &str = crate::EMPTY_STR;
    // let mut command_hash = String::with_capacity(16);
    let mut field_index = 0;
    while offset + 16 <= s.len() {
        unsafe {
            let block = _mm_loadu_si128(s.as_ptr().add(offset) as *const __m128i);
            let index = _mm_cmpistri(
                needles,
                block,
                _SIDD_UBYTE_OPS | _SIDD_CMP_EQUAL_ANY | _SIDD_POSITIVE_POLARITY,
            );
            // index will be 0-15 if a match is found, or 16 if not.
            if index < 16 {
                let found_at = offset + index as usize;
                // let field = std::slice::from_raw_parts(s.as_ptr(), len)
                let field = s.get_unchecked(cursor..found_at);
                // will work but be garbled if not utf8, as long as u
                // dont write to the input string
                // and it comes from a valid utf8 input your fine
                // let field = std::str::from_utf8_unchecked(field);
                cursor = found_at + 1; // Move past the delimiter
                // println!("field {field}");
                offset = cursor;
                match field_index {
                    0 => {
                        start_time.write(fast_atoi_i32(field));
                    }
                    1 => {
                        end_time.write(fast_atoi_i32(field));
                    }
                    2 => {
                        mtime.write(fast_atoi_i64(field));
                    }
                    3 => path = std::str::from_utf8_unchecked(field),
                    4 => {
                        let _e = LogEntyV7 {
                            start_time: start_time.assume_init_read(),
                            end_time: end_time.assume_init_read(),
                            mtime: mtime.assume_init_read(),
                            path: path,
                            command_hash: std::str::from_utf8_unchecked(field),
                        };
                        // path.clear();
                        // command_hash.clear();
                        field_index = 0;
                        // println!("{e:?}");
                        continue;
                    }
                    _ => {
                        field_index = 0;
                        continue;
                    }
                }
                field_index += 1;
                continue;
            }
            offset += 16;
        }
    }
    // println!("cursor {cursor}");
    Ok(())
}

#[target_feature(enable = "avx512bw,avx512f")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn parse_ninja_log_v7_text_using_avx512(b: &[u8]) -> Result<(), String> {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    let b = b.strip_prefix(b"# ninja log v7\n").unwrap_or(b);
    let len = b.len();
    let v_tab = _mm512_set1_epi8(b'\t' as i8);
    let v_nl = _mm512_set1_epi8(b'\n' as i8);
    let mut cursor = 0;
    let base_ptr = b.as_ptr();
    // use ptr dispatch. if the ptr is aligned correctly use load, else use loadu
    let loader = {
        if base_ptr.addr() & 63 == 0 {
            _mm512_load_si512
        } else {
            _mm512_loadu_si512
        }
    };
    while cursor + 64 <= len {
        unsafe {
            // 1. Load 64 bytes into ZMM
            let block = loader(b.as_ptr().add(cursor) as *const _);

            // 2. Compare directly to masks (k-registers)
            // _mm512_cmpeq_epi8_mask returns a __mmask64 (a u64)
            let mask_tab = _mm512_cmpeq_epi8_mask(block, v_tab);
            let mask_nl = _mm512_cmpeq_epi8_mask(block, v_nl);

            // 3. Combine masks
            let mask = mask_tab | mask_nl;

            if mask != 0 {
                // Found a delimiter!
                let first_hit = mask.trailing_zeros() as usize;
                let found_at = cursor + first_hit;
                let _field = b.get_unchecked(cursor..found_at);
                // ... [BEAST MODE] Zero-copy process s[cursor..found_at] ...
                println!("_field {_field:?}");
                cursor = found_at + 1;
                continue;
            }

            cursor += 64;
        }
    }
    Ok(())
}

pub fn parse_ninja_log_v7_text(_s: &[u8]) -> Result<(), String> {
    if is_x86_feature_detected!("avx512bw") && is_x86_feature_detected!("avx512f") && _s.len() >= 64
    {
        unsafe { parse_ninja_log_v7_text_using_avx512(_s) }
    } else if is_x86_feature_detected!("avx2") && _s.len() >= 32 {
        unsafe { parse_ninja_log_v7_text_using_avx2(_s) }
    } else if is_x86_feature_detected!("sse4.2") && _s.len() >= 16 {
        unsafe { parse_ninja_log_v7_text_using_sse_4_2(_s) }
    } else {
        todo!("Parse using lesser SIMD intrinics, asm, or split")
    }
}

#[cfg(test)]
fn generate_massive_log(path: &str, target_size_gb: f64) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::{BufWriter, Write};
    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(1024 * 1024, file);

    writer.write_all(b"# ninja log v7\n")?;

    let mut current_size = 15; // Header size
    let target_bytes = (target_size_gb * 1024.0 * 1024.0 * 1024.0) as u64;

    let mut i = 0;
    while current_size < target_bytes {
        // Generate a synthetic entry
        let line = format!(
            "{}\t{}\t{}\tbuild/very/long/path/to/a/synthetic/object/file_{}.obj\t{:016x}\n",
            i,
            i + 100,
            1700000000000 + i as i64,
            i % 1000,
            0x19b5ffbff1c330ecu64 ^ (i as u64)
        );
        let bytes = line.as_bytes();
        writer.write_all(bytes)?;
        current_size += bytes.len() as u64;
        i += 1;
    }
    writer.flush()?;
    Ok(())
}

#[test]
pub fn test_build_log() {
    // use std::io::Read;
    use memmap2::Mmap;
    if is_x86_feature_detected!("avx512bw") && is_x86_feature_detected!("avx512f") {
        println!("🚀 Congradulations, you have potentially unleashed max performance with avx512");
    }
    if is_x86_feature_detected!("avx2") {
        println!("🚀 avx2 detected! using WIDE PARSE.");

        // unsafe {parse_ninja_log_v7_text_using_avx2(_s)}
    }
    if is_x86_feature_detected!("sse4.2") {
        println!("🚀 SSE4.2 detected! STTNI shredder enabled.");
    } else {
        println!("⚠️  SSE4.2 not found. Falling back to slow-path parsing.");
    }
    // generate_massive_log("./target/.ninja_log_test3.log", 15.0).unwrap();
    // if !std::fs::exists("./target/.ninja_log_test.log").unwrap_or(false) {
    // }
    // let mut buf = String::with_capacity(10240);
    let file = std::fs::File::open("./target/.ninja_log_test.log").unwrap();
    let map = unsafe { Mmap::map(&file).unwrap() };
    // file.read_to_string(&mut buf).unwrap();
    // let mut window = &*buf;
    for _ in 1..5 {
        let then = std::time::Instant::now();
        std::hint::black_box(parse_ninja_log_v7_text(map.as_ref()).unwrap());
        let now = std::time::Instant::now();
        let elapsed = now - then;
        println!("Parsing took {:?}", elapsed.as_micros());
    }

    // crate::fs::hexdump(&buf);
}
