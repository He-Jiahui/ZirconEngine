//! 布局阶段借用节点文字，命令生成时才复制为拥有所有权的载荷；同一修剪与回退协议用于测量和绘制。

mod body;
mod entry;
mod title;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_tooltip_text;

pub(super) fn tooltip_title(node: &super::super::super::data::TemplatePaneNodeData) -> &str {
    let text = node.text.as_str().trim();
    if text.is_empty() {
        "Tooltip"
    } else {
        text
    }
}

pub(super) fn tooltip_body(node: &super::super::super::data::TemplatePaneNodeData) -> &str {
    node.label_text.as_str().trim()
}

#[cfg(test)]
#[path = "text/tests/borrowed_text_tests.rs"]
mod borrowed_text_tests;
