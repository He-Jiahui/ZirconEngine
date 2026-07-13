use crate::ui::retained_host::primitives::{ModelRc, SharedString};

#[derive(Clone, Default)]
pub(crate) struct TemplatePaneTimelineKeyData {
    pub time: f32,
    pub label: SharedString,
    pub selected: bool,
}

#[derive(Clone)]
pub(crate) struct TemplatePaneTimelineStripData {
    pub duration: f32,
    pub current_time: f32,
    pub tick_interval: f32,
    pub track_label: SharedString,
    pub keys: ModelRc<TemplatePaneTimelineKeyData>,
}

impl Default for TemplatePaneTimelineStripData {
    fn default() -> Self {
        Self {
            duration: 1.0,
            current_time: 0.0,
            tick_interval: 0.25,
            track_label: SharedString::default(),
            keys: ModelRc::default(),
        }
    }
}
