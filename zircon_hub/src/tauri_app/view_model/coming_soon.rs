use serde::Serialize;

use crate::settings::HubLanguage;

use super::localized::HubTextBundle;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubComingSoonEntry {
    pub id: String,
    pub category: String,
    pub category_label: String,
    pub title: String,
    pub detail: String,
    pub status: String,
    pub meta: String,
    pub disabled: bool,
}

pub(crate) fn coming_soon_entries(language: HubLanguage) -> Vec<HubComingSoonEntry> {
    let text = HubTextBundle::new(language);
    [
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
        let category_label = coming_soon_category_label(category, text).to_string();
        let status = text.pair("Coming Soon", "敬请期待").to_string();
        HubComingSoonEntry {
            id: id.to_string(),
            category: category.to_string(),
            meta: coming_soon_meta(&category_label, &status, text),
            category_label,
            title: title.to_string(),
            detail: detail.to_string(),
            status,
            disabled: true,
        }
    })
    .collect()
}

fn coming_soon_meta(category_label: &str, status: &str, text: HubTextBundle) -> String {
    format!("{}{}{}", category_label, text.pair(" / ", " / "), status)
}

fn coming_soon_category_label(category: &str, text: HubTextBundle) -> &'static str {
    match category {
        "assets" => text.pair("Assets", "资产"),
        "plugins" => text.pair("Plugins", "插件"),
        "local-delivery" => text.pair("Local Delivery", "本地交付"),
        "team" => text.pair("Team", "团队"),
        _ => text.pair("Reserved", "预留"),
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::HubLanguage;

    #[test]
    fn coming_soon_entries_include_visible_localized_category_labels() {
        let entries = super::coming_soon_entries(HubLanguage::Chinese);
        let remote_sync = entries
            .iter()
            .find(|entry| entry.id == "remote-sync")
            .expect("remote sync entry should exist");

        assert_eq!(remote_sync.category, "local-delivery");
        assert_eq!(remote_sync.category_label, "本地交付");
        assert_eq!(remote_sync.status, "敬请期待");
        assert_eq!(remote_sync.meta, "本地交付 / 敬请期待");
        assert!(remote_sync.disabled);
    }
}
