use super::*;
use crate::pack::ZrPackInputAsset;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[test]
fn delta_export_preserves_written_pack_and_report_after_verification() {
    let root = unique_temp_dir("delta-export-verification");
    let manifest_path = root.join("assets.json");
    let base_path = root.join("base.zrpack");
    let pack_path = root.join("target.zrpack");
    let delta_path = root.join("target.zrpd");
    let report_path = root.join("report.json");
    fs::write(root.join("shared.bin"), b"shared").unwrap();
    fs::write(root.join("changed.bin"), b"after").unwrap();
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("assets/changed.bin", b"before".to_vec()),
        ZrPackInputAsset::new("assets/removed.bin", b"removed".to_vec()),
        ZrPackInputAsset::new("assets/shared.bin", b"shared".to_vec()),
    ])
    .unwrap();
    fs::write(&base_path, &base.bytes).unwrap();
    fs::write(
        &manifest_path,
        serde_json::json!({
            "roots": ["assets/changed.bin", "assets/shared.bin"],
            "assets": [
                {"path": "assets/changed.bin", "source": "changed.bin", "dependencies": []},
                {"path": "assets/shared.bin", "source": "shared.bin", "dependencies": []}
            ]
        })
        .to_string(),
    )
    .unwrap();

    let exit = run([
        os("--profile"),
        os("windows-release"),
        os("--manifest"),
        manifest_path.into_os_string(),
        os("--pack"),
        pack_path.clone().into_os_string(),
        os("--previous-pack"),
        base_path.clone().into_os_string(),
        os("--delta-pack"),
        delta_path.clone().into_os_string(),
        os("--report"),
        report_path.clone().into_os_string(),
    ])
    .unwrap();
    assert_eq!(exit, ExitCode::SUCCESS);

    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(report_path).unwrap()).unwrap();
    assert_eq!(report["fatal"], false);
    assert_eq!(report["asset_count"], 2);
    assert_eq!(report["delta_asset_count"], 1);
    assert_eq!(
        report["delta_removed_assets"],
        serde_json::json!(["assets/removed.bin"])
    );
    assert_eq!(
        report["delta_reused_assets"],
        serde_json::json!(["assets/shared.bin"])
    );
    assert_eq!(report["delta_apply_verified"], true);

    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_bytes = fs::read(pack_path).unwrap();
    let delta_reader = ZrPackDeltaReader::from_bytes(fs::read(delta_path).unwrap()).unwrap();
    assert_eq!(
        delta_reader
            .read_changed_asset("assets/changed.bin")
            .unwrap(),
        b"after"
    );
    assert_eq!(
        delta_reader.apply_to_base(&base_reader).unwrap().bytes,
        target_bytes
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "Windows-native Release performance evidence"]
fn runtime04_pack_export_delta_reader_move_bench() {
    const ASSET_COUNT: usize = 1_024;
    const ASSET_BYTES: usize = 16 * 1_024;
    const WARMUP_COUNT: usize = 5;
    const SAMPLE_COUNT: usize = 31;

    let root = unique_temp_dir("delta-reader-move-bench");
    let base_path = root.join("base.zrpack");
    let mut base_assets = Vec::with_capacity(ASSET_COUNT);
    let mut target_assets = Vec::with_capacity(ASSET_COUNT);
    for index in 0..ASSET_COUNT {
        let path = format!("assets/{index:04}.bin");
        let mut base_bytes = vec![0; ASSET_BYTES];
        base_bytes[..4].copy_from_slice(&(index as u32).to_le_bytes());
        let mut target_bytes = base_bytes.clone();
        target_bytes[4] = 1;
        base_assets.push(ZrPackInputAsset::new(path.clone(), base_bytes));
        target_assets.push(ZrPackInputAsset::new(path, target_bytes));
    }
    let base_pack = ZrPackWriter::write(base_assets).unwrap();
    let target_pack = ZrPackWriter::write(target_assets).unwrap();
    fs::write(&base_path, &base_pack.bytes).unwrap();
    let base_reader = ZrPackReader::from_bytes(base_pack.bytes).unwrap();
    let target_bytes = target_pack.bytes;
    let target_reader = ZrPackReader::from_bytes(target_bytes.clone()).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    assert_eq!(delta.changed_assets.len(), ASSET_COUNT);
    assert!(delta.bytes.len() >= ASSET_COUNT * ASSET_BYTES);

    let args = super::super::args::PackArgs {
        profile: "windows-release".to_string(),
        manifest: root.join("assets.json"),
        pack: root.join("target.zrpack"),
        previous_pack: Some(base_path),
        delta_pack: Some(root.join("target.zrpd")),
        report: None,
        stage_output: None,
        pretty: false,
        determinism_check: false,
    };

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut clone_only_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut product_stage_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..(WARMUP_COUNT + SAMPLE_COUNT) {
        let (legacy, optimized) = if sample % 2 == 0 {
            (
                measure_reader_handoff(&delta.bytes, true),
                measure_reader_handoff(&delta.bytes, false),
            )
        } else {
            let optimized = measure_reader_handoff(&delta.bytes, false);
            let legacy = measure_reader_handoff(&delta.bytes, true);
            (legacy, optimized)
        };
        let clone_only = measure_clone_only(&delta.bytes);
        let start = Instant::now();
        let report = write_delta_pack_if_requested(&args, &target_bytes)
            .unwrap()
            .unwrap();
        let product_stage = start.elapsed().as_nanos();
        assert!(report.apply_verified);
        assert_eq!(report.changed_assets.len(), ASSET_COUNT);
        if sample < WARMUP_COUNT {
            continue;
        }
        legacy_samples.push(legacy);
        optimized_samples.push(optimized);
        clone_only_samples.push(clone_only);
        product_stage_samples.push(product_stage);
    }

    let (legacy_p50, legacy_p95, legacy_p99) = percentiles(&mut legacy_samples.clone());
    let (optimized_p50, optimized_p95, optimized_p99) = percentiles(&mut optimized_samples.clone());
    let (clone_p50, clone_p95, clone_p99) = percentiles(&mut clone_only_samples.clone());
    let (stage_p50, stage_p95, stage_p99) = percentiles(&mut product_stage_samples.clone());
    eprintln!(
        "RUNTIME04_PACK_EXPORT_DELTA_READER_MOVE_BENCH_V1 assets={ASSET_COUNT} delta_bytes={} warmups={WARMUP_COUNT} samples={SAMPLE_COUNT} clone_only_p50_ns={clone_p50} clone_only_p95_ns={clone_p95} clone_only_p99_ns={clone_p99} legacy_handoff_p50_ns={legacy_p50} legacy_handoff_p95_ns={legacy_p95} legacy_handoff_p99_ns={legacy_p99} optimized_handoff_p50_ns={optimized_p50} optimized_handoff_p95_ns={optimized_p95} optimized_handoff_p99_ns={optimized_p99} product_delta_stage_p50_ns={stage_p50} product_delta_stage_p95_ns={stage_p95} product_delta_stage_p99_ns={stage_p99} raw_clone_only_ns={clone_only_samples:?} raw_legacy_handoff_ns={legacy_samples:?} raw_optimized_handoff_ns={optimized_samples:?} raw_product_delta_stage_ns={product_stage_samples:?} os={} arch={} package_version={}",
        delta.bytes.len(),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION")
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "delta reader handoff P95 must be at most 95% of the legacy clone path"
    );
    let _ = fs::remove_dir_all(root);
}

