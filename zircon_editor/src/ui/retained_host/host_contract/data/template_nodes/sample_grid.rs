use crate::ui::retained_host::primitives::{ModelRc, SharedString};

#[derive(Clone, Default)]
pub(crate) struct TemplatePaneSampleGridPointData {
    pub x: f32,
    pub y: f32,
    pub label: SharedString,
    pub selected: bool,
}

#[derive(Clone)]
pub(crate) struct TemplatePaneSampleGridData {
    pub x_axis_label: SharedString,
    pub y_axis_label: SharedString,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub x_ticks: ModelRc<f32>,
    pub y_ticks: ModelRc<f32>,
    pub points: ModelRc<TemplatePaneSampleGridPointData>,
}

impl Default for TemplatePaneSampleGridData {
    fn default() -> Self {
        Self {
            x_axis_label: SharedString::default(),
            y_axis_label: SharedString::default(),
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
            x_ticks: ModelRc::default(),
            y_ticks: ModelRc::default(),
            points: ModelRc::default(),
        }
    }
}
