//! 弹出菜单、选项列表与命令面板通过这一行状态快照共享视觉配方；focused 代表调用端已选择的视觉焦点。
//! 快照没有宿主节点完整样式或焦点模态，调用端须先投影所需布尔状态。

use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiPainterState};

/// 由菜单、选项或命令面板调用端投影的弹出行状态快照；focused 为该行视觉焦点。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchPopupRowState
{
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
    pub checked: bool,
    pub selected: bool,
    pub open: bool,
    pub dragging: bool,
    pub drop_hovered: bool,
    pub loading: bool,
    pub danger: bool,
}

impl WorkbenchPopupRowState {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn painter_state(
        self,
    ) -> UiPainterState {
        UiPainterState {
            hovered: self.hovered,
            pressed: self.pressed,
            focused: self.focused,
            focus_visible: self.focused,
            disabled: self.disabled,
            checked: self.checked,
            selected: self.selected,
            open: self.open,
            dragging: self.dragging,
            drop_hovered: self.drop_hovered,
            loading: self.loading,
            ..UiPainterState::normal()
        }
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn marked(self) -> bool {
        self.checked || self.selected
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn hot(self) -> bool {
        self.hovered || self.pressed || self.open || self.dragging || self.drop_hovered
    }
}

/// 供弹出行表面、正文、快捷键和装饰分别消费的配方。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchPopupRowStyle
{
    pub background: Option<[u8; 4]>,
    pub outline: Option<[u8; 4]>,
    pub text: [u8; 4],
    pub shortcut: [u8; 4],
    pub adornment: [u8; 4],
    pub state: UiPainterResolvedState,
}
