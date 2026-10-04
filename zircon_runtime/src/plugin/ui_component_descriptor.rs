use serde::{Deserialize, Serialize};

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor as RuntimeUiComponentDescriptor,
    UiDefaultNodeTemplate, UiHostCapability, UiPaletteMetadata, UiPropSchema, UiSlotSchema,
    UiValue, UiValueKind,
};

/// 插件清单中的 UI 组件声明；登记时只校验身份与 .zui 引用，安装到 UI 目录时再投影为宿主组件描述符。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentDescriptor {
    pub component_id: String,
    pub plugin_id: String,
    pub ui_document: String,
}

impl UiComponentDescriptor {
    pub fn new(
        component_id: impl Into<String>,
        plugin_id: impl Into<String>,
        ui_document: impl Into<String>,
    ) -> Self {
        Self {
            component_id: component_id.into(),
            plugin_id: plugin_id.into(),
            ui_document: ui_document.into(),
        }
    }

    /// 保留插件身份和文档 URI 为必填 prop，并为编辑器 palette/默认节点建立可选择入口。
    // TODO: [CR-PLUGIN-BOUNDARY-0003] Editor 与 Runtime 在宿主集合中按 AND 解释；runtime_basic 不含 Editor，需确认插件组件在独立 Runtime 中不可选是否符合预期。
    pub fn to_runtime_component_descriptor(&self) -> RuntimeUiComponentDescriptor {
        RuntimeUiComponentDescriptor::new(
            self.component_id.clone(),
            self.display_name(),
            UiComponentCategory::Container,
            "plugin-ui-component",
        )
        .default_prop("plugin_id", UiValue::String(self.plugin_id.clone()))
        .default_prop("ui_document", UiValue::String(self.ui_document.clone()))
        .with_prop(UiPropSchema::new("plugin_id", UiValueKind::String).required(true))
        .with_prop(UiPropSchema::new("ui_document", UiValueKind::String).required(true))
        .slot(UiSlotSchema::new("content").multiple(true))
        .requires_host_capability(UiHostCapability::Editor)
        .requires_host_capability(UiHostCapability::Runtime)
        .default_node_template(UiDefaultNodeTemplate::native(self.component_id.as_str()))
        .palette(UiPaletteMetadata::new(
            self.display_name(),
            UiComponentCategory::Container,
            self.component_id.clone(),
            UiDefaultNodeTemplate::native(self.component_id.as_str()),
        ))
    }

    // 编辑器名称只取组件 ID 的末段；完整 ID 仍用于注册冲突检测和节点模板身份。
    fn display_name(&self) -> String {
        self.component_id
            .rsplit('.')
            .next()
            .unwrap_or(&self.component_id)
            .to_string()
    }
}
