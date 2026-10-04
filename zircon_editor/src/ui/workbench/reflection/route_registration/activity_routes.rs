//! 路由注册读取活动身份并改写其动作列表；列表短暂移出以保持同一活动快照。
use crate::ui::control::EditorUiControlService;
use crate::ui::EditorActivityReflection;

use super::action_route::register_action_route;

/// 在不复制活动属性快照的前提下补路由；返回前恢复原动作顺序。
pub(super) fn register_activity_routes(
    service: &mut EditorUiControlService,
    activity: &mut EditorActivityReflection,
) {
    let mut actions = std::mem::take(&mut activity.actions);
    for action in &mut actions {
        register_action_route(service, activity, action);
    }
    activity.actions = actions;
}

#[cfg(test)]
#[path = "tests/activity_routes_performance_tests.rs"]
mod performance_tests;
