use serde_json::Value;

use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, PaneTemplateSpec, ViewDescriptorId, ViewHost, ViewInstanceId,
    ViewKind,
};

use super::ViewContentKind;

#[derive(Clone, Debug)]
/// 实例状态与descriptor声明的联接结果；placeholder标记缺失引用，content_kind只指定UI策略。
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
