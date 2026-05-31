use std::fs;
use std::path::Path;

use slint::SharedString;

use crate::projects::project_paths_match;
use crate::settings::HubLanguage;
use crate::state::{HubSnapshot, ProjectScope};

use super::super::{CloudServiceData, CloudSummaryData};
use super::localization;

const PACKAGE_MANIFEST_FILE: &str = "zircon-package.toml";

#[derive(serde::Deserialize)]
struct PackageManifestScope {
    source_project: Option<String>,
}

pub(super) fn cloud_summary(snapshot: &HubSnapshot) -> CloudSummaryData {
    let language = snapshot.settings.language;
    let scope = snapshot.scope();
    let package_root = if snapshot
        .settings
        .default_build_output_dir
        .as_os_str()
        .is_empty()
    {
        None
    } else {
        Some(snapshot.settings.default_build_output_dir.join("packages"))
    };
    let package_path = package_root.as_deref().unwrap_or_else(|| Path::new(""));
    let package_action = selected_project_action_readiness(
        &scope.project,
        !snapshot
            .settings
            .default_build_output_dir
            .as_os_str()
            .is_empty(),
        language,
        localization::text(
            language,
            "Select a project before packaging",
            "请先选择项目再打包",
        ),
        localization::text(
            language,
            "Configure package output root before packaging",
            "请先配置包输出根目录再打包",
        ),
    );
    let install_action = selected_project_action_readiness(
        &scope.project,
        !snapshot
            .settings
            .default_device_install_dir
            .as_os_str()
            .is_empty(),
        language,
        localization::text(
            language,
            "Select a project before installing",
            "请先选择项目再安装",
        ),
        localization::text(
            language,
            "Configure device install directory before installing",
            "请先配置设备安装目录再安装",
        ),
    );

    CloudSummaryData {
        status: localization::text(language, "Offline local mode", "离线本地模式"),
        account_status: localization::text(language, "Local only", "仅本地"),
        local_mode_status: localization::text(language, "Local only", "仅本地"),
        package_action_detail: package_action.detail,
        package_action_enabled: package_action.enabled,
        install_action_detail: install_action.detail,
        install_action_enabled: install_action.enabled,
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
        device_status: install_state(
            &snapshot.settings.default_device_install_dir,
            language,
            snapshot.selected_project_path.as_deref(),
        ),
        package_path: shared(path_text(package_path, language)),
        package_status: package_state(
            package_path,
            language,
            snapshot.selected_project_path.as_deref(),
        ),
        operation_timeline_title: localization::text(
            language,
            "Cloud Operation Status",
            "云操作状态",
        ),
        operation_timeline_empty_title: localization::text(
            language,
            "No package or install operations yet",
            "尚无打包或安装操作",
        ),
        operation_timeline_empty_detail: localization::text(
            language,
            "Package or install the selected project to persist local operation status here.",
            "打包或安装选中项目后，本地操作状态会保存在这里。",
        ),
    }
}

