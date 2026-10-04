use super::*;

fn material_resource_id_formatting(material_id: &str, variant: Option<&str>) -> String {
    variant
        .map(|variant| format!("{material_id}#{variant}"))
        .unwrap_or_else(|| material_id.to_string())
}

#[test]
fn runtime_interface03_batch55_61_material_key_preserves_formatted_output() {
    for (material_id, variant) in [
        ("material", None),
        ("material", Some("default")),
        ("", Some("variant")),
        ("material", Some("")),
        ("material#family", Some("variant#nested")),
        ("material_\u{754c}\u{9762}", Some("\u{53d8}\u{4f53}")),
    ] {
        assert_eq!(
            material_resource_id(material_id, variant),
            material_resource_id_formatting(material_id, variant),
        );
    }

    let payload = UiMaterialBrushPayload {
        material_id: "editor.surface".to_string(),
        variant: Some("selected".to_string()),
        revision: Some(7),
        resource_state: UiRenderResourceState {
            atlas_page: Some(3),
            ..UiRenderResourceState::default()
        },
        fallback_color: Some("#ffffff".to_string()),
    };
    let key = payload.resource_key();
    assert_eq!(key.id, "editor.surface#selected");
    assert_eq!(key.revision, Some(7));
    assert_eq!(key.atlas_page, Some(3));
}

#[test]
#[ignore = "release-only single-buffer material resource key benchmark"]
fn runtime_interface03_batch55_61_single_buffer_material_resource_key_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BUILD_COUNT: usize = 250_000;
    const SAMPLE_COUNT: usize = 11;
    let material_id = "runtime_interface_material_family_".repeat(8);
    let variant = "selected_hover_variant".repeat(4);
    let mut formatting_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut single_buffer_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_formatting = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(material_resource_id_formatting(
                    black_box(&material_id),
                    Some(black_box(&variant)),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_single_buffer = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(material_resource_id(
                    black_box(&material_id),
                    Some(black_box(&variant)),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            formatting_samples.push(measure_formatting());
            single_buffer_samples.push(measure_single_buffer());
        } else {
            single_buffer_samples.push(measure_single_buffer());
            formatting_samples.push(measure_formatting());
        }
    }

    formatting_samples.sort_unstable();
    single_buffer_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_BUFFER_MATERIAL_KEY_BENCH_V1 builds={BUILD_COUNT} bytes={} samples={SAMPLE_COUNT} formatting_p95_ns={} single_buffer_p95_ns={}",
        material_id.len() + 1 + variant.len(),
        formatting_samples[p95],
        single_buffer_samples[p95],
    );
    assert!(
        single_buffer_samples[p95].saturating_mul(5) <= formatting_samples[p95].saturating_mul(4),
        "single-buffer material resource IDs must improve P95 by at least 20%: formatting={}ns single_buffer={}ns",
        formatting_samples[p95],
        single_buffer_samples[p95],
    );
}
