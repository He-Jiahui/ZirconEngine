//! 反射动作接到检查器批量命令，空变更集只是注册时的默认负载。
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::control::EditorUiControlService;
use crate::ui::EditorActivityReflection;
use zircon_runtime_interface::ui::{
    binding::{UiEventKind, UiEventPath},
    event_ui::UiRouteId,
};

use super::name_mapping::binding_view_id;
use super::route_registration::register_binding_route;

/// 登记批量提交的有类型路由；实际subject和changes须由事件载荷提供。
pub(super) fn register_inspector_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    event_kind: UiEventKind,
) -> Option<UiRouteId> {
    let path = UiEventPath::new(binding_view_id(activity), "ApplyBatchButton", event_kind);
    let default_subject = "entity://selected".to_string();
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        EditorUiBindingPayload::inspector_field_batch(default_subject, Vec::new()),
    );
    Some(register_binding_route(service, registration_binding))
}
