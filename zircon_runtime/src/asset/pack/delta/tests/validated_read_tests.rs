use std::hint::black_box;
use std::time::Instant;

use super::{
    delta_chunk_range, zrpack_content_hash, ZrPackDeltaReader, ZrPackDeltaWriteReport,
    ZrPackDeltaWriter, ZrPackError, ZrPackInputAsset, ZrPackReader, ZrPackWriter,
};

fn fixture(asset_count: usize, payload_size: usize) -> (ZrPackDeltaWriteReport, Vec<String>) {
    let base = ZrPackWriter::write(vec![ZrPackInputAsset::new("base.bin", b"old".to_vec())])
        .expect("write base pack");
    let base = ZrPackReader::from_bytes(base.bytes).expect("open base pack");
    let paths = (0..asset_count)
        .map(|index| format!("assets/{index:04}.bin"))
        .collect::<Vec<_>>();
    let mut assets = paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let mut bytes = vec![0x5a; payload_size];
            bytes[..8].copy_from_slice(&(index as u64).to_le_bytes());
            ZrPackInputAsset::new(path.clone(), bytes)
        })
        .collect::<Vec<_>>();
    let mut alias_bytes = vec![0x5a; payload_size];
    alias_bytes[..8].copy_from_slice(&0_u64.to_le_bytes());
    assets.push(ZrPackInputAsset::new("alias.bin", alias_bytes));
    let target = ZrPackWriter::write(assets).expect("write target pack");
    let target = ZrPackReader::from_bytes(target.bytes).expect("open target pack");
    (
        ZrPackDeltaWriter::write(&base, &target).expect("write delta"),
        paths,
    )
}

#[test]
fn runtime04_delta_reader_validated_snapshot_preserves_owned_read_isolation() {
    let (mut report, paths) = fixture(2, 64);
    let reader = ZrPackDeltaReader::from_bytes(report.bytes.as_slice()).expect("open delta");
    let cloned = reader.clone();
    let expected = reader.read_changed_asset(&paths[0]).expect("read asset");
    assert_eq!(reader.read_changed_asset("alias.bin").unwrap(), expected);
    let mut owned = reader.read_changed_asset(&paths[0]).unwrap();
    owned[0] ^= 0xff;
    report.bytes[report.manifest.chunks[0].offset as usize] ^= 0xff;
    for _ in 0..3 {
        assert_eq!(reader.read_changed_asset(&paths[0]).unwrap(), expected);
        assert_eq!(cloned.read_changed_asset("alias.bin").unwrap(), expected);
    }
    assert_eq!(
        reader.read_changed_asset("absent.bin"),
        Err(ZrPackError::AssetNotFound("absent.bin".to_string()))
    );
}

#[test]
fn runtime04_delta_reader_rejects_each_corrupted_chunk_before_any_read() {
    let (report, _) = fixture(4, 64);
    assert_eq!(report.manifest.chunks.len(), 4);
    for chunk in &report.manifest.chunks {
        let mut damaged = report.bytes.clone();
        damaged[chunk.offset as usize] ^= 0xff;
        assert!(matches!(
            ZrPackDeltaReader::from_bytes(damaged),
            Err(ZrPackError::ChunkHashMismatch(_))
        ));
    }
}

// This is the retired production read path, including both lookups, checks,
// payload hashing and owned output. It compares read-time work after open.
fn legacy_read(reader: &ZrPackDeltaReader, path: &str) -> Result<Vec<u8>, ZrPackError> {
    let asset = reader
        .manifest
        .changed_asset(path)
        .ok_or_else(|| ZrPackError::AssetNotFound(path.to_string()))?;
    let chunk = reader
        .manifest
        .chunks
        .binary_search_by_key(&asset.chunk_hash, |chunk| chunk.hash)
        .ok()
        .map(|index| &reader.manifest.chunks[index])
        .ok_or_else(|| ZrPackError::MissingChunk(path.to_string()))?;
    if u64::from(chunk.size) != asset.size {
        return Err(ZrPackError::ChunkOutOfBounds(path.to_string()));
    }
    let bytes = delta_chunk_range(&reader.bytes, chunk)
        .ok_or_else(|| ZrPackError::ChunkOutOfBounds(path.to_string()))?;
    if zrpack_content_hash(bytes) != chunk.hash {
        return Err(ZrPackError::ChunkHashMismatch(path.to_string()));
    }
    Ok(bytes.to_vec())
}

fn sample(reader: &ZrPackDeltaReader, paths: &[String], legacy: bool) -> u128 {
    let started = Instant::now();
    for ordinal in 0..128 {
        let path = &paths[(ordinal * 17) % paths.len()];
        let bytes = if legacy {
            legacy_read(black_box(reader), black_box(path))
        } else {
            black_box(reader).read_changed_asset(black_box(path))
        }
        .expect("read already validated asset");
        black_box(bytes);
    }
    started.elapsed().as_nanos()
}

fn percentiles(samples: &[u128]) -> [u128; 3] {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    [50_usize, 95, 99].map(|percentile| sorted[(sorted.len() * percentile).div_ceil(100) - 1])
}

#[test]
#[ignore = "managed Windows Release evidence for repeated changed-asset reads"]
fn runtime04_delta_reader_validated_read_reuse_bench() {
    assert!(!cfg!(debug_assertions), "run in Release");
    let (report, paths) = fixture(64, 256 * 1024);
    let delta_hash = zrpack_content_hash(&report.bytes);
    let reader = ZrPackDeltaReader::from_bytes(report.bytes).expect("open delta outside timing");
    for path in &paths {
        assert_eq!(legacy_read(&reader, path), reader.read_changed_asset(path));
    }
    assert_eq!(
        reader.read_changed_asset("alias.bin"),
        reader.read_changed_asset(&paths[0])
    );
    let mut old = Vec::with_capacity(31);
    let mut new = Vec::with_capacity(31);
    for pair in 0..36 {
        let (old_sample, new_sample) = if pair % 2 == 0 {
            (
                sample(&reader, &paths, true),
                sample(&reader, &paths, false),
            )
        } else {
            let new_sample = sample(&reader, &paths, false);
            (sample(&reader, &paths, true), new_sample)
        };
        if pair >= 5 {
            old.push(old_sample);
            new.push(new_sample);
        }
    }
    let old_percentiles = percentiles(&old);
    let new_percentiles = percentiles(&new);
    println!(
        "RUNTIME04_DELTA_READER_VALIDATED_READ_REUSE_BENCH_V1 os={} arch={} crate={} chunks=64 payload_bytes=262144 reads_per_sample=128 warmups=5 samples=31 delta_hash={:02x?} old_raw_ns={:?} new_raw_ns={:?} old_p50_p95_p99_ns={:?} new_p50_p95_p99_ns={:?}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
        delta_hash,
        old,
        new,
        old_percentiles,
        new_percentiles,
    );
    assert!(
        new_percentiles[1].saturating_mul(100) <= old_percentiles[1].saturating_mul(95),
        "validated delta read P95 must be at most 95% of legacy"
    );
}
