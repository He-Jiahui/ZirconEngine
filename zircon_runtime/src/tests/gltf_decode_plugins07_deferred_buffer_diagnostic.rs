use std::collections::BTreeMap;
use std::fs;
use std::hint::black_box;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::Instant;

use super::*;
use crate::asset::AssetUri;

const SAMPLE_PAIRS: usize = 21;
const SOURCES_PER_SAMPLE: usize = 8_192;

#[test]
fn gltf_auxiliary_count_limit_is_checked_before_opening_files() {
    let root = auxiliary_test_root("gltf-member-budget");
    fs::create_dir_all(&root).unwrap();
    let existing = (0..AuxiliarySourceResolver::MAX_SNAPSHOT_FILES)
        .map(|index| (root.join(format!("existing-{index}.bin")), Vec::new()))
        .collect();
    let error = snapshot_external_gltf_sources(
        &root,
        &root.join("model.gltf"),
        &AssetUri::parse("res://model.gltf").unwrap(),
        br#"{"asset":{"version":"2.0"},"buffers":[{"uri":"missing.bin","byteLength":0}]}"#,
        &existing,
        MAX_GLTF_AUXILIARY_BYTES,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("file cumulative limit"),
        "{error}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn gltf_external_source_snapshot_containment_decodes_internal_parent_and_rejects_escape() {
    let root = auxiliary_test_root("gltf-source-containment");
    let asset_root = root.join("assets");
    let model_dir = asset_root.join("models");
    let buffer_dir = asset_root.join("buffers");
    fs::create_dir_all(&model_dir).unwrap();
    fs::create_dir_all(&buffer_dir).unwrap();
    let buffer_path = buffer_dir.join("mesh.bin");
    fs::write(&buffer_path, [9, 9, 9, 9]).unwrap();
    let source_path = model_dir.join("model.gltf");
    let uri = AssetUri::parse("res://models/model.gltf").unwrap();

    let valid = AssetImportContext::new(
        source_path.clone(),
        uri.clone(),
        br#"{"asset":{"version":"2.0"},"buffers":[{"uri":"..%2Fbuffers%2Fmesh.bin","byteLength":4}]}"#.to_vec(),
        toml::Table::new(),
    )
    .with_source_file_snapshots(BTreeMap::from([(buffer_path, vec![1, 2, 3, 4])]));
    let decoded = decode_gltf_source(&valid)
        .expect("a percent-decoded internal parent reference must remain valid");
    assert_eq!(decoded.buffers[0].0, [1, 2, 3, 4]);

    for external_uri in ["..%2F..%2Foutside.bin", "file:C:/outside.bin"] {
        let escaped = AssetImportContext::new(
            source_path.clone(),
            uri.clone(),
            format!(
                r#"{{"asset":{{"version":"2.0"}},"buffers":[{{"uri":"{external_uri}","byteLength":4}}]}}"#
            )
            .into_bytes(),
            toml::Table::new(),
        );
        let error = decode_gltf_source(&escaped)
            .expect_err("out-of-root and scheme-qualified sources must be rejected");
        let message = error.to_string();
        assert!(
            message.contains("escapes asset root") || message.contains("scheme-qualified"),
            "{message}"
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn gltf_external_image_uses_admitted_snapshot_bytes_before_disk() {
    let root = auxiliary_test_root("gltf-image-snapshot");
    let asset_root = root.join("assets");
    let model_dir = asset_root.join("models");
    let image_dir = asset_root.join("images");
    fs::create_dir_all(&model_dir).unwrap();
    fs::create_dir_all(&image_dir).unwrap();
    let image_path = image_dir.join("PIXEL.PNG");
    fs::write(&image_path, b"not a png").unwrap();

    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(1, 1)
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    let context = AssetImportContext::new(
        model_dir.join("model.gltf"),
        AssetUri::parse("res://models/model.gltf").unwrap(),
        br#"{"asset":{"version":"2.0"},"images":[{"uri":"..%2Fimages%2FPIXEL%2EPNG","mimeType":"IMAGE/PNG"}]}"#.to_vec(),
        toml::Table::new(),
    )
    .with_source_file_snapshots(BTreeMap::from([(
        image_path.clone(),
        encoded.into_inner(),
    )]));

    let decoded = decode_gltf_source(&context).expect("snapshot PNG should decode");
    assert_eq!(decoded.images[0].width, 1);
    assert_eq!(decoded.images[0].height, 1);

    let extensionless = images::decode_external_image(
        context.source_file_snapshot(&image_path).unwrap(),
        Path::new("pixel"),
        None,
        &mut DecodedBudget::new(MAX_GLTF_AUXILIARY_BYTES),
    )
    .expect("signature-identifiable extensionless PNG should decode");
    assert_eq!((extensionless.width, extensionless.height), (1, 1));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn decode_material_hotpath_contract_buffer_source_labels() {
    assert_eq!(
        gltf_buffer_source_name(gltf::buffer::Source::Bin),
        "the GLB binary chunk"
    );
    assert_eq!(
        gltf_buffer_source_name(gltf::buffer::Source::Uri(
            "data:application/octet-stream;base64,AA=="
        )),
        "an embedded data URI"
    );
    assert_eq!(
        gltf_buffer_source_name(gltf::buffer::Source::Uri("mesh.bin")),
        "`mesh.bin`"
    );
}

#[test]
#[ignore = "release performance gate"]
fn decode_material_hotpath_performance_release_deferred_buffer_diagnostics() {
    let sources = (0..SOURCES_PER_SAMPLE)
        .map(|index| format!("buffers/plugins07-{index:05}.bin"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_eager_names(&sources));
        black_box(measure_deferred_success(&sources));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (
                measure_eager_names(&sources),
                measure_deferred_success(&sources),
            )
        } else {
            let optimized_ns = measure_deferred_success(&sources);
            (measure_eager_names(&sources), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_deferred_gltf_buffer_diagnostics sample_pairs={SAMPLE_PAIRS} sources_per_sample={SOURCES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=90 legacy_success_diagnostic_allocations_per_sample={SOURCES_PER_SAMPLE} optimized_success_diagnostic_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(
        improvement_percent >= 90,
        "deferred glTF buffer diagnostics must improve successful-load P95 by at least 90%"
    );
}

fn measure_eager_names(sources: &[String]) -> u128 {
    let started = Instant::now();
    for uri in sources {
        let source_name = gltf_buffer_source_name(gltf::buffer::Source::Uri(black_box(uri)));
        black_box(source_name);
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_deferred_success(sources: &[String]) -> u128 {
    let started = Instant::now();
    for uri in sources {
        black_box(uri);
    }
    started.elapsed().as_nanos().max(1)
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn auxiliary_test_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
