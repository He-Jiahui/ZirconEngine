//! 命令面板显示内容的 fallback 边界；查询保持上游原始值，trim 只用于判定是否显示占位文字。

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

// TODO: [CR-EDITOR-PAINT-OVERLAY-0002] 空态和搜索占位文字目前由本地英文常量生成；
// 确认它们应走哪条宿主本地化投影，避免上游已切换语言而 painter 仍显示英文。
const EMPTY_MESSAGE: &str = "No commands found";
const SEARCH_ICON: &str = "search";
const SEARCH_PLACEHOLDER: &str = "Search commands";

pub(super) struct CommandPaletteSearchText<'a> {
    pub(super) value: &'a str,
    pub(super) placeholder: bool,
}

pub(super) fn command_palette_text_style() -> UiTextRunPaintStyle {
    UiTextRunPaintStyle::default()
}

pub(super) fn command_palette_empty_message() -> &'static str {
    EMPTY_MESSAGE
}

pub(super) fn command_palette_search_icon() -> &'static str {
    SEARCH_ICON
}

pub(super) fn command_palette_search_text(query: &str) -> CommandPaletteSearchText<'_> {
    if query.trim().is_empty() {
        CommandPaletteSearchText {
            value: SEARCH_PLACEHOLDER,
            placeholder: true,
        }
    } else {
        CommandPaletteSearchText {
            value: query,
            placeholder: false,
        }
    }
}

#[cfg(test)]
#[path = "tests/text.rs"]
mod tests;