pub(super) fn cloud_services(language: HubLanguage) -> Vec<CloudServiceData> {
    [
        (
            localization::text(language, "Profile Sync Slot", "资料同步槽位"),
            localization::text(
                language,
                "Reserved for optional profile synchronization after local workflows are stable.",
                "本地工作流稳定后预留给可选资料同步。",
            ),
            localization::text(language, "Reserved", "预留"),
        ),
        (
            localization::text(language, "Remote Build Slot", "远程构建槽位"),
            localization::text(
                language,
                "Reserved for hosted build workers after local packaging is stable.",
                "本地打包稳定后预留给托管构建节点。",
            ),
            localization::text(language, "Local only", "仅本地"),
        ),
        (
            localization::text(language, "Artifact Upload Slot", "产物上传槽位"),
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

fn package_state(
    package_root: &Path,
    language: HubLanguage,
    selected_project_path: Option<&Path>,
) -> SharedString {
    if package_root.as_os_str().is_empty() {
        return localization::text(language, "Not configured", "未配置");
    }
    if !package_root.is_dir() {
        if selected_project_path.is_some() {
            return localization::text(
                language,
                "No local packages for selected project yet",
                "选中项目尚无本地包",
            );
        }
        return localization::text(language, "No local packages yet", "尚无本地包");
    }
    let selected_project = selected_project_path.filter(|path| !path.as_os_str().is_empty());
    let package_count = selected_project
        .map(|path| selected_project_package_count(package_root, path))
        .unwrap_or_else(|| package_directory_count(package_root));
    if package_count == 0 {
        if selected_project.is_some() {
            return localization::text(
                language,
                "No local packages for selected project yet",
                "选中项目尚无本地包",
            );
        }
        return localization::text(language, "No local packages yet", "尚无本地包");
    }
    match language {
        HubLanguage::English if selected_project.is_some() => {
            let noun = if package_count == 1 {
                "package"
            } else {
                "packages"
            };
            SharedString::from(format!("{package_count} local {noun} for selected project"))
        }
        HubLanguage::English => {
            let noun = if package_count == 1 {
                "package"
            } else {
                "packages"
            };
            SharedString::from(format!("{package_count} local {noun}"))
        }
        HubLanguage::Chinese if selected_project.is_some() => {
            SharedString::from(format!("选中项目 {package_count} 个本地包"))
        }
        HubLanguage::Chinese => SharedString::from(format!("{package_count} 个本地包")),
    }
}

fn install_state(
    install_root: &Path,
    language: HubLanguage,
    selected_project_path: Option<&Path>,
) -> SharedString {
    let Some(selected_project) = selected_project_path.filter(|path| !path.as_os_str().is_empty())
    else {
        return directory_state(
            install_root,
            language,
            localization::text(language, "Ready", "就绪"),
            localization::text(language, "Created by local install", "由本地安装创建"),
        );
    };
    if install_root.as_os_str().is_empty() {
        return localization::text(language, "Not configured", "未配置");
    }
    if !install_root.is_dir() {
        return localization::text(
            language,
            "No local install for selected project yet",
            "选中项目尚无本地安装",
        );
    }
    let install_count = selected_project_install_count(install_root, selected_project);
    if install_count == 0 {
        return localization::text(
            language,
            "No local install for selected project yet",
            "选中项目尚无本地安装",
        );
    }
    match language {
        HubLanguage::English => {
            let noun = if install_count == 1 {
                "install"
            } else {
                "installs"
            };
            SharedString::from(format!("{install_count} local {noun} for selected project"))
        }
        HubLanguage::Chinese => SharedString::from(format!("选中项目 {install_count} 个本地安装")),
    }
}

fn package_directory_count(package_root: &Path) -> usize {
    fs::read_dir(package_root)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_dir())
                .count()
        })
        .unwrap_or_default()
}

fn selected_project_package_count(package_root: &Path, selected_project_path: &Path) -> usize {
    fs::read_dir(package_root)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    package_matches_selected_project(&entry.path(), selected_project_path)
                })
                .count()
        })
        .unwrap_or_default()
}

fn selected_project_install_count(install_root: &Path, selected_project_path: &Path) -> usize {
    selected_project_package_count(install_root, selected_project_path)
}

fn package_matches_selected_project(package_dir: &Path, selected_project_path: &Path) -> bool {
    if !package_dir.is_dir() {
        return false;
    }

    let manifest_path = package_dir.join(PACKAGE_MANIFEST_FILE);
    let Ok(manifest) = fs::read_to_string(manifest_path) else {
        return false;
    };
    let Ok(manifest) = toml::from_str::<PackageManifestScope>(&manifest) else {
        return false;
    };
    let Some(source_project) = manifest.source_project.as_deref() else {
        return false;
    };

    project_paths_match(Path::new(source_project), selected_project_path)
}

fn path_text(path: &Path, language: HubLanguage) -> String {
    if path.as_os_str().is_empty() {
        return localization::text(language, "Not configured", "未配置").to_string();
    }
    path.to_string_lossy().into_owned()
}

struct CloudActionReadiness {
    detail: SharedString,
    enabled: bool,
}

fn selected_project_action_readiness(
    project_scope: &ProjectScope,
    local_root_configured: bool,
    language: HubLanguage,
    missing_detail: SharedString,
    unconfigured_detail: SharedString,
) -> CloudActionReadiness {
    let project_path = match project_scope {
        ProjectScope::Selected(project) => project.path.as_path(),
        ProjectScope::StaleSelection { requested_path } => {
            return CloudActionReadiness {
                detail: stale_project_detail(requested_path, language),
                enabled: false,
            };
        }
        ProjectScope::LatestRecent(_) | ProjectScope::None => {
            return CloudActionReadiness {
                detail: missing_detail,
                enabled: false,
            };
        }
    };

    if !local_root_configured {
        return CloudActionReadiness {
            detail: unconfigured_detail,
            enabled: false,
        };
    }

    CloudActionReadiness {
        detail: shared(path_text(project_path, language)),
        enabled: true,
    }
}

