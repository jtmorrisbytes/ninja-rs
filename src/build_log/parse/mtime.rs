unsafe extern "C" {
    pub fn strtoll(
        nptr: *const std::os::raw::c_char,
        endptr: *mut *mut std::os::raw::c_char,
        base: std::os::raw::c_int,
    ) -> std::os::raw::c_longlong;
}

#[repr(C, align(16))]
struct SimdAligned<T, const N: usize>([T; N]);
impl<const N: usize> SimdAligned<u8, N> {
    pub fn from_ascii(src: &[u8]) -> Self {
        let mut out = [b'0'; N];

        out[..src.len()].copy_from_slice(src);

        Self(out)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 1> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 2> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 3> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 4> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 5> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 6> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 7> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 8> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 9> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 10> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 11> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 12> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 13> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 14> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 15> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 16> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 17> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 18> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 19> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}
impl std::convert::From<&[u8]> for SimdAligned<u8, 20> {
    fn from(value: &[u8]) -> Self {
        Self::from_ascii(value)
    }
}

// Level 1: [10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1]
static MULTS_10_1: SimdAligned<i8, 16> =
    SimdAligned([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1]);

static MULTS_100_1: SimdAligned<i16, 8> = SimdAligned([100, 1, 100, 1, 100, 1, 100, 1]);

static MULTS_10000_1: SimdAligned<i32, 4> = SimdAligned([10000, 1, 10000, 1]);

// Pre-compute these masks for every length (0-16)
// This mask "right-justifies" a string of length 'L'
// and fills the front with a value that becomes 0 after '0' subtraction
static SHUFFLE_MASKS: SimdAligned<[u8; 16], 20> = SimdAligned([
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
        0x80,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3, 4,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3, 4, 5,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3, 4, 5, 6,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3, 4, 5, 6, 7,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8,
    ],
    /* ... 0-9 ... */
    // Example for length 10: shifts bytes 0..10 to 6..16, fills 0..6 with 0x2F
    // (0x2F - 0x30 = 0x80, but we can use 0x30 to get 0)
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    ],
    [
        0x80, 0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    ],
    [0x80, 0x80, 0x80, 0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    [0x80, 0x80, 0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    [0x80, 0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
    [0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    // these are here so we dont crash by accident but will
    // produce incorrect results
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
]);
#[allow(unused_macros)]
macro_rules! dump_xmm_binary {
    ($reg:expr) => {
        // unsafe {
            let low: u64;
            let high: u64;
            core::arch::asm!(
                concat!("movq {0}, ", $reg),        // Extract low 64 bits
                concat!("movhlps xmm15, ", $reg),   // Move high bits to xmm15
                "movq {1}, xmm15",                  // Extract high 64 bits
                out(reg) low,
                out(reg) high,
                out("xmm15") _, // Mark temp register as clobbered
            );

            let bytes: [u8; 16] = std::mem::transmute([low, high]);
            print!("{} binary: ", $reg);
            // Print from high byte (15) to low byte (0)
            for b in bytes.iter().rev() {
                print!("{:08b} ", b);
            }
            println!();
        }
    // };
}

thread_local! {
    static SIMD_INPUT_BUFFER: std::cell::UnsafeCell<SimdAligned<u8,16>> =
        std::cell::UnsafeCell::new(SimdAligned([0u8; 16]));
}
#[allow(non_snake_case)]
#[target_feature(enable = "sse4.1")]
/// parses an i64 UNSAFELY but fast. 2x faster than strtoll.
/// relies on the input buffer len so pass the true string len
/// but you MUST make sure that the input buffer from the slice is at least 20 bytes long
/// even if you pass an input slice of len 1.
/// if you EVER give it a 0 size buffer it may overread the first byte
/// trying to figure out if its signed
unsafe fn ninja_rs__unsafe__atoi__i64(bytes: &[u8]) -> i64 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    // let ptr = bytes.as_ptr();
    // let len =
    // we overread the first byte on purpose for this
    // let bytes = container.0;
    let first_byte = unsafe { *bytes.get_unchecked(0) } as i32;
    let is_negative = ((first_byte - 46) >> 31) & 1;
    let bytes = unsafe { bytes.get_unchecked(is_negative as usize..bytes.len()) };
    let len = bytes.len().min(19);
    // let bytes = b.unwrap_or(bytes);
    // 1. Load 16 bytes (might read past the end of small strings, which is fine in mmap)
    unsafe {
        let raw = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);

        // let raw = SIMD_INPUT_BUFFER.with(|b| {
        //     let p = b.get();

        //     let p: *mut u8 = p as _;
        //     std::ptr::write_bytes(p, 0, 16);
        //     p.copy_from_nonoverlapping(bytes.as_ptr(), len);
        //     _mm_load_si128(p as *const __m128i)
        // });
        // let raw = _mm_loadu_si128(ptr as *const __m128i);
        // println!("raw {raw:?}");

        // 2. Right-justify using the length-specific mask
        // This effectively "pads" with '0' without a single branch or copy
        let mask = _mm_load_si128(SHUFFLE_MASKS.0[len].as_ptr() as *const __m128i);
        // println!("mask {mask:?}");

        let justified = _mm_shuffle_epi8(raw, mask);
        // println!("justified {justified:?}");

        // 2. Identify the slots that were zeroed (mask < 0)
        let zeroed_slots = _mm_cmplt_epi8(mask, _mm_set1_epi8(0));
        // println!("zeroed_slots {zeroed_slots:?}");

        // 3. Fill those slots with ASCII '0' (0x30)
        let padding = _mm_and_si128(zeroed_slots, _mm_set1_epi8(b'0' as i8));
        // println!("padding output {padding:?}");

        let mut scratch = _mm_or_si128(justified, padding);
        // println!("justfied por padding =  {scratch:?}");
        // 3. Convert ASCII -> Nibbles
        scratch = _mm_sub_epi8(scratch, _mm_set1_epi8(b'0' as i8));
        // println!("sub epi8 output {scratch:?}");

        let w1 = _mm_set1_epi16(0x010A);

        let sum_16 = _mm_maddubs_epi16(scratch, w1);
        // println!("w1 maddubs scratch {sum_16:?}");

        let w2 = _mm_set1_epi32(0x00010064); // 0x0001 (1) and 0x0064 (100)
        let sum_32 = _mm_madd_epi16(sum_16, w2);
        // println!("w2 madd sum16 =  {sum_32:?}");

        // println!(
        //     "sum32 {} {} {} {}",
        //     _mm_extract_epi32::<0>(sum_32),
        //     _mm_extract_epi32::<1>(sum_32),
        //     _mm_extract_epi32::<2>(sum_32),
        //     _mm_extract_epi32::<3>(sum_32),
        // );

        let w2_1 = _mm_set_epi32(1, 10000, 1, 10000); // 10000
        let w2_2 = _mm_mullo_epi32(sum_32, w2_1);
        let w2_3 = _mm_srli_si128(w2_2, 4);
        let w2_4 = _mm_add_epi32(w2_3, w2_2);
        // println!("W2_1 {w2_1:?}\nW2_2 {w2_2:?}\nW2_2 {w2_2:?} w2_3 {w2_3:?}\n w2_4 {w2_4:?}");

        // 2. Weights for PMULDQ
        let w3 = _mm_set_epi32(0, 1, 0, 100_000_000);

        // 3. Multiply
        let sum_64 = _mm_mul_epu32(w2_4, w3);
        // println!("sum64 =  {sum_64:?}");

        let high_shifted = _mm_srli_si128(sum_64, 8);
        // println!("high_shifted =  {high_shifted:?}");

        // 3. Add them together in SIMD
        let final_xmm = _mm_add_epi64(sum_64, high_shifted);
        // println!("head: =  {final_xmm:?}");

        let mut head = _mm_extract_epi64::<0>(final_xmm);
        // println!("head {head}");
        let tail_start = std::cmp::min(bytes.len(), 16);
        let tail = bytes.get_unchecked(tail_start..bytes.len());
        // let tail_str = std::str::from_utf8_unchecked(tail);
        // let input_str = std::str::from_utf8_unchecked(bytes);
        let mut tail_val = 0;
        // println!("input: {input_str} tail {tail_str}");
        for byte in tail {
            head = head * 10;
            tail_val = tail_val * 10 + (byte - b'0') as i64;
            // println!(" Tail byte {byte} ");
        }
        let mask = -(is_negative as i64);
        let result = head + tail_val;
        let final_i64 = (result ^ mask).wrapping_add(is_negative as i64);
        return final_i64;
        // 4. NOW extract once to get your final i64
        // println!("final_result =  {final_result:?}");
        // let _low_8_raw: u64 = i64::cast_unsigned(_mm_extract_epi64::<0>(sum_64));
        // let _high_8_raw: u64 = i64::cast_unsigned(_mm_extract_epi64::<1>(sum_64));
        // println!("_low_8_raw {_low_8_raw:064b} {_high_8_raw:064b}");
    }
}

#[allow(non_snake_case)]
pub fn ninja_rs__atoi_i64_variable(input: &[u8]) -> i64 {
    if !is_x86_feature_detected!("sse4.1") {
        return self::normal_atoi_mtime(&input);
    } else {
        //
        unsafe { ninja_rs__unsafe__atoi__i64(&input) as i64 }
    }

  
}
// #[inline(always)]
// works on any length of string
pub fn normal_atoi_mtime(bytes: &[u8]) -> i64 {
    let mut result = 0i64;
    for &byte in bytes {
        // Shift left by 1 decimal place and add new digit
        // (result * 10) can be optimized to (result << 3) + (result << 1)
        result = result.wrapping_mul(10).wrapping_add((byte - b'0') as i64);
    }
    result
}
#[cfg_attr(test, test)]
pub fn mtime_test_atoi() {
    // let randoms =
    // Efficiently collect 255 random i64s
    use rand::RngExt;
    let mut rng = rand::rng();
    let random_values: Vec<i64> = (0..10_000_000).map(|_| rng.random::<i64>()).collect();
    let inputs: Vec<_> = random_values.iter().map(|s| s.to_string()).collect();
    let cstrs: Vec<_> = inputs
        .iter()
        .map(|s| unsafe { std::ffi::CString::from_vec_unchecked(s.as_bytes().to_vec()) })
        .collect();
    let strol_start = unsafe { core::arch::x86_64::_rdtsc() };
    let outputs: Vec<i64> = cstrs
        .iter()
        .map(|c| unsafe { strtoll(c.as_ptr(), std::ptr::null_mut(), 10) })
        .collect();
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    let cpu_timestamp = unsafe { core::arch::x86_64::_rdtsc() };
    // let strol_elapsed = strol_start.elapsed();
    println!(
        "strotll loop took {} cycles, avg {} cycles for {} outputs",
        cpu_timestamp - strol_start,
        (cpu_timestamp - strol_start) / outputs.len() as u64,
        outputs.len()
    );

    let cycle_start = unsafe { core::arch::x86_64::_rdtsc() };
    let start = std::time::Instant::now();
    let iters = 1;
    for i in 0..inputs.len() {
        let input = unsafe { inputs.get_unchecked(i) };
        let output = unsafe { outputs.get_unchecked(i) };
        // assert_eq!(&input.parse::<i64>().unwrap(),output);
        // let rust = std::hint::black_box(normal_atoi_mtime(input.as_bytes()));

        // println!("input{input}");

        let simd = std::hint::black_box(self::ninja_rs__atoi_i64_variable(input.as_bytes()));
        // assert_eq!(output, &rust);

        if output != &simd {
            panic!("simd test failed: input {input} C:{output:064b} != R:{simd:064b}");
        } // println!("Rust result {rust}");
        // }
    }
    let cycle_end = unsafe { core::arch::x86_64::_rdtsc() };

    let elapsed = start.elapsed().as_nanos();
    let avg = elapsed / (inputs.len() as u128 * iters);
    println!(
        "Test passed in {elapsed}ns  with avg {avg}ns, avg {2} cycles for {} iterations and took approx {} cycles",
        inputs.len() as i128 * iters as i128,
        cycle_end - cycle_start,
        (cycle_end - cycle_start) / inputs.len() as u64
    );
}
