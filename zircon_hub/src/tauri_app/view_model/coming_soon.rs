//! 集中投影尚未接入的能力与原因，供各页按稳定类别展示预留入口。
//! 这些条目是界面说明，不注册动作或启动远程服务。

use std::borrow::Cow;

use serde::Serialize;

use crate::settings::HubLanguage;

use super::localized::HubTextBundle;

/// 预留能力的跨端说明；固定身份与类别供筛选，显示文本供当前语言阅读。
/// 借用的文案均为静态资源，序列化后仍是普通字符串；组合信息保留拥有的字符串。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubComingSoonEntry {
    pub id: Cow<'static, str>,
    pub category: Cow<'static, str>,
    pub category_label: Cow<'static, str>,
    pub title: Cow<'static, str>,
    pub detail: Cow<'static, str>,
    pub status: Cow<'static, str>,
    pub meta: String,
    pub disabled: bool,
}

/// 由视图投影调用，集中声明当前禁用能力及原因；启用工作流时须同时核对真实动作准入。
pub(crate) fn coming_soon_entries(language: HubLanguage) -> Vec<HubComingSoonEntry> {
    let text = HubTextBundle::new(language);
    [
        (
            "project-template-2d-scene",
            "projects",
            text.pair("2D Scene Template", "2D 场景模板"),
            text.pair(
                "The 2D scene template is reserved until the local authoring workflow is ready.",
                "2D 场景模板会在本地创作工作流就绪后开放。",
            ),
        ),
        (
            "project-template-3d-scene",
            "projects",
            text.pair("3D Scene Template", "3D 场景模板"),
            text.pair(
                "The 3D scene template is reserved until the local authoring workflow is ready.",
                "3D 场景模板会在本地创作工作流就绪后开放。",
            ),
        ),
        (
            "project-template-sample-world",
            "projects",
            text.pair("Sample World Template", "示例世界模板"),
            text.pair(
                "The sample world template is reserved for sample content generation.",
                "示例世界模板为示例内容生成预留。",
            ),
        ),
        (
            "asset-import",
            "assets",
            text.pair("Asset Import", "资产导入"),
            text.pair(
                "Import pipelines are reserved for the next local content workflow.",
                "导入管线为下一阶段本地内容工作流预留。",
            ),
        ),
        (
            "plugin-install",
            "plugins",
            text.pair("Plugin Install", "插件安装"),
            text.pair(
                "Installing or downloading plugins is disabled in v1.",
                "v1 暂不支持安装或下载插件。",
            ),
        ),
        (
            "plugin-toggle",
            "plugins",
            text.pair("Plugin Enable/Disable", "插件启停"),
            text.pair(
                "Plugin activation controls will be connected after the local manifest workflow is stable.",
                "插件启停会在本地清单工作流稳定后接入。",
            ),
        ),
        (
            "marketplace-download",
            "plugins",
            text.pair("Marketplace Download", "市场下载"),
            text.pair(
                "Remote marketplace access is outside the local-only v1 scope.",
                "远程市场访问不属于本地 v1 范围。",
            ),
        ),
        (
            "remote-sync",
            "local-delivery",
            text.pair("Remote Sync", "远程同步"),
            text.pair(
                "Cloud synchronization is reserved; packages stay local in v1.",
                "云同步为预留能力；v1 包输出仅保留在本地。",
            ),
        ),
        (
            "account-service",
            "local-delivery",
            text.pair("Account Service", "账号服务"),
            text.pair(
                "No remote account or identity service is required for v1.",
                "v1 不需要远程账号或身份服务。",
            ),
        ),
        (
            "cloud-repository",
            "local-delivery",
            text.pair("Cloud Repository", "云仓库"),
            text.pair(
                "Remote package repositories are disabled until the cloud service layer exists.",
                "云服务层完成前禁用远程包仓库。",
            ),
        ),
        (
            "notification-center",
            "shell",
            text.pair("Notification Center", "通知中心"),
            text.pair(
                "Desktop notifications are reserved; v1 shows local task feedback in the Hub window.",
                "桌面通知为预留能力；v1 在 Hub 窗口内显示本地任务反馈。",
            ),
        ),
        (
            "sign-out",
            "shell",
            text.pair("Sign Out", "退出登录"),
            text.pair(
                "Remote accounts are disabled for the local-only Hub.",
                "本地版 Hub 不启用远程账号。",
            ),
        ),
        (
            "team-invite",
            "team",
            text.pair("Invite Members", "邀请成员"),
            text.pair(
                "Team invitations require a remote collaboration service and are reserved.",
                "团队邀请依赖远程协作服务，当前仅预留。",
            ),
        ),
        (
            "team-permissions",
            "team",
            text.pair("Permissions", "权限"),
            text.pair(
                "Permission management is disabled for the local-only Hub.",
                "本地版 Hub 暂不启用权限管理。",
            ),
        ),
        (
            "remote-collaboration",
            "team",
            text.pair("Remote Collaboration", "远程协作"),
            text.pair(
                "Remote collaboration is outside the v1 desktop-local loop.",
                "远程协作不属于 v1 桌面本地闭环。",
            ),
        ),
    ]
    .into_iter()
    .map(|(id, category, title, detail)| {
        let category_label = coming_soon_category_label(category, text);
        let status = text.pair("Coming Soon", "敬请期待");
        HubComingSoonEntry {
            id: Cow::Borrowed(id),
            category: Cow::Borrowed(category),
            meta: coming_soon_meta(category_label, status, text),
            category_label: Cow::Borrowed(category_label),
            title: Cow::Borrowed(title),
            detail: Cow::Borrowed(detail),
            status: Cow::Borrowed(status),
            disabled: true,
        }
    })
    .collect()
}

// 类别与状态的组合标点由语言投影拥有，页面只展示完整文案。
fn coming_soon_meta(category_label: &str, status: &str, text: HubTextBundle) -> String {
    format!("{}{}{}", category_label, text.pair(" / ", " / "), status)
}

fn coming_soon_category_label(category: &str, text: HubTextBundle) -> &'static str {
    match category {
        "assets" => text.pair("Assets", "资产"),
        "projects" => text.pair("Projects", "项目"),
        "plugins" => text.pair("Plugins", "插件"),
        "local-delivery" => text.pair("Local Delivery", "本地交付"),
        "shell" => text.pair("Shell", "外壳"),
        "team" => text.pair("Team", "团队"),
        _ => text.pair("Reserved", "预留"),
    }
}

#[cfg(test)]
#[path = "tests/coming_soon.rs"]
mod tests;
