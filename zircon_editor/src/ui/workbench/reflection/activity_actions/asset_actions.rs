//! 资产标签仅公布导入路径草稿和模型导入动作；占位标签由上层解析器过滤。
use zircon_runtime_interface::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) const ASSET_ACTION_COUNT: usize = 2;

/// 构建资产标签公开动作；注册阶段为这些描述补齐有类型的调用路由。
pub(super) fn asset_actions() -> [UiActionDescriptor; ASSET_ACTION_COUNT] {
    [
        UiActionDescriptor::new(
            "workbench.asset.mesh_import.path.set",
            UiEventKind::Change,
            "DraftCommand.SetMeshImportPath",
        )
        .with_parameter(UiParameterDescriptor::new("value", UiValueType::String)),
        UiActionDescriptor::new(
            "workbench.asset.model.import",
            UiEventKind::Click,
            "AssetCommand.ImportModel",
        ),
    ]
}
