//! Inspector 专用行识别与布尔展示值汇聚入口；specialized 绘制和事件路由共享控件身份，但绘制子类仍依赖标签约定。

#[path = "inspector_row_kind/bool_values.rs"]
mod bool_values;
#[path = "inspector_row_kind/classifier.rs"]
mod classifier;
#[path = "inspector_row_kind/constants.rs"]
mod constants;
#[path = "inspector_row_kind/kind.rs"]
mod kind;
#[path = "inspector_row_kind/matching.rs"]
mod matching;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use bool_values::{
    bool_display_value, bool_value,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use classifier::inspector_row_kind;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use constants::{
    COMPONENT_PROPERTY_SLOT_03, MATERIAL_PROPERTY_ROW,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use kind::{
    InspectorResourceKind, InspectorRowKind,
};
