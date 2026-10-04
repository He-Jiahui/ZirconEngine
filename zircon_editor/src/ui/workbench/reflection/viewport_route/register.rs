//! 画布动作汇入现有ViewportSurface分派路径，事件类型和命令负载共同决定路由身份。
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::control::EditorUiControlService;
use crate::ui::EditorActivityReflection;
use zircon_runtime_interface::ui::{
    binding::{UiEventKind, UiEventPath},
    event_ui::UiRouteId,
};

use super::super::name_mapping::binding_view_id;
use super::super::route_registration::register_binding_route;
use super::default_command::default_viewport_command;

/// 登记画布事件的有类型绑定；同控件上的动作通过完整负载保持独立。
pub(crate) fn register_viewport_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<UiRouteId> {
    let default_command = default_viewport_command(action_id)?;
    let path = UiEventPath::new(binding_view_id(activity), "ViewportSurface", event_kind);
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        EditorUiBindingPayload::viewport_command(default_command.clone()),
    );
    Some(register_binding_route(service, registration_binding))
}
