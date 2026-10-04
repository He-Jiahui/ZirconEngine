//! 动作颜色的只读快照；取消、普通动作、确认/破坏性确认各有宿主语义角色。
//! 不可用或确认禁用优先于破坏性配色，防止被禁用的动作仍显示为可执行。

use super::super::palette::dialog_palette;
use super::super::variants::variant_contains_any;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct DialogActionPaint {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_action_paint(
    unavailable: bool,
) -> DialogActionPaint {
    let palette = dialog_palette();
    if unavailable {
        DialogActionPaint {
            surface: palette.disabled_surface,
            border: palette.disabled_border,
            text: palette.disabled_text,
        }
    } else {
        DialogActionPaint {
            surface: palette.surface,
            border: palette.border,
            text: palette.action,
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn cancel_action_paint(
    unavailable: bool,
) -> DialogActionPaint {
    let palette = dialog_palette();
    if unavailable {
        DialogActionPaint {
            surface: palette.disabled_surface,
            border: palette.disabled_border,
            text: palette.disabled_text,
        }
    } else {
        DialogActionPaint {
            surface: palette.surface,
            border: palette.border,
            text: palette.body,
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn confirm_action_paint(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    confirm_enabled: bool,
) -> DialogActionPaint {
    let palette = dialog_palette();
    if unavailable || !confirm_enabled {
        return DialogActionPaint {
            surface: palette.disabled_surface,
            border: palette.disabled_border,
            text: palette.disabled_text,
        };
    }
    if variant_contains_any(node, &["destructive"]) {
        return DialogActionPaint {
            surface: palette.error_border,
            border: palette.error,
            text: palette.title,
        };
    }
    DialogActionPaint {
        surface: palette.action,
        border: palette.action,
        text: palette.title,
    }
}
