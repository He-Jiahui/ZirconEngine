use std::fs;
use std::path::Path;

use slint::SharedString;

use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::super::{CloudServiceData, CloudSummaryData};
use super::localization;

pub(super) fn cloud_summary(snapshot: &HubSnapshot) -> CloudSummaryData {
    let language = snapshot.settings.language;
    let package_root = snapshot.settings.default_build_output_dir.join("packages");

    CloudSummaryData {
        status: localization::text(language, "Offline local mode", "离线本地模式"),
        account_status: localization::text(language, "Not connected", "未连接"),
        output_path: shared(path_text(
            &snapshot.settings.default_build_output_dir,
            language,
        )),
        output_status: directory_state(
            &snapshot.settings.default_build_output_dir,
            language,
            localization::text(language, "Ready", "就绪"),
            localization::text(language, "Created by local builds", "由本地构建创建"),
        ),
        device_path: shared(path_text(
            &snapshot.settings.default_device_install_dir,
            language,
        )),
        device_status: directory_state(
            &snapshot.settings.default_device_install_dir,
            language,
            localization::text(language, "Ready", "就绪"),
            localization::text(language, "Created by local install", "由本地安装创建"),
        ),
        package_path: shared(path_text(&package_root, language)),
        package_status: package_state(&package_root, language),
    }
}

pub(super) fn cloud_services(language: HubLanguage) -> Vec<CloudServiceData> {
    [
        (
            localization::text(language, "Account Sync", "账号同步"),
            localization::text(
                language,
                "Reserved for sign-in, license, and profile synchronization.",
                "预留给登录、授权和资料同步。",
            ),
            localization::text(language, "Not connected", "未连接"),
        ),
        (
            localization::text(language, "Remote Build", "远程构建"),
            localization::text(
                language,
                "Reserved for hosted build workers after local packaging is stable.",
                "本地打包稳定后预留给托管构建节点。",
            ),
            localization::text(language, "Local only", "仅本地"),
        ),
        (
            localization::text(language, "Package Upload", "包上传"),
            localization::text(
                language,
                "Reserved for artifact upload from the local package directory.",
                "预留给从本地包目录上传产物。",
            ),
            localization::text(language, "Offline", "离线"),
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (title, detail, status))| CloudServiceData {
        title,
        detail,
        status,
        accent: index as i32,
    })
    .collect()
}

fn directory_state(
    path: &Path,
    language: HubLanguage,
    exists_text: SharedString,
    missing_text: SharedString,
) -> SharedString {
    if path.as_os_str().is_empty() {
        return localization::text(language, "Not configured", "未配置");
    }
    if path.is_dir() {
        return exists_text;
    }
    missing_text
}

fn package_state(package_root: &Path, language: HubLanguage) -> SharedString {
    if package_root.as_os_str().is_empty() {
        return localization::text(language, "Not configured", "未配置");
    }
    if !package_root.is_dir() {
        return localization::text(language, "No local packages yet", "尚无本地包");
    }
    let package_count = fs::read_dir(package_root)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_dir())
                .count()
        })
        .unwrap_or_default();
    if package_count == 0 {
        return localization::text(language, "No local packages yet", "尚无本地包");
    }
    match language {
        HubLanguage::English => SharedString::from(format!("{package_count} local packages")),
        HubLanguage::Chinese => SharedString::from(format!("{package_count} 个本地包")),
    }
}

fn path_text(path: &Path, language: HubLanguage) -> String {
    if path.as_os_str().is_empty() {
        return localization::text(language, "Not configured", "未配置").to_string();
    }
    path.to_string_lossy().into_owned()
}

fn shared(value: impl Into<SharedString>) -> SharedString {
    value.into()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::settings::{HubLanguage, HubSettings};
    use crate::state::{HubPage, ProjectFilterMode, ProjectSortMode, ProjectViewMode, TaskStatus};

    use super::*;

    #[test]
    fn cloud_summary_reports_package_count_from_output_root() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-cloud-packages-{}",
            crate::projects::now_unix_ms()
        ));
        let packages = root.join("packages");
        fs::create_dir_all(packages.join("first")).unwrap();
        fs::create_dir_all(packages.join("second")).unwrap();

        let mut settings = HubSettings::default();
        settings.default_build_output_dir = root.clone();
        settings.default_device_install_dir = PathBuf::new();
        let snapshot = HubSnapshot {
            selected_page: HubPage::Cloud,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: String::new(),
            selected_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            engines: Vec::new(),
            active_engine_id: None,
            settings,
        };

        let summary = cloud_summary(&snapshot);
        fs::remove_dir_all(root).unwrap();

        assert_eq!(
            summary.package_status,
            SharedString::from("2 local packages")
        );
        assert_eq!(summary.device_status, SharedString::from("Not configured"));
    }

    #[test]
    fn cloud_services_are_reserved_local_slots() {
        let services = cloud_services(HubLanguage::English);

        assert_eq!(services.len(), 3);
        assert_eq!(services[0].title, SharedString::from("Account Sync"));
        assert_eq!(services[2].status, SharedString::from("Offline"));
    }
}
