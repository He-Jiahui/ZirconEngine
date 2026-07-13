use crate::ui::retained_host::primitives::{ModelRc, SharedString};

#[derive(Clone, Default)]
pub(crate) struct TemplatePaneWeightHeatmapSourceData {
    pub x: f32,
    pub y: f32,
    pub weight: f32,
    pub selected: bool,
}

#[derive(Clone)]
pub(crate) struct TemplatePaneWeightHeatmapData {
    pub columns: i32,
    pub rows: i32,
    pub low_label: SharedString,
    pub high_label: SharedString,
    pub sources: ModelRc<TemplatePaneWeightHeatmapSourceData>,
}

impl Default for TemplatePaneWeightHeatmapData {
    fn default() -> Self {
        Self {
            columns: 12,
            rows: 8,
            low_label: "0.0".into(),
            high_label: "1.0".into(),
            sources: ModelRc::default(),
        }
    }
}
