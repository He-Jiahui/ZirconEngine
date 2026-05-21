use std::fs;
use std::path::Path;

use slint::SharedString;

use crate::settings::HubLanguage;
use crate::state::HubSnapshot;

use super::super::{CloudServiceData, CloudSummaryData};
use super::localization;

const PACKAGE_MANIFEST_FILE: &str = "zircon-package.toml";
const PACKAGE_SOURCE_PROJECT_KEY: &str = "source_project";

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
        package_status: package_state(
            &package_root,
            language,
            snapshot.selected_project_path.as_deref(),
        ),
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

fn package_matches_selected_project(package_dir: &Path, selected_project_path: &Path) -> bool {
    if !package_dir.is_dir() {
        return false;
    }

    let manifest_path = package_dir.join(PACKAGE_MANIFEST_FILE);
    let Ok(manifest) = fs::read_to_string(manifest_path) else {
        return false;
    };
    let Ok(manifest) = manifest.parse::<toml::Value>() else {
        return false;
    };
    let Some(source_project) = manifest
        .get(PACKAGE_SOURCE_PROJECT_KEY)
        .and_then(toml::Value::as_str)
    else {
        return false;
    };

    paths_match_for_summary(Path::new(source_project), selected_project_path)
}

fn paths_match_for_summary(left: &Path, right: &Path) -> bool {
    if let (Ok(left), Ok(right)) = (left.canonicalize(), right.canonicalize()) {
        return left == right;
    }

    normalized_path_key(left) == normalized_path_key(right)
}

fn normalized_path_key(path: &Path) -> String {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();
    if cfg!(windows) {
        normalized.to_ascii_lowercase()
    } else {
        normalized
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
    use std::path::{Path, PathBuf};

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
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            recent_projects: Vec::new(),
            project_metadata: crate::projects::ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: crate::team::TeamOverview::empty(),
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
            format!("{PACKAGE_SOURCE_PROJECT_KEY} = \"{source_project}\"\n"),
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
        assert_eq!(services[0].title, SharedString::from("Account Sync"));
        assert_eq!(services[2].status, SharedString::from("Offline"));
    }

    #[test]
    fn cloud_summary_labels_selected_project_package_state() {
        let mut settings = HubSettings::default();
        settings.default_build_output_dir = std::env::temp_dir().join(format!(
            "zircon-hub-cloud-selected-{}",
            crate::projects::now_unix_ms()
        ));
        let snapshot = cloud_snapshot(settings, Some(PathBuf::from("E:/projects/demo")));

        let summary = cloud_summary(&snapshot);

        assert_eq!(
            summary.package_status,
            SharedString::from("No local packages for selected project yet")
        );
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
}
