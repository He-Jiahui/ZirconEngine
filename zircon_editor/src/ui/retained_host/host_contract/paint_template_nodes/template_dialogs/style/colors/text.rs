//! 标题和正文的语义颜色；破坏性或error标题可显示严重性色，正文维持易读的次级角色。

use super::super::super::identity::DialogKind;
use super::super::palette::dialog_palette;
use super::super::severity::{severity, severity_mark_color, DialogSeverity};
use super::super::variants::variant_contains_any;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_title_color(
    node: &TemplatePaneNodeData,
    kind: DialogKind,
    unavailable: bool,
) -> [u8; 4] {
    let palette = dialog_palette();
    if unavailable {
        palette.disabled_text
    } else if kind.uses_severity_chrome()
        && (variant_contains_any(node, &["destructive"])
            || matches!(severity(node), DialogSeverity::Error))
    {
        severity_mark_color(node)
    } else {
        palette.title
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_body_color(
    unavailable: bool,
) -> [u8; 4] {
    let palette = dialog_palette();
    if unavailable {
        palette.disabled_text
    } else {
        palette.body
    }
}
