//! 旧分配式实现作为语义基线，逐例验证字段路径优先于 URI 后缀并覆盖大小写、query/fragment 和尾斜线；release 基准比较 P95。
use super::*;

fn legacy_infer_from_path(path: &str) -> Option<UiResourceKind> {
    let normalized = path.to_ascii_lowercase();
    let segments: Vec<&str> = normalized
        .split(['.', '_', '-', '/', ':', '#', '[', ']'])
        .filter(|segment| !segment.is_empty())
        .collect();

    for index in (0..segments.len()).rev() {
        if index > 0 {
            if let Some(kind) =
                legacy_infer_from_path_name(&format!("{}_{}", segments[index - 1], segments[index]))
            {
                return Some(kind);
            }
        }
        if let Some(kind) = legacy_infer_from_path_name(segments[index]) {
            return Some(kind);
        }
    }
    None
}

fn legacy_infer_from_path_name(name: &str) -> Option<UiResourceKind> {
    match name {
        "font" | "font_asset" => Some(UiResourceKind::Font),
        "image" | "icon" | "background_image" => Some(UiResourceKind::Image),
        "media" | "video" | "audio" => Some(UiResourceKind::Media),
        "asset" | "resource" => Some(UiResourceKind::GenericAsset),
        _ => None,
    }
}

fn legacy_infer_from_uri_extension(uri: &str) -> UiResourceKind {
    let uri = uri.to_ascii_lowercase();
    let resource_path = uri
        .split(['#', '?'])
        .next()
        .unwrap_or(uri.as_str())
        .trim_end_matches('/');
    if resource_path.ends_with(".font.toml") {
        return UiResourceKind::Font;
    }
    match resource_path.rsplit_once('.') {
        Some((_, "ttf" | "otf" | "woff" | "woff2")) => UiResourceKind::Font,
        Some((_, "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tga" | "svg" | "ico")) => {
            UiResourceKind::Image
        }
        Some((_, "mp3" | "ogg" | "wav" | "flac" | "mp4" | "webm" | "mov")) => UiResourceKind::Media,
        _ => UiResourceKind::GenericAsset,
    }
}

fn legacy_infer_from_path_and_uri(path: &str, uri: &str) -> UiResourceKind {
    legacy_infer_from_path(path).unwrap_or_else(|| legacy_infer_from_uri_extension(uri))
}

#[test]
fn borrowed_resource_kind_inference_preserves_legacy_results() {
    for (path, uri) in [
        ("root.props.ICON", "asset://audio/click.mp3"),
        ("root.background-image", "asset://icons/fallback.svg"),
        ("fonts/my_font_asset", "asset://fonts/fallback.ttf"),
        ("root.unknown", "asset://audio/click.MP3?cache=1#fragment"),
        ("ROOT/RESOURCE", "asset://unknown.bin"),
        ("汉字", "asset://images/FALLBACK.PNG"),
        ("", "asset://fonts/theme.FONT.TOML/"),
    ] {
        assert_eq!(
            UiResourceKind::infer_from_path_and_uri(path, uri),
            legacy_infer_from_path_and_uri(path, uri),
            "path={path:?} uri={uri:?}",
        );
    }
}

#[test]
#[ignore = "release-only borrowed resource kind inference benchmark"]
fn runtime_interface03_batch54_borrowed_resource_kind_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const ITERATIONS: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let path = "assets/editor/background-image/high-contrast-panel-resource";
    let uri = "asset://textures/editor-panel-fallback.WEBP?revision=42#preview";
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_infer_from_path_and_uri(
                    black_box(path),
                    black_box(uri),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(UiResourceKind::infer_from_path_and_uri(
                    black_box(path),
                    black_box(uri),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BORROWED_RESOURCE_KIND_BENCH_V1 iterations={ITERATIONS} samples={SAMPLE_COUNT} legacy_p95_ns={} optimized_p95_ns={}",
        legacy_samples[p95], optimized_samples[p95],
    );
    assert!(
        optimized_samples[p95].saturating_mul(5) <= legacy_samples[p95].saturating_mul(4),
        "borrowed resource kind inference must improve P95 by at least 20%: legacy={}ns optimized={}ns",
        legacy_samples[p95],
        optimized_samples[p95],
    );
}
