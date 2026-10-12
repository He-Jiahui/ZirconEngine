//! 未由 paint_template_nodes/mod.rs 接入的遗留图标类别门面；当前生产图标按钮直接消费实例 icon_name。

// TODO: [CR-EDITOR-PAINT-FORMS-0001] 确认该未接入门面是否应随旧手工图标映射一并移除；重新接入前必须恢复缺失的 kind/mapping 子模块。
mod kind;
mod mapping;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use kind::IconButtonGlyphKind;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use mapping::icon_button_glyph_kind;