fn stale_project_detail(requested_path: &Path, language: HubLanguage) -> SharedString {
    match language {
        HubLanguage::English => SharedString::from(format!(
            "Selected project is no longer in the recent-project registry: {}",
            path_text(requested_path, language)
        )),
        HubLanguage::Chinese => SharedString::from(format!(
            "选中项目已不在最近项目登记中：{}",
            path_text(requested_path, language)
        )),
    }
}

fn shared(value: impl Into<SharedString>) -> SharedString {
    value.into()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::projects::{ProjectMetadataMap, RecentProject};
    use crate::settings::{HubLanguage, HubSettings};
    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    fn cloud_snapshot(
        settings: HubSettings,
        selected_project_path: Option<PathBuf>,
    ) -> HubSnapshot {
        HubSnapshot {
            selected_page: HubPage::Cloud,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path,
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
            action_history: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            settings,
        }
    }

    fn write_package_manifest(package_dir: &Path, source_project: &Path) {
        fs::create_dir_all(package_dir).unwrap();
        let source_project = source_project
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        fs::write(
            package_dir.join(PACKAGE_MANIFEST_FILE),
            format!("source_project = \"{source_project}\"\n"),
        )
        .unwrap();
    }

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
        let snapshot = cloud_snapshot(settings, None);

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
        assert_eq!(services[0].title, SharedString::from("Profile Sync Slot"));
        assert_eq!(services[0].status, SharedString::from("Reserved"));
        assert_eq!(services[2].status, SharedString::from("Offline"));
    }

    #[test]
    fn cloud_summary_reports_package_not_configured_without_output_root() {
        let mut settings = HubSettings::default();
        settings.default_build_output_dir = PathBuf::new();
        let snapshot = cloud_snapshot(settings, None);

        let summary = cloud_summary(&snapshot);

        assert_eq!(summary.package_path, SharedString::from("Not configured"));
        assert_eq!(summary.package_status, SharedString::from("Not configured"));
    }

    #[test]
    fn cloud_summary_disables_package_action_without_output_root() {
        let selected_project = PathBuf::from("E:/projects/demo");
        let mut settings = HubSettings::default();
        settings.default_build_output_dir = PathBuf::new();
        let mut snapshot = cloud_snapshot(settings, Some(selected_project.clone()));
        snapshot.recent_projects = vec![RecentProject::new("Demo", selected_project, 42)];

        let summary = cloud_summary(&snapshot);

        assert_eq!(
            summary.package_action_detail,
            SharedString::from("Configure package output root before packaging")
        );
        assert!(!summary.package_action_enabled);
        assert_eq!(
            summary.install_action_detail,
            SharedString::from("E:/projects/demo")
        );
        assert!(summary.install_action_enabled);
    }

    #[test]
    fn cloud_summary_disables_install_action_without_device_root() {
        let selected_project = PathBuf::from("E:/projects/demo");
        let mut settings = HubSettings::default();
        settings.default_device_install_dir = PathBuf::new();
        let mut snapshot = cloud_snapshot(settings, Some(selected_project.clone()));
        snapshot.recent_projects = vec![RecentProject::new("Demo", selected_project, 42)];

        let summary = cloud_summary(&snapshot);

        assert_eq!(
            summary.package_action_detail,
            SharedString::from("E:/projects/demo")
        );
        assert!(summary.package_action_enabled);
        assert_eq!(
            summary.install_action_detail,
            SharedString::from("Configure device install directory before installing")
        );
        assert!(!summary.install_action_enabled);
    }

    #[test]
    fn cloud_summary_labels_selected_project_package_state() {
        let mut settings = HubSettings::default();
        settings.default_build_output_dir = std::env::temp_dir().join(format!(
            "zircon-hub-cloud-selected-{}",
            crate::projects::now_unix_ms()
        ));
        let selected_project = PathBuf::from("E:/projects/demo");
        let mut snapshot = cloud_snapshot(settings, Some(selected_project.clone()));
        snapshot.recent_projects = vec![RecentProject::new("Demo", selected_project, 42)];

        let summary = cloud_summary(&snapshot);

        assert_eq!(
            summary.package_status,
            SharedString::from("No local packages for selected project yet")
        );
        assert_eq!(
            summary.package_action_detail,
            SharedString::from("E:/projects/demo")
        );
        assert!(summary.package_action_enabled);
        assert_eq!(
            summary.install_action_detail,
            SharedString::from("E:/projects/demo")
        );
        assert!(summary.install_action_enabled);
    }

    #[test]
    fn cloud_summary_disables_project_actions_without_selection() {
        let summary = cloud_summary(&cloud_snapshot(HubSettings::default(), None));

        assert_eq!(
            summary.package_action_detail,
            SharedString::from("Select a project before packaging")
        );
        assert!(!summary.package_action_enabled);
        assert_eq!(
            summary.install_action_detail,
            SharedString::from("Select a project before installing")
        );
        assert!(!summary.install_action_enabled);
    }

    #[test]
    fn cloud_summary_disables_project_actions_for_stale_selection() {
        let mut snapshot = cloud_snapshot(
            HubSettings::default(),
            Some(PathBuf::from("E:/projects/missing")),
        );
        snapshot.recent_projects = vec![RecentProject::new("Demo", "E:/projects/demo", 42)];
        snapshot.project_metadata = ProjectMetadataMap::new();

        let summary = cloud_summary(&snapshot);

        assert_eq!(
            summary.package_action_detail,
            SharedString::from(
                "Selected project is no longer in the recent-project registry: E:/projects/missing"
            )
        );
        assert!(!summary.package_action_enabled);
        assert_eq!(
            summary.install_action_detail,
            SharedString::from(
                "Selected project is no longer in the recent-project registry: E:/projects/missing"
            )
        );
        assert!(!summary.install_action_enabled);
    }

    #[test]
    fn cloud_summary_counts_only_selected_project_packages() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-cloud-selected-packages-{}",
            crate::projects::now_unix_ms()
        ));
        let packages = root.join("packages");
        let selected_project = root.join("selected-project");
        let other_project = root.join("other-project");
        fs::create_dir_all(&selected_project).unwrap();
        fs::create_dir_all(&other_project).unwrap();
        write_package_manifest(&packages.join("selected-42"), &selected_project);
        write_package_manifest(&packages.join("other-42"), &other_project);

        let mut settings = HubSettings::default();
        settings.default_build_output_dir = root.clone();
        let snapshot = cloud_snapshot(settings, Some(selected_project));

        let summary = cloud_summary(&snapshot);
        fs::remove_dir_all(root).unwrap();

        assert_eq!(
            summary.package_status,
            SharedString::from("1 local package for selected project")
        );
    }

    #[test]
    fn cloud_summary_ignores_other_project_packages_when_project_selected() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-cloud-other-packages-{}",
            crate::projects::now_unix_ms()
        ));
        let packages = root.join("packages");
        let selected_project = root.join("selected-project");
        let other_project = root.join("other-project");
        fs::create_dir_all(&selected_project).unwrap();
        fs::create_dir_all(&other_project).unwrap();
        write_package_manifest(&packages.join("other-42"), &other_project);

        let mut settings = HubSettings::default();
        settings.default_build_output_dir = root.clone();
        let snapshot = cloud_snapshot(settings, Some(selected_project));

        let summary = cloud_summary(&snapshot);
        fs::remove_dir_all(root).unwrap();

        assert_eq!(
            summary.package_status,
            SharedString::from("No local packages for selected project yet")
        );
    }

    #[test]
    fn cloud_summary_counts_only_selected_project_installs() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-cloud-selected-installs-{}",
            crate::projects::now_unix_ms()
        ));
        let installs = root.join("device");
        let selected_project = root.join("selected-project");
        let other_project = root.join("other-project");
        fs::create_dir_all(&selected_project).unwrap();
        fs::create_dir_all(&other_project).unwrap();
        write_package_manifest(&installs.join("selected-42"), &selected_project);
        write_package_manifest(&installs.join("other-42"), &other_project);

        let mut settings = HubSettings::default();
        settings.default_device_install_dir = installs;
        let snapshot = cloud_snapshot(settings, Some(selected_project));

        let summary = cloud_summary(&snapshot);
        fs::remove_dir_all(root).unwrap();

        assert_eq!(
            summary.device_status,
            SharedString::from("1 local install for selected project")
        );
    }

    #[test]
    fn cloud_summary_ignores_other_project_installs_when_project_selected() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-cloud-other-installs-{}",
            crate::projects::now_unix_ms()
        ));
        let installs = root.join("device");
        let selected_project = root.join("selected-project");
        let other_project = root.join("other-project");
        fs::create_dir_all(&selected_project).unwrap();
        fs::create_dir_all(&other_project).unwrap();
        write_package_manifest(&installs.join("other-42"), &other_project);

        let mut settings = HubSettings::default();
        settings.default_device_install_dir = installs;
        let snapshot = cloud_snapshot(settings, Some(selected_project));

        let summary = cloud_summary(&snapshot);
        fs::remove_dir_all(root).unwrap();

        assert_eq!(
            summary.device_status,
            SharedString::from("No local install for selected project yet")
        );
    }
}
