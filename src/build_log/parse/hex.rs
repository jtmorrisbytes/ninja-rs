// Copyright Jordan Morris 2026 jthecybertinkerer@gmail.com
// https://github.com/jtmorrisbytes
// fast hex parsing using SWAR, SIMD (sse,avx,etc)
// you may use this code under the MIT or apache licenses including but not limited to
// Copy modify redistribute sell or otherwise unless also restricted by
// the code in the surrounding Ninja repository as long as you include
// this header in every file
// see the Google Ninja LICENSE more permissions and restrictions

use std::{ffi::c_char, sync::atomic::AtomicPtr};

use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::{ParallelSlice, ParallelSliceMut}};

/// callers MUST ensure input type is [u8;16] or will crash or garble memory
type SingleHexDecodeFn = unsafe fn(*const u8) -> u64;
// #[allow(dead_code)]
static SINGLE_HEX_DECODER_FN: AtomicPtr<()> = AtomicPtr::new(setup_and_parse_single as *mut ());
type MultipleHexDecodeFn = unsafe fn(&[[u8; 16]], &mut [u64]);

static MULTIPLE_HEX_DECODER_FN: AtomicPtr<()> = AtomicPtr::new(setup_and_parse_multiple as *mut ());

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
/// uses rayon to execute the multiple byte parse over different cores
/// so we execute multiple elements at once. incurs a start up cost
/// so we want this on really large dataasets
pub fn ninja_rs__build_log__parse_hex_parallel(
    parallelism: Option<usize>,
    input: &[[u8; 16]],
    results: &mut [u64],
) {
    // the input MUST be as least large as the output
    debug_assert!(input.len()>=results.len());
    let parallelism = std::thread::available_parallelism()
        .map(|n| n.get())
        .ok()
        .or(parallelism)
        .unwrap_or(crate::ninja::Parallelism::DEFAULT_JOBS_COUNT);
    let chunk_size = input.len() / parallelism;
    let chunks_in = input.par_chunks_exact(chunk_size);
    let mut chunks_out = results.par_chunks_exact_mut(chunk_size);
    let rem_in = chunks_in.remainder();
    let rem_out = chunks_out.take_remainder();
    chunks_in.zip(chunks_out).for_each(|(in_c,out_c)| {
        ninja_rs__build_log__parse__fast_hex_multiple_to_u64(in_c, out_c);
    });
    ninja_rs__build_log__parse__fast_hex_multiple_to_u64(rem_in, rem_out);
    
}

pub unsafe fn setup_and_parse_multiple(input: &[[u8; 16]], output: &mut [u64]) {
    let fast_fn: MultipleHexDecodeFn = if is_x86_feature_detected!("sse3") {
        println!("SSE DETECTED using fast fn for hex parser");
        // tracing::debug!("SSE detected, using FAST fn");
        multiple_hex_to_u64_sse
    } else {
        // tracing::debug!("sse not detected. not using fast fn");
        multiple_hex_to_u64_SWAR
    };
    // Replace the pointer to this setup function with the fast function
    MULTIPLE_HEX_DECODER_FN.store(fast_fn as *mut (), std::sync::atomic::Ordering::Release);
    unsafe { fast_fn(input, output) }
}

