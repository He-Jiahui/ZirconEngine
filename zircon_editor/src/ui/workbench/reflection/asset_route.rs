//! 资产动作的反射路由接到既有导入命令入口，实际导入由事件命令链执行。
use crate::ui::binding::{AssetCommand, EditorUiBinding, EditorUiBindingPayload};
use crate::ui::control::EditorUiControlService;
use crate::ui::EditorActivityReflection;
use zircon_runtime_interface::ui::{
    binding::{UiEventKind, UiEventPath},
    event_ui::UiRouteId,
};

use super::name_mapping::binding_view_id;
use super::route_registration::register_binding_route;

/// 为模型导入声明登记绑定，保持反射动作与宿主控件使用同一命令入口。
pub(super) fn register_asset_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<UiRouteId> {
    let (control_id, payload) = match action_id {
        "workbench.asset.model.import" => (
            "ImportModel",
            EditorUiBindingPayload::asset_command(AssetCommand::ImportModel),
        ),
        _ => return None,
    };
    let path = UiEventPath::new(binding_view_id(activity), control_id, event_kind);
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        payload,
    );
    Some(register_binding_route(service, registration_binding))
}
