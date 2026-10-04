use std::alloc::{GlobalAlloc, Layout, System};
use std::env;
use std::fs;
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use image::ImageFormat;

struct CountingAllocator;

static TRACK_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
            ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

mod core {
    pub mod math {
        pub type Real = f32;
    }
}

#[path = "../zircon_runtime/src/core/framework/render/environment/rgba16f.rs"]
mod production_rgba16f;

const ZCUBE_HEADER_SIZE: usize = 32;
const ZCUBE_MAGIC: &[u8; 8] = b"ZRZCUBE1";
const ZRIBL_HEADER_SIZE: usize = 120;
const ZRIBL_CHECKSUM_SIZE: usize = 32;
const ZRIBL_MAGIC: &[u8; 8] = b"ZRIBLBAK";
const LEGACY_ZRIBL_HEADER_SIZE: usize = 108;
const CURRENT_ZRIBL_FORMAT_VERSION: u32 = 4;
const CURRENT_IBL_ALGORITHM_VERSION: u64 = 2026_08_26_0008;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) == Some("upgrade-legacy-zribl") {
        assert_eq!(
            args.len(),
            4,
            "usage: warm-cache-before upgrade-legacy-zribl <input> <output>"
        );
        upgrade_legacy_zribl(&args[2], &args[3]);
        return;
    }
    assert_eq!(
        args.len(),
        5,
        "usage: warm-cache-before <hdr> <zcube> <zribl> <iterations>"
    );
    let source_bytes = fs::read(&args[1]).expect("read HDR source");
    let iterations = args[4].parse::<usize>().expect("parse iteration count");

    for iteration in 0..iterations {
        reset_allocation_counters();
        let decode_started = Instant::now();
        let image = image::load_from_memory_with_format(&source_bytes, ImageFormat::Hdr)
            .expect("decode HDR source");
        let width = image.width();
        let height = image.height();
        let rgba = image.to_rgba32f();
        let decode_elapsed_ns = decode_started.elapsed().as_nanos();
        let decode_allocation_count = allocation_count();
        let decode_allocated_bytes = allocated_bytes();
        TRACK_ALLOCATIONS.store(false, Ordering::Relaxed);
        let decode_checksum = rgba
            .pixels()
            .step_by(4096)
            .fold(0.0_f64, |sum, pixel| sum + f64::from(pixel.0[0]));
        black_box(&rgba);

        reset_allocation_counters();
        let cache_probe_started = Instant::now();
        let zcube_bytes = fs::read(&args[2]).expect("read staged zcube");
        assert_eq!(&zcube_bytes[..ZCUBE_MAGIC.len()], ZCUBE_MAGIC);
        assert_eq!(
            (zcube_bytes.len() - ZCUBE_HEADER_SIZE) % production_rgba16f::RGBA16F_TEXEL_SIZE_BYTES,
            0
        );
        let zcube_texels =
            production_rgba16f::decode_rgba16f_texels(&zcube_bytes[ZCUBE_HEADER_SIZE..]);

        let zribl_bytes = fs::read(&args[3]).expect("read staged zribl");
        assert_eq!(&zribl_bytes[..ZRIBL_MAGIC.len()], ZRIBL_MAGIC);
        let payload_offset = ZRIBL_HEADER_SIZE + ZRIBL_CHECKSUM_SIZE;
        let expected_checksum = &zribl_bytes[ZRIBL_HEADER_SIZE..payload_offset];
        let payload_bytes = &zribl_bytes[payload_offset..];
        let actual_checksum = blake3::hash(payload_bytes);
        assert_eq!(expected_checksum, actual_checksum.as_bytes());
        let derived_payload = payload_bytes.to_vec();
        let cache_probe_elapsed_ns = cache_probe_started.elapsed().as_nanos();
        let cache_probe_allocation_count = allocation_count();
        let cache_probe_allocated_bytes = allocated_bytes();
        TRACK_ALLOCATIONS.store(false, Ordering::Relaxed);

        let cache_probe_read_bytes = zcube_bytes.len() + zribl_bytes.len();
        let cache_checksum = zcube_texels
            .iter()
            .step_by(4096)
            .fold(0.0_f64, |sum, texel| sum + f64::from(texel[0]))
            + f64::from(
                derived_payload
                    .iter()
                    .step_by(4096)
                    .copied()
                    .fold(0_u8, u8::wrapping_add),
            );
        black_box((&zcube_texels, &derived_payload));

        println!(
            concat!(
                "{{\"iteration\":{iteration},\"width\":{width},\"height\":{height},",
                "\"source_bytes\":{},\"decode_elapsed_ns\":{decode_elapsed_ns},",
                "\"decode_allocation_count\":{decode_allocation_count},",
                "\"decode_allocated_bytes\":{decode_allocated_bytes},",
                "\"cache_probe_elapsed_ns\":{cache_probe_elapsed_ns},",
                "\"cache_probe_allocation_count\":{cache_probe_allocation_count},",
                "\"cache_probe_allocated_bytes\":{cache_probe_allocated_bytes},",
                "\"cache_probe_read_bytes\":{cache_probe_read_bytes},",
                "\"decode_checksum\":{decode_checksum:.9},",
                "\"cache_checksum\":{cache_checksum:.9}}}"
            ),
            source_bytes.len(),
            iteration = iteration,
            width = width,
            height = height,
            decode_elapsed_ns = decode_elapsed_ns,
            decode_allocation_count = decode_allocation_count,
            decode_allocated_bytes = decode_allocated_bytes,
            cache_probe_elapsed_ns = cache_probe_elapsed_ns,
            cache_probe_allocation_count = cache_probe_allocation_count,
            cache_probe_allocated_bytes = cache_probe_allocated_bytes,
            cache_probe_read_bytes = cache_probe_read_bytes,
            decode_checksum = decode_checksum,
            cache_checksum = cache_checksum,
        );
    }
}