#[target_feature(enable = "ssse3")]
/// THE CALLER MUST ENSURE THE INPUT TYPE ISmultiple of u8;16 OR WILL CRASH
pub unsafe fn multiple_hex_to_u64_sse(input: &[[u8; 16]], output: &mut [u64]) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    let addr = input.as_ptr().addr();
    // detect pointer alignment and use aligned loads if possible
    // because we require operating on a multiple of 16 bytes anyways this will just work
    let simd_loadfn = {
        if addr % 16 == 0 {
            _mm_load_si128
        } else {
            _mm_loadu_si128
        }
    };
    // 1. Load 16 ASCII bytes
    // --- LIFTED CONSTANTS (Loaded once into registers) ---
    let mask_0f = _mm_set1_epi8(0x0F);
    let char_9 = _mm_set1_epi8(b'9' as i8);
    let offset_9 = _mm_set1_epi8(9);
    let pack_mul = _mm_set1_epi16(0x0110);

    let len = input.len();
    for i in 0..len {
        // --- THE HOT KERNEL ---
        let x = unsafe { simd_loadfn(input[i].as_ptr() as *const __m128i) };

        let lo = _mm_and_si128(x, mask_0f);
        let gt9 = _mm_cmpgt_epi8(x, char_9);
        let offset = _mm_and_si128(gt9, offset_9);
        let nibbles = _mm_add_epi8(lo, offset);

        let packed16 = _mm_maddubs_epi16(nibbles, pack_mul);
        let packed8 = _mm_packus_epi16(packed16, packed16);

        // Use the index directly to avoid Vec::push overhead
        output[i] = (_mm_cvtsi128_si64(packed8) as u64).swap_bytes();
    }

    // 6. Fix endian (important)
    // result.swap_bytes()
}

// #[inline(always)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
/// the caller MUST ensure the source of this ptr is [u8;16] or will cause UB
pub fn multiple_hex_to_u64_SWAR(hex: &[[u8; 16]], output: &mut [u64]) {
    // Step 1: load bytes (big-endian so order matches hex string)
    // let hex: &[u8;16] = unsafe {std::mem::transmute(hex)};
    for i in 0..hex.len() {
        let input = unsafe { hex.get_unchecked(i) };

        let x = u128::from_be_bytes(*input);

        // Step 2: ASCII → nibble (0–15)
        let mut n = x & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
        let letter = (x >> 6) & 0x01010101010101010101010101010101;
        n += letter * 9;

        // Step 3: extract each byte and assemble u64
        // (fully unrolled, no loop)
        let b = n.to_be_bytes();

        let b = ((b[0] as u64) << 60)
            | ((b[1] as u64) << 56)
            | ((b[2] as u64) << 52)
            | ((b[3] as u64) << 48)
            | ((b[4] as u64) << 44)
            | ((b[5] as u64) << 40)
            | ((b[6] as u64) << 36)
            | ((b[7] as u64) << 32)
            | ((b[8] as u64) << 28)
            | ((b[9] as u64) << 24)
            | ((b[10] as u64) << 20)
            | ((b[11] as u64) << 16)
            | ((b[12] as u64) << 12)
            | ((b[13] as u64) << 8)
            | ((b[14] as u64) << 4)
            | ((b[15] as u64) << 0);
        output[i] = b;
    }
}

#[target_feature(enable = "ssse3")]
/// THE CALLER MUST ENSURE THE INPUT TYPE IS [u8;16] or multiple of 16 OR WILL CRASH
pub unsafe fn single_hex_to_u64_sse(input: *const u8) -> u64 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    // 1. Load 16 ASCII bytes
    let x = unsafe { _mm_loadu_si128(input as *const __m128i) };

    // 2. Convert ASCII → nibbles
    let lo = _mm_and_si128(x, _mm_set1_epi8(0x0F));

    let gt9 = _mm_cmpgt_epi8(x, _mm_set1_epi8(b'9' as i8));
    let offset = _mm_and_si128(gt9, _mm_set1_epi8(9));

    let nibbles = _mm_add_epi8(lo, offset);

    // 3. Pack nibbles into bytes using maddubs
    let packed16 = _mm_maddubs_epi16(
        nibbles,
        _mm_set1_epi16(0x0110), // (hi*16 + lo)
    );

    // 4. Pack 16-bit → 8-bit
    let packed8 = _mm_packus_epi16(packed16, packed16);

    // 5. Extract lower 64 bits
    let result = _mm_cvtsi128_si64(packed8) as u64;

    // 6. Fix endian (important)
    result.swap_bytes()
}

