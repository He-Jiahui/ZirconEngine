//! 用旧的拼接并小写实现逐例核对类别，覆盖混合大小写、重叠关键字和非 ASCII 路由；忽略的 release 基准交错采样比较 P95。
use super::*;

fn infer_allocating(route: Option<&str>, action: Option<&str>) -> UiActionSideEffectClass {
    let text = format!(
        "{} {}",
        route.unwrap_or_default().to_ascii_lowercase(),
        action.unwrap_or_default().to_ascii_lowercase()
    );
    if text.contains("network") || text.contains("http") || text.contains("socket") {
        UiActionSideEffectClass::Network
    } else if text.contains("process") || text.contains("command") || text.contains("shell") {
        UiActionSideEffectClass::ExternalProcess
    } else if text.contains("scene") || text.contains("entity") || text.contains("world") {
        UiActionSideEffectClass::SceneMutation
    } else if text.contains("asset")
        || text.contains("file")
        || text.contains("save")
        || text.contains("load")
        || text.contains("import")
    {
        UiActionSideEffectClass::AssetIo
    } else if text.contains("editor") || text.contains("inspector") || text.contains("undo") {
        UiActionSideEffectClass::EditorMutation
    } else {
        UiActionSideEffectClass::LocalUi
    }
}

#[test]
fn runtime_interface03_batch51_52_borrowed_side_effect_inference_preserves_allocating_results() {
    for (route, action) in [
        (None, None),
        (Some("ui.popup"), Some("open")),
        (Some("Editor.Asset"), Some("ImportFile")),
        (Some("scene.entity"), Some("update_world")),
        (Some("host.Process"), Some("run_COMMAND")),
        (Some("editor.asset"), Some("NETWORK_socket_sync")),
        (Some("menu"), Some("loadout")),
        (Some("界面"), Some("HTTP_Request")),
    ] {
        assert_eq!(
            UiActionSideEffectClass::infer(route, action),
            infer_allocating(route, action),
        );
    }
}

fn p95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
#[ignore = "release-only borrowed side-effect inference benchmark"]
fn runtime_interface03_batch51_52_borrowed_side_effect_inference_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const INFERENCE_COUNT: usize = 100_000;
    const SAMPLE_COUNT: usize = 11;
    let route = "editor.asset.import.panel";
    let action = format!("NETWORK_socket_sync_{}", "payload_".repeat(64));
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..INFERENCE_COUNT {
                black_box(infer_allocating(
                    Some(black_box(route)),
                    Some(black_box(&action)),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_borrowed = || {
            let started = Instant::now();
            for _ in 0..INFERENCE_COUNT {
                black_box(UiActionSideEffectClass::infer(
                    Some(black_box(route)),
                    Some(black_box(&action)),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            borrowed_samples.push(measure_borrowed());
        } else {
            borrowed_samples.push(measure_borrowed());
            allocating_samples.push(measure_allocating());
        }
    }

    let allocating_p95_ns = p95(allocating_samples);
    let borrowed_p95_ns = p95(borrowed_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_BORROWED_SIDE_EFFECT_INFERENCE_BENCH_V1 inferences={INFERENCE_COUNT} route_bytes={} action_bytes={} samples={SAMPLE_COUNT} allocating_p95_ns={allocating_p95_ns} borrowed_p95_ns={borrowed_p95_ns}",
        route.len(),
        action.len(),
    );
    assert!(
        borrowed_p95_ns.saturating_mul(5) <= allocating_p95_ns.saturating_mul(4),
        "borrowed side-effect inference must improve P95 by at least 20%: allocating={allocating_p95_ns}ns borrowed={borrowed_p95_ns}ns",
    );
}