fn reset_allocation_counters() {
    TRACK_ALLOCATIONS.store(false, Ordering::Relaxed);
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    TRACK_ALLOCATIONS.store(true, Ordering::Relaxed);
}

fn allocation_count() -> u64 {
    ALLOCATION_COUNT.load(Ordering::Relaxed)
}

fn allocated_bytes() -> u64 {
    ALLOCATED_BYTES.load(Ordering::Relaxed)
}

fn upgrade_legacy_zribl(input: &str, output: &str) {
    let legacy = fs::read(input).expect("read legacy zribl");
    assert_eq!(&legacy[..ZRIBL_MAGIC.len()], ZRIBL_MAGIC);
    assert_eq!(read_u32(&legacy, 8), 1, "expected legacy v1 fixture");
    let face_size = read_u32(&legacy, 20);
    let mip_count = read_u32(&legacy, 24);
    let contents = read_u32(&legacy, 28);
    let payload = &legacy[LEGACY_ZRIBL_HEADER_SIZE..];

    let mut current = vec![0_u8; ZRIBL_HEADER_SIZE + ZRIBL_CHECKSUM_SIZE];
    current[..ZRIBL_MAGIC.len()].copy_from_slice(ZRIBL_MAGIC);
    write_u32(&mut current, 8, CURRENT_ZRIBL_FORMAT_VERSION);
    write_u64(&mut current, 12, CURRENT_IBL_ALGORITHM_VERSION);
    write_u32(&mut current, 20, face_size);
    write_u32(&mut current, 24, mip_count);
    write_u32(&mut current, 28, face_size);
    write_u32(&mut current, 32, mip_count);
    write_u32(&mut current, 36, contents);
    write_u32(&mut current, 40, 1);
    current[ZRIBL_HEADER_SIZE..ZRIBL_HEADER_SIZE + ZRIBL_CHECKSUM_SIZE]
        .copy_from_slice(blake3::hash(payload).as_bytes());
    current.extend_from_slice(payload);
    fs::write(output, &current).expect("write current-wire zribl profile fixture");
    println!(
        "{{\"legacy_bytes\":{},\"current_bytes\":{},\"payload_bytes\":{},\"face_size\":{face_size},\"mip_count\":{mip_count},\"contents\":{contents},\"algorithm_version\":{CURRENT_IBL_ALGORITHM_VERSION}}}",
        legacy.len(),
        current.len(),
        payload.len(),
    );
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
