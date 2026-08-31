use crate::ui::asset_editor::presentation::{
    UiAssetEditorPreviewCanvasNode, UiAssetEditorPreviewCanvasSlotTarget,
};
use crate::ui::retained_host::ui_perf::{UiPerfCounter, record_current_ui_perf_counter};

use super::super::{
    palette::PaletteInsertMode,
    palette_drop::build_palette_drag_slot_target_overlays,
    palette_target_chooser::UiAssetPaletteTargetChooser,
    preview_host::UiAssetPreviewHost,
    preview_projection::{UiAssetPreviewProjection, build_preview_projection},
    ui_asset_editor_session::UiAssetEditorSession,
};

pub(super) struct UiAssetPreviewPaneData {
    pub(super) summary: String,
    pub(super) items: Vec<String>,
    pub(super) canvas_items: Vec<UiAssetEditorPreviewCanvasNode>,
    pub(super) selected_index: i32,
    pub(super) surface_width: f32,
    pub(super) surface_height: f32,
    pub(super) palette_drag_target_preview_index: i32,
    pub(super) palette_drag_target_action: String,
    pub(super) palette_drag_target_label: String,
    pub(super) palette_drag_slot_target_items: Vec<UiAssetEditorPreviewCanvasSlotTarget>,
    pub(super) palette_drag_candidate_items: Vec<String>,
    pub(super) palette_drag_candidate_selected_index: i32,
    pub(super) palette_target_chooser_active: bool,
}

impl UiAssetEditorSession {
    pub(super) fn preview_pane_presentation(&self) -> UiAssetPreviewPaneData {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "preview",);
        let summary = preview_summary(self.preview_host.as_ref());
        let projection = build_preview_projection(
            &self.last_valid_document,
            self.preview_host.as_ref(),
            &self.selection,
        );
        let selected_palette_drag_target = self.selected_palette_drag_target();
        let palette_drag_slot_target_items = selected_palette_drag_target
            .map(|drag_target| {
                let source_items = build_palette_drag_slot_target_overlays(
                    &self.last_valid_document,
                    drag_target,
                    self.palette_catalog.reference_imports(),
                    &projection,
                );
                let mut output = Vec::with_capacity(source_items.len());
                for item in source_items {
                    output.push(UiAssetEditorPreviewCanvasSlotTarget {
                        label: item.label,
                        detail: item.detail,
                        x: item.x,
                        y: item.y,
                        width: item.width,
                        height: item.height,
                        selected: item.selected,
                    });
                }
                output
            })
            .unwrap_or_default();
        let palette_drag_candidate_items = self
            .palette_target_chooser
            .as_ref()
            .map(|chooser| {
                let candidates = &chooser.resolution().candidates;
                let mut output = Vec::with_capacity(candidates.len());
                for candidate in candidates {
                    output.push(if candidate.detail.is_empty() {
                        candidate.key.clone()
                    } else {
                        format!("{} • {}", candidate.key, candidate.detail)
                    });
                }
                output
            })
            .unwrap_or_default();
        record_current_ui_perf_counter(UiPerfCounter::AssetEditorPanePreviewBuildCount, 1.0);
        let UiAssetPreviewProjection {
            items,
            canvas_nodes,
            selected_index,
            surface_width,
            surface_height,
        } = projection;
        UiAssetPreviewPaneData {
            summary,
            items,
            canvas_items: {
                let mut output = Vec::with_capacity(canvas_nodes.len());
                for item in canvas_nodes {
                    output.push(UiAssetEditorPreviewCanvasNode {
                        node_id: item.node_id,
                        label: item.label,
                        kind: item.kind,
                        x: item.x,
                        y: item.y,
                        width: item.width,
                        height: item.height,
                        depth: item.depth,
                        z_index: item.z_index,
                        selected: item.selected,
                    });
                }
                output
            },
            selected_index,
            surface_width,
            surface_height,
            palette_drag_target_preview_index: selected_palette_drag_target
                .and_then(|target| target.preview_index.map(|index| index as i32))
                .unwrap_or(-1),
            palette_drag_target_action: selected_palette_drag_target
                .map(|target| palette_insert_mode_action(target.plan.mode).to_string())
                .unwrap_or_default(),
            palette_drag_target_label: selected_palette_drag_target
                .map(|target| target.plan.label.clone())
                .unwrap_or_default(),
            palette_drag_slot_target_items,
            palette_drag_candidate_items,
            palette_drag_candidate_selected_index: self
                .palette_target_chooser
                .as_ref()
                .map(|chooser| chooser.resolution().selected_index as i32)
                .unwrap_or(-1),
            palette_target_chooser_active: self
                .palette_target_chooser
                .as_ref()
                .map(UiAssetPaletteTargetChooser::sticky)
                .unwrap_or(false),
        }
    }
}

fn palette_insert_mode_action(mode: PaletteInsertMode) -> &'static str {
    match mode {
        PaletteInsertMode::Child => "palette.insert.child",
        PaletteInsertMode::After => "palette.insert.after",
    }
}

fn preview_summary(preview_host: Option<&UiAssetPreviewHost>) -> String {
    let Some(preview_host) = preview_host else {
        return "preview unavailable".to_string();
    };
    format!(
        "{} rendered nodes @ {:.0}x{:.0}",
        preview_host.surface().render_extract.list.commands.len(),
        preview_host.preview_size().width,
        preview_host.preview_size().height
    )
}

#[cfg(test)]
mod optimization_batch_20260830by_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const ITEMS_PER_SAMPLE: usize = 256;

    #[test]
    fn preview_projection_reserves_all_output_collection_lengths() {
        let source = include_str!("preview.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(source_items.len())"));
        assert!(implementation.contains("Vec::with_capacity(candidates.len())"));
        assert!(implementation.contains("Vec::with_capacity(canvas_nodes.len())"));
        assert!(implementation.contains("for item in source_items"));
        assert!(implementation.contains("for candidate in candidates"));
        assert!(implementation.contains("for item in canvas_nodes"));
    }

    #[test]
    fn preview_projection_keeps_slot_candidates_before_canvas_mapping() {
        let source = include_str!("preview.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let slot = implementation
            .find("for item in source_items")
            .expect("slot loop");
        let candidate = implementation
            .find("for candidate in candidates")
            .expect("candidate loop");
        let canvas = implementation
            .find("for item in canvas_nodes")
            .expect("canvas loop");
        assert!(slot < candidate);
        assert!(candidate < canvas);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830by_editor_preview_projection_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR323_PREVIEW_PROJECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items_per_sample={ITEMS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut slots = if optimized {
                Vec::with_capacity(ITEMS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            let mut candidates = if optimized {
                Vec::with_capacity(ITEMS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            let mut canvas = if optimized {
                Vec::with_capacity(ITEMS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..ITEMS_PER_SAMPLE {
                slots.push(index);
                candidates.push(index);
                canvas.push(index);
            }
            checksum ^= slots.len() ^ candidates.len() ^ canvas.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
