use zircon_runtime_interface::ui::surface::{
    UiDebugOverlayPrimitive, UiDebugOverlayPrimitiveKind, UiRenderVisualizerOverlay,
    UiRenderVisualizerOverlayKind, UiSurfaceDebugSnapshot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EditorUiDebugReflectorOverlayState {
    pub selected_frame: bool,
    pub clip_frame: bool,
    pub wireframe: bool,
    pub hit_grid: bool,
    pub hit_path: bool,
    pub rejected_bounds: bool,
    pub overdraw: bool,
    pub material_batches: bool,
    pub text_debug: bool,
    pub resource_atlas: bool,
    pub damage: bool,
}

impl Default for EditorUiDebugReflectorOverlayState {
    fn default() -> Self {
        Self {
            selected_frame: true,
            clip_frame: true,
            wireframe: true,
            hit_grid: true,
            hit_path: true,
            rejected_bounds: true,
            overdraw: true,
            material_batches: true,
            text_debug: true,
            resource_atlas: true,
            damage: true,
        }
    }
}

impl EditorUiDebugReflectorOverlayState {
    pub(crate) fn allows(self, primitive: &UiDebugOverlayPrimitive) -> bool {
        self.allows_kind(primitive.kind)
    }

    fn allows_kind(self, kind: UiDebugOverlayPrimitiveKind) -> bool {
        match kind {
            UiDebugOverlayPrimitiveKind::SelectedFrame => self.selected_frame,
            UiDebugOverlayPrimitiveKind::ClipFrame => self.clip_frame,
            UiDebugOverlayPrimitiveKind::Wireframe => self.wireframe,
            UiDebugOverlayPrimitiveKind::HitCell => self.hit_grid,
            UiDebugOverlayPrimitiveKind::HitPath => self.hit_path,
            UiDebugOverlayPrimitiveKind::RejectedBounds => self.rejected_bounds,
            UiDebugOverlayPrimitiveKind::OverdrawCell => self.overdraw,
            UiDebugOverlayPrimitiveKind::MaterialBatchBounds => self.material_batches,
            UiDebugOverlayPrimitiveKind::TextGlyphBounds
            | UiDebugOverlayPrimitiveKind::TextBaseline => self.text_debug,
            UiDebugOverlayPrimitiveKind::ResourceAtlas => self.resource_atlas,
            UiDebugOverlayPrimitiveKind::DamageRegion => self.damage,
        }
    }

    pub(crate) fn primitives_from_snapshot(
        self,
        snapshot: &UiSurfaceDebugSnapshot,
    ) -> Vec<UiDebugOverlayPrimitive> {
        let shared_count = snapshot
            .overlay_primitives
            .iter()
            .filter(|primitive| self.allows(primitive))
            .count();
        let visualizer_count = snapshot
            .render_batches
            .visualizer
            .overlays
            .iter()
            .filter(|overlay| self.allows_visualizer_overlay(overlay))
            .count();
        let synthesize_damage = self.damage
            && snapshot.damage.damage_region.is_some()
            && !snapshot
                .overlay_primitives
                .iter()
                .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::DamageRegion);
        let mut primitives =
            Vec::with_capacity(shared_count + visualizer_count + usize::from(synthesize_damage));
        primitives.extend(
            snapshot
                .overlay_primitives
                .iter()
                .filter(|primitive| self.allows(primitive))
                .cloned(),
        );
        primitives.extend(
            snapshot
                .render_batches
                .visualizer
                .overlays
                .iter()
                .filter_map(|overlay| self.primitive_from_visualizer_overlay(overlay)),
        );
        if synthesize_damage {
            if let Some(frame) = snapshot.damage.damage_region {
                primitives.push(UiDebugOverlayPrimitive {
                    kind: UiDebugOverlayPrimitiveKind::DamageRegion,
                    node_id: None,
                    frame,
                    label: Some("damage".to_string()),
                    severity: Some("warning".to_string()),
                });
            }
        }
        primitives
    }

    fn primitive_from_visualizer_overlay(
        self,
        overlay: &UiRenderVisualizerOverlay,
    ) -> Option<UiDebugOverlayPrimitive> {
        let kind = visualizer_overlay_kind(overlay.kind);
        self.allows_kind(kind).then(|| UiDebugOverlayPrimitive {
            kind,
            node_id: overlay.node_id,
            frame: overlay.frame,
            label: visualizer_overlay_label(overlay),
            severity: None,
        })
    }

    fn allows_visualizer_overlay(self, overlay: &UiRenderVisualizerOverlay) -> bool {
        self.allows_kind(visualizer_overlay_kind(overlay.kind))
    }
}

