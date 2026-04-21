use crate::ui::binding::EditorUiBinding;

#[derive(Clone, Debug, PartialEq)]
pub struct PaneActionModel {
    pub label: String,
    pub binding: Option<EditorUiBinding>,
    pub prominent: bool,
}
