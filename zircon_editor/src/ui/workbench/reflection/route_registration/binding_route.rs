//! 反射刷新会重复登记同一动作；完整绑定匹配让稳定路由可复用。
use crate::ui::binding::EditorUiBinding;
use crate::ui::control::EditorUiControlService;
use zircon_runtime_interface::ui::event_ui::UiRouteId;

/// 先按完整命令绑定查重，再为新绑定分配路由；同控件的不同命令仍独立。
pub(crate) fn register_binding_route(
    service: &mut EditorUiControlService,
    binding: EditorUiBinding,
) -> UiRouteId {
    service
        .route_id_for_binding(&binding.as_ui_binding())
        .unwrap_or_else(|| service.register_binding_route(binding.as_ui_binding()))
}
