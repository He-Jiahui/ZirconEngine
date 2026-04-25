use serde_json::Value;

use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstanceId, ViewKind};

use super::{ViewContentKind, ViewTabSnapshot};

pub(super) fn placeholder_view(
    instance_id: ViewInstanceId,
    descriptor_id: ViewDescriptorId,
    title: String,
) -> ViewTabSnapshot {
    ViewTabSnapshot {
        instance_id,
        descriptor_id,
        title,
        icon_key: "missing".to_string(),
        kind: ViewKind::ActivityView,
        host: ViewHost::Document(MainPageId::workbench(), vec![]),
        serializable_payload: Value::Null,
        dirty: false,
        content_kind: ViewContentKind::Placeholder,
        pane_template: None,
        placeholder: true,
    }
}