fn measure_clone_only(delta_bytes: &[u8]) -> u128 {
    let owned = delta_bytes.to_vec();
    let start = Instant::now();
    let copy = black_box(owned.clone());
    let elapsed = start.elapsed().as_nanos();
    assert_eq!(copy.len(), delta_bytes.len());
    assert_ne!(copy.as_ptr(), owned.as_ptr());
    black_box(&owned);
    elapsed
}

fn measure_reader_handoff(delta_bytes: &[u8], legacy_clone: bool) -> u128 {
    let owned = delta_bytes.to_vec();
    if legacy_clone {
        let start = Instant::now();
        let reader = ZrPackDeltaReader::from_bytes(owned.clone()).unwrap();
        black_box(reader.manifest().changed_assets.len());
        let elapsed = start.elapsed().as_nanos();
        black_box(&owned);
        elapsed
    } else {
        let start = Instant::now();
        let reader = ZrPackDeltaReader::from_bytes(owned).unwrap();
        black_box(reader.manifest().changed_assets.len());
        start.elapsed().as_nanos()
    }
}

fn percentiles(samples: &mut [u128]) -> (u128, u128, u128) {
    samples.sort_unstable();
    let rank = |percent: usize| samples[(samples.len() * percent).div_ceil(100) - 1];
    (rank(50), rank(95), rank(99))
}

fn os(value: impl Into<OsString>) -> OsString {
    value.into()
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon-export-pack-{label}-{nanos}"));
    fs::create_dir_all(&root).unwrap();
    root
}
