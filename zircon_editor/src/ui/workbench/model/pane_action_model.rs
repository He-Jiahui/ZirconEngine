use crate::ui::binding::EditorUiBinding;

#[derive(Clone, Debug, PartialEq)]
/// 空状态恢复动作；绑定决定实际命令，label和prominent仅表达呈现优先级。
pub struct PaneActionModel {
    pub label: String,
    pub binding: Option<EditorUiBinding>,
    pub prominent: bool,
}