fn visualizer_overlay_kind(kind: UiRenderVisualizerOverlayKind) -> UiDebugOverlayPrimitiveKind {
    match kind {
        UiRenderVisualizerOverlayKind::Wireframe => UiDebugOverlayPrimitiveKind::Wireframe,
        UiRenderVisualizerOverlayKind::ClipScissor => UiDebugOverlayPrimitiveKind::ClipFrame,
        UiRenderVisualizerOverlayKind::BatchBounds => {
            UiDebugOverlayPrimitiveKind::MaterialBatchBounds
        }
        UiRenderVisualizerOverlayKind::OverdrawHeat => UiDebugOverlayPrimitiveKind::OverdrawCell,
        UiRenderVisualizerOverlayKind::TextGlyphBounds => {
            UiDebugOverlayPrimitiveKind::TextGlyphBounds
        }
        UiRenderVisualizerOverlayKind::TextBaseline => UiDebugOverlayPrimitiveKind::TextBaseline,
        UiRenderVisualizerOverlayKind::ResourceAtlas => UiDebugOverlayPrimitiveKind::ResourceAtlas,
    }
}

fn visualizer_overlay_label(overlay: &UiRenderVisualizerOverlay) -> Option<String> {
    overlay.label.clone().or_else(|| {
        overlay
            .batch_index
            .map(|batch_index| format!("batch:{batch_index}"))
            .or_else(|| {
                overlay
                    .paint_index
                    .map(|paint_index| format!("paint:{paint_index}"))
            })
    })
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use zircon_runtime_interface::ui::layout::UiFrame;

    const BENCHMARK_OVERLAY_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 64;

    #[test]
    fn overlay_capacity_preserves_filtered_order_and_damage_semantics() {
        let mut snapshot = UiSurfaceDebugSnapshot::default();
        snapshot.overlay_primitives = vec![
            shared_primitive(UiDebugOverlayPrimitiveKind::SelectedFrame, "selected"),
            shared_primitive(UiDebugOverlayPrimitiveKind::HitCell, "hit"),
        ];
        snapshot.render_batches.visualizer.overlays = vec![
            visualizer_overlay(UiRenderVisualizerOverlayKind::BatchBounds, Some("batch")),
            visualizer_overlay(UiRenderVisualizerOverlayKind::ResourceAtlas, Some("atlas")),
        ];
        snapshot.damage.damage_region = Some(UiFrame::new(1.0, 2.0, 3.0, 4.0));
        let state = EditorUiDebugReflectorOverlayState {
            hit_grid: false,
            resource_atlas: false,
            ..EditorUiDebugReflectorOverlayState::default()
        };

        assert_eq!(
            state.primitives_from_snapshot(&snapshot),
            retired_primitives_from_snapshot(state, &snapshot)
        );

        snapshot.overlay_primitives.push(shared_primitive(
            UiDebugOverlayPrimitiveKind::DamageRegion,
            "existing damage",
        ));
        assert_eq!(
            state.primitives_from_snapshot(&snapshot),
            retired_primitives_from_snapshot(state, &snapshot)
        );
    }

    #[test]
    fn overlay_capacity_preallocates_exact_output_and_skips_eager_labels() {
        let source = include_str!("overlay.rs");
        let production = source
            .split_once("pub(crate) fn primitives_from_snapshot")
            .expect("overlay collection function")
            .1
            .split_once("\n    fn primitive_from_visualizer_overlay")
            .expect("overlay collection function end")
            .0;
        let production_module = source
            .split_once("#[cfg(test)]")
            .expect("production module end")
            .0;

        assert!(production.contains("Vec::with_capacity"));
        assert!(!production.contains("collect::<Vec"));
        assert!(production_module.contains(".then(|| UiDebugOverlayPrimitive"));
        assert!(!production_module.contains(".then_some(primitive)"));

        let mut snapshot = UiSurfaceDebugSnapshot::default();
        snapshot.render_batches.visualizer.overlays = (0..4_000)
            .map(|index| {
                let kind = if index % 4 == 0 {
                    UiRenderVisualizerOverlayKind::BatchBounds
                } else {
                    UiRenderVisualizerOverlayKind::ResourceAtlas
                };
                visualizer_overlay(kind, Some("capacity"))
            })
            .collect();
        let state = benchmark_state();
        let primitives = state.primitives_from_snapshot(&snapshot);

        assert_eq!(primitives.len(), 1_000);
        assert_eq!(primitives.capacity(), primitives.len());

        snapshot.render_batches.visualizer.overlays.clear();
        let empty = state.primitives_from_snapshot(&snapshot);
        assert!(empty.is_empty());
        assert_eq!(empty.capacity(), 0);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn overlay_capacity_release_benchmark() {
        let mut snapshot = UiSurfaceDebugSnapshot::default();
        let long_label = "debug-overlay-label/".to_string() + &"x".repeat(112);
        snapshot.render_batches.visualizer.overlays = (0..BENCHMARK_OVERLAY_COUNT)
            .map(|index| {
                let kind = if index % 4 == 0 {
                    UiRenderVisualizerOverlayKind::BatchBounds
                } else {
                    UiRenderVisualizerOverlayKind::ResourceAtlas
                };
                visualizer_overlay(kind, Some(&long_label))
            })
            .collect();
        let state = benchmark_state();
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_overlay_collection(|| {
                    retired_primitives_from_snapshot(state, &snapshot)
                }));
                optimized_samples.push(measure_overlay_collection(|| {
                    state.primitives_from_snapshot(&snapshot)
                }));
            } else {
                optimized_samples.push(measure_overlay_collection(|| {
                    state.primitives_from_snapshot(&snapshot)
                }));
                retired_samples.push(measure_overlay_collection(|| {
                    retired_primitives_from_snapshot(state, &snapshot)
                }));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "EDITOR25_OVERLAY_EXACT_CAPACITY_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
overlays={BENCHMARK_OVERLAY_COUNT} accepted_overlays=1024 \
retired_label_clones_per_collection=4096 optimized_label_clones_per_collection=1024 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(65),
            "exact-capacity lazy overlay collection must reduce P95 by at least 35%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn benchmark_state() -> EditorUiDebugReflectorOverlayState {
        EditorUiDebugReflectorOverlayState {
            selected_frame: false,
            clip_frame: false,
            wireframe: false,
            hit_grid: false,
            hit_path: false,
            rejected_bounds: false,
            overdraw: false,
            material_batches: true,
            text_debug: false,
            resource_atlas: false,
            damage: false,
        }
    }

    fn shared_primitive(kind: UiDebugOverlayPrimitiveKind, label: &str) -> UiDebugOverlayPrimitive {
        UiDebugOverlayPrimitive {
            kind,
            node_id: None,
            frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
            label: Some(label.to_string()),
            severity: None,
        }
    }

    fn visualizer_overlay(
        kind: UiRenderVisualizerOverlayKind,
        label: Option<&str>,
    ) -> UiRenderVisualizerOverlay {
        UiRenderVisualizerOverlay {
            kind,
            frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
            node_id: None,
            paint_index: None,
            batch_index: None,
            label: label.map(str::to_string),
            color: None,
            intensity: 1.0,
        }
    }

    fn measure_overlay_collection(
        mut collect: impl FnMut() -> Vec<UiDebugOverlayPrimitive>,
    ) -> Duration {
        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            black_box(collect());
        }
        started.elapsed()
    }

    fn retired_primitives_from_snapshot(
        state: EditorUiDebugReflectorOverlayState,
        snapshot: &UiSurfaceDebugSnapshot,
    ) -> Vec<UiDebugOverlayPrimitive> {
        let mut primitives = snapshot
            .overlay_primitives
            .iter()
            .filter(|primitive| state.allows(primitive))
            .cloned()
            .collect::<Vec<_>>();
        primitives.extend(
            snapshot
                .render_batches
                .visualizer
                .overlays
                .iter()
                .filter_map(|overlay| retired_visualizer_overlay(state, overlay)),
        );
        if state.damage
            && !primitives
                .iter()
                .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::DamageRegion)
        {
            if let Some(frame) = snapshot.damage.damage_region {
                primitives.push(UiDebugOverlayPrimitive {
                    kind: UiDebugOverlayPrimitiveKind::DamageRegion,
                    node_id: None,
                    frame,
                    label: Some("damage".to_string()),
                    severity: Some("warning".to_string()),
                });
            }
        }
        primitives
    }

    fn retired_visualizer_overlay(
        state: EditorUiDebugReflectorOverlayState,
        overlay: &UiRenderVisualizerOverlay,
    ) -> Option<UiDebugOverlayPrimitive> {
        let kind = match overlay.kind {
            UiRenderVisualizerOverlayKind::Wireframe => UiDebugOverlayPrimitiveKind::Wireframe,
            UiRenderVisualizerOverlayKind::ClipScissor => UiDebugOverlayPrimitiveKind::ClipFrame,
            UiRenderVisualizerOverlayKind::BatchBounds => {
                UiDebugOverlayPrimitiveKind::MaterialBatchBounds
            }
            UiRenderVisualizerOverlayKind::OverdrawHeat => {
                UiDebugOverlayPrimitiveKind::OverdrawCell
            }
            UiRenderVisualizerOverlayKind::TextGlyphBounds => {
                UiDebugOverlayPrimitiveKind::TextGlyphBounds
            }
            UiRenderVisualizerOverlayKind::TextBaseline => {
                UiDebugOverlayPrimitiveKind::TextBaseline
            }
            UiRenderVisualizerOverlayKind::ResourceAtlas => {
                UiDebugOverlayPrimitiveKind::ResourceAtlas
            }
        };
        let primitive = UiDebugOverlayPrimitive {
            kind,
            node_id: overlay.node_id,
            frame: overlay.frame,
            label: visualizer_overlay_label(overlay),
            severity: None,
        };
        state.allows(&primitive).then_some(primitive)
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
