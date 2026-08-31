use super::super::RetainedEditorHost;
use zircon_runtime_interface::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata,
};

fn scene_node_reference(id: u64) -> String {
    const PREFIX: &str = "object://scene/node/";
    let mut reference = String::with_capacity(PREFIX.len() + 20);
    reference.push_str(PREFIX);
    push_u64_decimal(&mut reference, id);
    reference
}

fn push_u64_decimal(output: &mut String, mut value: u64) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for digit in &digits[start..] {
        output.push(char::from(*digit));
    }
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn inspector_reference_pointer_event(
        &mut self,
        kind: i32,
        button: i32,
        _x: f32,
        _y: f32,
        _width: f32,
        _height: f32,
    ) {
        if button == 1 && kind == 2 {
            self.active_object_drag_payload = None;
            return;
        }
        if kind != 0 || button != 1 {
            return;
        }

        self.active_asset_drag_payload = None;
        self.active_scene_drag_payload = None;
        self.focus_callback_source_window();
        self.active_object_drag_payload = self.object_drag_payload_from_selected_inspector();
        if let Some(summary) = self
            .active_object_drag_payload
            .as_ref()
            .and_then(UiDragPayload::source_summary)
        {
            self.set_status_line(format!("Object drag source: {summary}"));
        }
    }

    fn object_drag_payload_from_selected_inspector(&self) -> Option<UiDragPayload> {
        let inspector = self.runtime.editor_snapshot().inspector?;
        let reference = scene_node_reference(inspector.id);
        Some(
            UiDragPayload::new(UiDragPayloadKind::Object, reference.clone()).with_source(
                UiDragSourceMetadata {
                    source_surface: "inspector".to_string(),
                    source_control_id: "InspectorHeaderPanel".to_string(),
                    locator: Some(reference),
                    display_name: Some(inspector.name),
                    asset_kind: Some("Scene Object".to_string()),
                    ..UiDragSourceMetadata::default()
                },
            ),
        )
    }
}

#[cfg(test)]
mod optimization_batch_fh_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const REFERENCES_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fh_editor394_object_reference_preserves_bytes() {
        for id in [0, 1, 42, u64::MAX] {
            assert_eq!(scene_node_reference(id), legacy_scene_node_reference(id));
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fh_editor394_direct_object_reference_benchmark() {
        let id = 18_446_744_073_709_551_615;
        for _ in 0..4 {
            black_box(measure(legacy_scene_node_reference, id));
            black_box(measure(scene_node_reference, id));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure(legacy_scene_node_reference, id));
                optimized_samples.push(measure(scene_node_reference, id));
            } else {
                optimized_samples.push(measure(scene_node_reference, id));
                legacy_samples.push(measure(legacy_scene_node_reference, id));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn legacy_scene_node_reference(id: u64) -> String {
        format!("object://scene/node/{id}")
    }

    fn measure(mut build: impl FnMut(u64) -> String, id: u64) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..REFERENCES_PER_SAMPLE {
            checksum = checksum.wrapping_add(black_box(build(black_box(id))).len());
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR394_DIRECT_OBJECT_REFERENCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} references_per_sample={REFERENCES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "optimized p95 {optimized_p95}ns must be at most 75% of legacy p95 {legacy_p95}ns"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
