//! 按钮的内置图标词汇。生产内容从节点各身份字段选取图标；单字符串入口目前由短键回归测试消费。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 内容布局使用的有限图标语义；None 是合法的纯文字按钮。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum ButtonGlyph {
    None,
    Plus,
    Trash,
    ChevronDown,
}

/// 单个、小写身份键的旧语义识别入口；目前只有回归测试消费，生产节点选图标由内容域统一处理。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_glyph_for_key(
    key: &str,
) -> ButtonGlyph {
    if key.len() < 3 {
        return ButtonGlyph::None;
    }
    if key.contains("delete") || key.contains("trash") || key.contains("danger") {
        ButtonGlyph::Trash
    } else if key.contains("dropdown") || key.contains("drop-down") || key.contains("menu") {
        ButtonGlyph::ChevronDown
    } else if key.contains("icon") || key.contains("add") || key.contains("plus") {
        ButtonGlyph::Plus
    } else {
        ButtonGlyph::None
    }
}

#[cfg(test)]
#[path = "identity/tests/short_key_tests.rs"]
mod short_key_tests;
