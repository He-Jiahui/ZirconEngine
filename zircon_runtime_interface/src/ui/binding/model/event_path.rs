use serde::{Deserialize, Serialize};

use super::UiEventKind;

/// UI 事件分发所用的视图、控件与事件种类三元路由地址。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiEventPath {
    pub view_id: String,
    pub control_id: String,
    pub event_kind: UiEventKind,
}

impl UiEventPath {
    pub fn new(
        view_id: impl Into<String>,
        control_id: impl Into<String>,
        event_kind: UiEventKind,
    ) -> Self {
        Self {
            view_id: view_id.into(),
            control_id: control_id.into(),
            event_kind,
        }
    }

    pub fn native_prefix(&self) -> String {
        let mut output = String::with_capacity(
            self.view_id.len() + self.control_id.len() + self.event_kind.native_name().len() + 2,
        );
        self.native_prefix_into(&mut output);
        output
    }

    // TODO: [CR-R02-public_ui_binding-0001] 此处直接拼接 view_id/control_id，parser.rs 按首个冒号和斜线切分；需确认字段是否保证不含分隔符，或定义可逆转义。
    pub(crate) fn native_prefix_into(&self, output: &mut String) {
        output.push_str(&self.view_id);
        output.push('/');
        output.push_str(&self.control_id);
        output.push(':');
        output.push_str(self.event_kind.native_name());
    }
}
