//! 菜单尾部标记优先级为子菜单、checked、显式图标，再退回少量英文菜单标签的默认图标。
//! 选项行则用selected/special决定是否出现勾选；这些只是视觉反馈，激活仍归交互owner。

use super::super::super::data::TemplatePaneMenuItemData;
use super::flags::{menu_item_flag_value, menu_item_has_flag};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum PopupRowAdornmentKind<'a>
{
    Check,
    Chevron,
    Icon(&'a str),
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn option_adornment_kind(
    selected: bool,
) -> Option<PopupRowAdornmentKind<'static>> {
    selected.then_some(PopupRowAdornmentKind::Check)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_row_adornment_kind<
    'a,
>(
    item: &'a TemplatePaneMenuItemData,
) -> Option<PopupRowAdornmentKind<'a>> {
    if menu_item_has_flag(item, "submenu") {
        return Some(PopupRowAdornmentKind::Chevron);
    }
    if item.checked {
        return Some(PopupRowAdornmentKind::Check);
    }
    if let Some(icon) = menu_item_flag_value(item, "icon") {
        return Some(PopupRowAdornmentKind::Icon(icon));
    }
    menu_item_default_adornment(item.label.as_str())
}

// TODO: [CR-EDITOR-PAINT-OVERLAY-0011] 最后一级默认图标由英文显示标签推断；
// 确认产品菜单是否保证上游总给稳定icon字段，否则本地化或重命名后默认图标会消失。
fn menu_item_default_adornment(label: &str) -> Option<PopupRowAdornmentKind<'static>> {
    let label = label.trim();
    if label.eq_ignore_ascii_case("new") {
        Some(PopupRowAdornmentKind::Icon("add"))
    } else if label.eq_ignore_ascii_case("open") {
        Some(PopupRowAdornmentKind::Icon("folder"))
    } else if label.eq_ignore_ascii_case("save") {
        Some(PopupRowAdornmentKind::Icon("save"))
    } else if label.eq_ignore_ascii_case("delete") {
        Some(PopupRowAdornmentKind::Icon("trash"))
    } else if label.eq_ignore_ascii_case("more tools") {
        Some(PopupRowAdornmentKind::Chevron)
    } else {
        None
    }
}
