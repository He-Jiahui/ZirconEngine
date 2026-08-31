use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::component::{UiComponentProjectionPatch, UiValue};

use crate::core::commands::DocumentKind;
use crate::core::editor_operation::{EditorOperationPath, EditorOperationPathError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewDescriptor {
    id: String,
    display_name: String,
    category: String,
    document_kind: Option<DocumentKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ui_template_id: Option<String>,
}

impl ViewDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            category: category.into(),
            document_kind: None,
            ui_template_id: None,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn with_document_kind(mut self, document_kind: DocumentKind) -> Self {
        self.document_kind = Some(document_kind);
        self
    }

    pub fn document_kind(&self) -> Option<&DocumentKind> {
        self.document_kind.as_ref()
    }

    pub fn with_ui_template_id(mut self, template_id: impl Into<String>) -> Self {
        self.ui_template_id = Some(template_id.into());
        self
    }

    pub fn ui_template_id(&self) -> Option<&str> {
        self.ui_template_id.as_deref()
    }

    pub(crate) fn bind_ui_template_id(&mut self, template_id: impl Into<String>) {
        self.ui_template_id = Some(template_id.into());
    }

    pub fn open_operation_path(&self) -> Result<EditorOperationPath, EditorOperationPathError> {
        EditorOperationPath::parse(build_view_open_operation_path(&self.id))
    }
}

fn build_view_open_operation_path(view_id: &str) -> String {
    const PREFIX: &str = "view.";
    const SUFFIX: &str = ".open";

    let mut path = String::with_capacity(PREFIX.len() + view_id.len() + SUFFIX.len());
    path.push_str(PREFIX);
    path.push_str(view_id);
    path.push_str(SUFFIX);
    path
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditorUiTemplatePaneDataSnapshot {
    values: BTreeMap<String, UiValue>,
    component_patches: Vec<UiComponentProjectionPatch>,
}

impl EditorUiTemplatePaneDataSnapshot {
    pub fn new(values: BTreeMap<String, UiValue>) -> Self {
        Self {
            values,
            component_patches: Vec::new(),
        }
    }

    pub fn values(&self) -> &BTreeMap<String, UiValue> {
        &self.values
    }

    pub fn with_component_patch(mut self, patch: UiComponentProjectionPatch) -> Self {
        self.component_patches.push(patch);
        self
    }

    pub fn component_patches(&self) -> &[UiComponentProjectionPatch] {
        &self.component_patches
    }
}

pub trait EditorUiTemplatePaneDataSource: Send + Sync {
    fn snapshot(&self) -> EditorUiTemplatePaneDataSnapshot;
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const PATHS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fc_editor391_open_operation_path_uses_view_identity() {
        let descriptor = ViewDescriptor::new("asset.browser", "Assets", "Content");
        assert_eq!(
            descriptor.open_operation_path().unwrap().as_str(),
            "view.asset.browser.open"
        );

        for id in ["scene", "runtime.diagnostics", "plugin.alpha.custom_view"] {
            assert_eq!(
                build_view_open_operation_path(id),
                format!("view.{id}.open")
            );
        }

        let production = include_str!("view_descriptor.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("format!("));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fc_editor391_direct_view_operation_path_benchmark() {
        const VIEW_ID: &str = "plugin.world_partition.runtime_diagnostics";
        for _ in 0..4 {
            black_box(measure_paths(|id| format!("view.{id}.open"), VIEW_ID));
            black_box(measure_paths(build_view_open_operation_path, VIEW_ID));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_paths(|id| format!("view.{id}.open"), VIEW_ID));
                optimized_samples.push(measure_paths(build_view_open_operation_path, VIEW_ID));
            } else {
                optimized_samples.push(measure_paths(build_view_open_operation_path, VIEW_ID));
                legacy_samples.push(measure_paths(|id| format!("view.{id}.open"), VIEW_ID));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_paths(mut build: impl FnMut(&str) -> String, id: &str) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..PATHS_PER_SAMPLE {
            let path = black_box(build(black_box(id)));
            checksum = checksum.wrapping_add(path.len());
            black_box(path);
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
            "EDITOR391_DIRECT_VIEW_OPERATION_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} paths_per_sample={PATHS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(70) / 100,
            "direct view operation path construction must reduce P95 by at least 30%"
        );
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
}
