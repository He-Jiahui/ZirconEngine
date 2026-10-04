use zircon_runtime_interface::ui::dispatch::{UiAccessibilityInputEvent, UiInputDispatchResult};

use crate::ui::dispatch::UiTextDocumentSession;

use super::super::surface::UiSurface;
use super::{route_policy::annotate_route_policy, route_steps::annotate_result_route_steps};

/// 将辅助技术命令交给无障碍 owner 执行，再补齐统一输入路由的诊断视图。
/// 文本命令可传入活动文档会话；不能把此结果当成指针/键盘事件再次默认处理。
pub(super) fn dispatch_accessibility_input(
    surface: &mut UiSurface,
    accessibility: UiAccessibilityInputEvent,
    text_documents: Option<&mut UiTextDocumentSession>,
) -> UiInputDispatchResult {
    let result = crate::ui::accessibility::dispatch_accessibility_action(
        surface,
        accessibility,
        text_documents,
    );
    with_accessibility_route_policy(surface, result)
}

fn with_accessibility_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}
