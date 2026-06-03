use serde_json::Value;

use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PaneTemplateSpec, ViewDescriptorId, ViewHost, ViewInstanceId,
    ViewKind,
};

use super::ViewContentKind;

#[derive(Clone, Debug)]
pub struct ViewTabSnapshot {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub icon_key: String,
    pub kind: ViewKind,
    pub host: ViewHost,
    pub serializable_payload: Value,
    pub dirty: bool,
    pub content_kind: ViewContentKind,
    pub pane_template: Option<PaneTemplateSpec>,
    pub activity_window_template: Option<ActivityWindowTemplateSpec>,
    pub placeholder: bool,
}
