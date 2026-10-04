//! 将保存的创建草稿和当前可选模板/引擎投影为跨端数据。
//! 失效选项的后备只服务表单初值，不代替创建动作的准入校验。

use serde::Serialize;

use crate::projects::project_template_catalog;
use crate::state::HubSnapshot;

use super::display::path_text;

/// 创建草稿的跨端投影；模板和引擎选项会随当前注册表修正，原始草稿由会话持有。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubNewProjectDraft {
    pub name: String,
    pub location: String,
    pub template: String,
    pub engine_id: Option<String>,
}

// TODO: [CR-HUBSTATE-0006] 确认持久化草稿的前端恢复意图；当前创建对话框只收默认目录/活动引擎，需检查重开及失败后字段恢复链路。
/// 为跨端状态提供可选表单初值；本函数不保存草稿或验证项目能否创建。
pub(super) fn new_project_draft(snapshot: &HubSnapshot) -> HubNewProjectDraft {
    HubNewProjectDraft {
        name: snapshot.new_project_name.clone(),
        location: path_text(&snapshot.new_project_location, snapshot.settings.language),
        template: selected_template_id(snapshot),
        engine_id: selected_engine_id(snapshot),
    }
}

// 历史配置可能引用已移除或禁用模板；表单初值只选当前可用条目。
fn selected_template_id(snapshot: &HubSnapshot) -> String {
    let template_id = snapshot.selected_template_id.trim();
    if project_template_catalog()
        .iter()
        .any(|template| template.id == template_id && template.enabled)
    {
        return template_id.to_string();
    }

    project_template_catalog()
        .iter()
        .find(|template| template.enabled)
        .map(|template| template.id.to_string())
        .unwrap_or_else(|| "renderable-empty".to_string())
}

// 优先保留有效草稿选择，再采用活动/首个已注册引擎；执行时仍须校验检出和产物。
fn selected_engine_id(snapshot: &HubSnapshot) -> Option<String> {
    snapshot
        .new_project_engine_id
        .as_deref()
        .filter(|id| snapshot.engines.iter().any(|engine| engine.id == *id))
        .map(str::to_string)
        .or_else(|| {
            snapshot
                .active_engine_id
                .as_deref()
                .filter(|id| snapshot.engines.iter().any(|engine| engine.id == *id))
                .map(str::to_string)
        })
        .or_else(|| snapshot.engines.first().map(|engine| engine.id.clone()))
}

#[cfg(test)]
#[path = "tests/new_project.rs"]
mod tests;