// 1. The "Setup" function that only runs once
unsafe fn setup_and_parse_single(ptr: *const u8) -> u64 {
    let fast_fn: SingleHexDecodeFn = if is_x86_feature_detected!("sse3") {
        print!("SSE DETECTED using fast fn for hex parser");
        // tracing::debug!("SSE detected, using FAST fn");
        single_hex_to_u64_sse
    } else {
        // tracing::debug!("sse not detected. not using fast fn");
        single_hex_to_u64_SWAR
    };

    // Replace the pointer to this setup function with the fast function
    SINGLE_HEX_DECODER_FN.store(fast_fn as *mut (), std::sync::atomic::Ordering::Release);

    // Call the fast function for this first request
    unsafe { fast_fn(ptr) }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
/// the caller MUST ensure this fn is called with [u8;16] or will cause UB
pub fn ninja_rs__build_log__parse__fast_hex_single_to_u64(hex: &[u8; 16]) -> u64 {
    let fn_ptr = SINGLE_HEX_DECODER_FN.load(std::sync::atomic::Ordering::Acquire);
    let fn_ptr: SingleHexDecodeFn = unsafe { std::mem::transmute(fn_ptr) };
    // let fn_ptr = unsafe {( as HashFn};
    unsafe { fn_ptr(hex.as_ptr()) }
}
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
/// the caller MUST ensure this fn is called with [u8;16] or will cause UB
pub fn ninja_rs__build_log__parse__fast_hex_multiple_to_u64(
    input: &[[u8; 16]],
    output: &mut [u64],
) {
    let fn_ptr = MULTIPLE_HEX_DECODER_FN.load(std::sync::atomic::Ordering::Acquire);
    let fn_ptr: MultipleHexDecodeFn = unsafe { std::mem::transmute(fn_ptr) };
    // let fn_ptr = unsafe {( as HashFn};
    unsafe { fn_ptr(input, output) };
}
// #[inline(always)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
/// the caller MUST ensure the source of this ptr is [u8;16] or will cause UB
pub fn single_hex_to_u64_SWAR(hex: *const u8) -> u64 {
    // Step 1: load bytes (big-endian so order matches hex string)
    let hex: &[u8; 16] = unsafe { std::mem::transmute(hex) };
    let x = u128::from_be_bytes(*hex);

    // Step 2: ASCII → nibble (0–15)
    let mut n = x & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
    let letter = (x >> 6) & 0x01010101010101010101010101010101;
    n += letter * 9;

    // Step 3: extract each byte and assemble u64
    // (fully unrolled, no loop)
    let b = n.to_be_bytes();

    ((b[0] as u64) << 60)
        | ((b[1] as u64) << 56)
        | ((b[2] as u64) << 52)
        | ((b[3] as u64) << 48)
        | ((b[4] as u64) << 44)
        | ((b[5] as u64) << 40)
        | ((b[6] as u64) << 36)
        | ((b[7] as u64) << 32)
        | ((b[8] as u64) << 28)
        | ((b[9] as u64) << 24)
        | ((b[10] as u64) << 20)
        | ((b[11] as u64) << 16)
        | ((b[12] as u64) << 12)
        | ((b[13] as u64) << 8)
        | ((b[14] as u64) << 4)
        | ((b[15] as u64) << 0)
}

// #[cfg(test)]
// this used here so we can confirm that we are mimicing the compilers stroll using the above function
unsafe extern "C" {
    // We use strtoull for hashes (unsigned long long)
    fn strtoull(nptr: *const c_char, endptr: *mut *mut c_char, base: i32) -> u64;
}

#[cfg_attr(test, test)]
pub fn test_fast_hex_to_u64() {
    fn run_test(input: &str) {
        let c_input = std::ffi::CString::new(input).unwrap();
        let c_ull = unsafe { strtoull(c_input.as_ptr(), std::ptr::null_mut(), 16) };
        let r_output = ninja_rs__build_log__parse__fast_hex_single_to_u64(
            input.as_bytes().try_into().unwrap(),
        );
        // assert_eq!(c_ull,r_output);
        if r_output != c_ull {
            let b_input: String = input
                .as_bytes()
                .iter()
                .map(|b| format!("{b:08b},"))
                .collect();
            panic!("Test failed input: {b_input} stroll: {c_ull:064b} != rust:{r_output:064b}");
        }
    }
    run_test("19b5ffbff15c5c7a");
}

#[cfg_attr(test, test)]
pub fn fuzz_hex_parser() {
    fn run_test(input: &[u8; 16], expected: &u64) {
        // let c_input = std::ffi::CString::new(input).unwrap();
        // let c_ull = unsafe { strtoull(c_input.as_ptr(), std::ptr::null_mut(), 16) };
        let r_output = ninja_rs__build_log__parse__fast_hex_single_to_u64(input);
        // assert_eq!(c_ull,r_output);
        if &r_output != expected {
            // let b_input: String = input
            //     .as_bytes()
            //     .iter()
            //     .map(|b| format!("{b:08b},"))
            //     .collect();
            panic!("Test failed input: {input:?} stroll: {expected:064b} != rust:{r_output:064b}");
        }
    }
    use rand::RngExt;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use std::time::{SystemTime, UNIX_EPOCH};

    // #[cfg(test)]
    fn random_hex_16<R: Rng>(rng: &mut R, out: &mut [u8; 16]) {
        const HEX: &[u8] = b"0123456789abcdef";
        // let mut out = [0u8; 16];

        for i in 0..16 {
            out[i] = HEX[rng.random_range(0..16)];
        }
    }
    let test_start = std::time::Instant::now();
    let seed = std::env::var("FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        });
    println!("seed = {}", seed);
    const ITERS: usize = 1_000_000;
    // let mut sum = 0u64;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut inputs = vec![[0u8; 16]; ITERS];
    let mut ouptuts = vec![0_u64; ITERS];
    // let mut c_strs = vec![std::ffi::CString::default();ITERS];
    let mut c_strtoulls = vec![0u64; ITERS];
    // this fakes a cString by leaving 0 at the end
    let mut scratch =[0u8;17];
    // ptr required here due to aliasing rules;
    let scratch_ptr = scratch.as_ptr();
    // and mutates this instead
    let input = unsafe {scratch.get_unchecked_mut(0..16)};
    for i in 0..ITERS {
        // let input = unsafe { inputs.get_unchecked_mut(i) };
        for b in input.iter_mut() {
            *b = b"0123456789abcdef"[rng.random_range(0..16)];
        }
        let arr = input.try_into().unwrap();
        inputs[i] = arr;
        // let c_string = std::ffi::CString::new(scratch).unwrap();
        let expected = unsafe { strtoull(scratch_ptr.cast(), std::ptr::null_mut(), 16) };
        c_strtoulls[i] = expected;
    }
    let setup_elapsed = test_start.elapsed();
    println!("Test setup took {}",setup_elapsed.as_nanos());
    //  let mut out = [0u8;16];
    // let mut start;
    // let mut elapsed;
    // let mut total_ns = 0u128;
    // let mut count = 0usize;
    // let mut i = 0;

    let start = std::time::Instant::now();
    std::hint::black_box(ninja_rs__build_log__parse_hex_parallel(
        None,
        inputs.as_slice(),
        &mut ouptuts,
    ));
    let elapsed = start.elapsed().as_nanos();
    for i in 0..ITERS {
        let expected = unsafe { c_strtoulls.get_unchecked(i) };
        let output = unsafe { ouptuts.get_unchecked(i) };
        if expected != output {
            panic!("Test failed {expected} !={output} at index {i} of {ITERS}");
        }
    }
    // const BATCH:usize = (ITERS.ilog2() as usize) * 32;
    println!(
        "Whole parse took : {:.2} ns average for {ITERS} iters",
        elapsed as f64 / ITERS as f64
    );
}
