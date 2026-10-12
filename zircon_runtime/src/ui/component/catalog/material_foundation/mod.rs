//! Material 编辑器目录的统一注册入口：模板和编辑器保留宿主共享同一份只读元数据，项目扩展可取得独立副本。

use std::sync::OnceLock;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;

#[path = "button_inputs.rs"]
mod button_inputs;
#[path = "data_display.rs"]
mod data_display;
#[path = "../../../catalog/material_foundation/data_display_editor.rs"]
mod data_display_editor;
#[path = "../../../catalog/material_foundation/data_display_subcomponents.rs"]
mod data_display_subcomponents;
#[path = "../../../catalog/material_foundation/data_display_table.rs"]
mod data_display_table;
#[path = "../../../catalog/material_foundation/data_display_visuals.rs"]
mod data_display_visuals;
#[path = "feedback.rs"]
mod feedback;
#[path = "../../../catalog/material_foundation/feedback_editor_overlays.rs"]
mod feedback_editor_overlays;
#[path = "form_controls.rs"]
mod form_controls;
#[path = "inputs.rs"]
mod inputs;
#[path = "lab_subcomponents.rs"]
mod lab_subcomponents;
#[path = "layout.rs"]
mod layout;
#[path = "layout_editor.rs"]
mod layout_editor;
#[path = "layout_mui.rs"]
mod layout_mui;
#[path = "../../../catalog/material_foundation/layout_transitions.rs"]
mod layout_transitions;
#[path = "layout_utilities.rs"]
mod layout_utilities;
#[path = "mui_x.rs"]
mod mui_x;
#[path = "navigation.rs"]
mod navigation;
#[path = "navigation_editor.rs"]
mod navigation_editor;
#[path = "../../../catalog/material_foundation/navigation_secondary.rs"]
mod navigation_secondary;
#[path = "../../../catalog/material_foundation/navigation_subcomponents.rs"]
mod navigation_subcomponents;
#[path = "selection_inputs.rs"]
mod selection_inputs;
#[path = "shared.rs"]
mod shared;
#[path = "../../../catalog/material_foundation/surface_subcomponents.rs"]
mod surface_subcomponents;
#[path = "surfaces.rs"]
mod surfaces;
#[path = "text_inputs.rs"]
mod text_inputs;

#[cfg(test)]
#[path = "tests/capacity.rs"]
mod capacity_tests;

static MATERIAL_EDITOR_FOUNDATION_REGISTRY: OnceLock<UiComponentDescriptorRegistry> =
    OnceLock::new();

const MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT: usize = 25;

impl UiComponentDescriptorRegistry {
    /// Builds the component catalog for the Material Dark editor foundation.
    /// 需要增加或替换项目描述符时使用独立副本；编译与只读查询可直接借用共享目录。
    pub fn material_editor_foundation() -> Self {
        Self::material_editor_foundation_shared().clone()
    }

    /// Returns the process-wide read-only Material editor foundation catalog.
    pub fn material_editor_foundation_shared() -> &'static Self {
        MATERIAL_EDITOR_FOUNDATION_REGISTRY.get_or_init(build_material_editor_foundation_registry)
    }
}

// 内建描述符必须在构建目录时通过统一校验；失败表示内建声明违反契约，无法交付半成品目录。
fn build_material_editor_foundation_registry() -> UiComponentDescriptorRegistry {
    let mut registry = UiComponentDescriptorRegistry::new();
    for descriptor in material_editor_foundation_descriptors() {
        registry
            .register(descriptor)
            .expect("Material editor foundation descriptors must validate");
    }
    registry
}

fn material_editor_foundation_descriptors() -> Vec<UiComponentDescriptor> {
    let descriptor_groups: [Vec<UiComponentDescriptor>;
        MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT] = [
        button_inputs::descriptors(),
        inputs::descriptors(),
        selection_inputs::descriptors(),
        text_inputs::descriptors(),
        form_controls::descriptors(),
        data_display::descriptors(),
        data_display_editor::descriptors(),
        data_display_subcomponents::descriptors(),
        data_display_table::descriptors(),
        data_display_visuals::descriptors(),
        feedback::descriptors(),
        feedback_editor_overlays::descriptors(),
        surface_subcomponents::descriptors(),
        surfaces::descriptors(),
        navigation::descriptors(),
        navigation_subcomponents::descriptors(),
        navigation_secondary::descriptors(),
        navigation_editor::descriptors(),
        layout_mui::descriptors(),
        layout::descriptors(),
        layout_utilities::descriptors(),
        layout_transitions::descriptors(),
        layout_editor::descriptors(),
        mui_x::descriptors(),
        lab_subcomponents::descriptors(),
    ];
    let descriptor_capacity = descriptor_groups.iter().fold(0usize, |capacity, group| {
        capacity.saturating_add(group.len())
    });
    let mut descriptors = Vec::with_capacity(descriptor_capacity);
    for group in descriptor_groups {
        descriptors.extend(group);
    }
    descriptors
}
