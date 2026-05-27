//! Static contracts for Hub selected-project page copy and scope labels.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .to_string_lossy()
            .into_owned()
    }))
}

fn read_crate_file(path: &str) -> String {
    fs::read_to_string(crate_dir().join(path))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn cloud_package_state_respects_unconfigured_output_path() {
    let cloud = read_crate_file("src/app/view_model/cloud.rs");
    for snippet in [
        "let package_root = if snapshot\n        .settings\n        .default_build_output_dir\n        .as_os_str()\n        .is_empty()",
        "let package_path = package_root.as_deref().unwrap_or_else(|| Path::new(\"\"));",
        "package_path: shared(path_text(package_path, language)),",
        "package_status: package_state(",
        "fn cloud_summary_reports_package_not_configured_without_output_root()",
    ] {
        assert!(
            cloud.contains(snippet),
            "Cloud package state should stay Not configured when the output root is empty instead of scanning a relative packages directory; missing {snippet}"
        );
    }
}

#[test]
fn builds_navigation_copy_targets_selected_project() {
    let navigation = read_crate_file("src/state/navigation.rs");
    let localization = read_crate_file("src/app/localization.rs");

    for snippet in [
        "Self::Builds => \"Build and package workflows for the selected project.\",",
        "Build and package workflows for the selected project.",
        "选中项目的构建与打包工作流。",
        "build_controls: text(language, \"Selected Project Actions\", \"选中项目操作\"),",
        "No build history has been recorded for the selected project or active Source Engine.",
        "选中项目或当前 Source Engine 还没有构建历史。",
    ] {
        assert!(
            navigation.contains(snippet) || localization.contains(snippet),
            "Builds navigation and page subtitle should describe the selected-project workflow; missing {snippet}"
        );
    }

    for forbidden in [
        "Self::Builds => \"Build and package workflows for the active project.\",",
        "Build and package workflows for the active project.",
        "当前项目的构建与打包工作流。",
        "build_controls: text(language, \"Build Controls\", \"构建控制\"),",
        "No build history has been recorded for the active source engine.",
        "当前源码引擎还没有构建历史。",
    ] {
        assert!(
            !navigation.contains(forbidden) && !localization.contains(forbidden),
            "Builds copy must not return to ambiguous active-project wording: {forbidden}"
        );
    }
}

#[test]
fn shell_context_copy_uses_selected_project_label() {
    let localization = read_crate_file("src/app/localization.rs");
    let shared = read_crate_file("ui/shared.slint");
    let shell = read_crate_file("ui/shell.slint");

    assert!(
        localization.contains("current_project: text(language, \"Selected Project\", \"选中项目\"),"),
        "Shell project context label should say Selected Project while retaining the existing UiTextData field"
    );
    assert!(
        shared.contains("current-project: string,"),
        "UiTextData should retain the current-project field as an additive-compatible Slint interface"
    );
    assert!(
        shell.contains("root.ui-text.current-project"),
        "Shell chrome should continue consuming the shared project-context label from UiTextData"
    );
    assert!(
        !localization
            .contains("current_project: text(language, \"Current Project\", \"当前项目\"),"),
        "Project context copy must not return to ambiguous Current Project wording"
    );
}

#[test]
fn assets_navigation_copy_targets_selected_project_and_source_engine() {
    let navigation = read_crate_file("src/state/navigation.rs");
    let localization = read_crate_file("src/app/localization.rs");

    for snippet in [
        "Self::Assets => \"Browse selected project and Source Engine assets.\",",
        "Browse selected project and Source Engine assets.",
        "浏览选中项目和 Source Engine 资产。",
        "No assets were found in checked project folders or Source Engine asset roots.",
        "已检查的项目目录或 Source Engine 资产根目录中没有发现资产。",
        "Checked the selected project's asset folders, recent project asset folders, and Source Engine asset roots.",
        "Checked recent project asset folders and Source Engine asset roots.",
    ] {
        assert!(
            navigation.contains(snippet) || localization.contains(snippet),
            "Assets navigation and page subtitle should describe selected-project and Source Engine asset scope; missing {snippet}"
        );
    }

    for forbidden in [
        "Self::Assets => \"Asset library integration will live here.\",",
        "Asset library integration will live here.",
        "Browse discovered project and engine assets.",
        "浏览已发现的项目和引擎资产。",
        "No assets were found in recent project folders or the active source checkout.",
        "最近项目目录或当前源码检出中没有发现资产。",
        "Checked recent project asset folders and the active Source Engine checkout.",
        "Checked the selected project's asset folders, recent project asset folders, and Source Engine assets.",
    ] {
        assert!(
            !navigation.contains(forbidden) && !localization.contains(forbidden),
            "Assets copy must not return to placeholder or ambiguous discovered-assets wording: {forbidden}"
        );
    }
}

#[test]
fn plugins_navigation_copy_targets_project_manifests_and_source_engine() {
    let navigation = read_crate_file("src/state/navigation.rs");
    let localization = read_crate_file("src/app/localization.rs");

    for snippet in [
        "Self::Plugins => \"Browse selected project plugin manifests and Source Engine plugins.\",",
        "Browse selected project plugin manifests and Source Engine plugins.",
        "浏览选中项目插件清单和 Source Engine 插件。",
        "No plugin manifests were found in checked project folders or Source Engine plugin roots.",
        "已检查的项目目录或 Source Engine 插件根目录中没有发现插件清单。",
        "Checked the selected project plugin manifests and Source Engine plugin roots.",
        "Checked Source Engine plugin roots and local repository fallback roots.",
    ] {
        assert!(
            navigation.contains(snippet) || localization.contains(snippet),
            "Plugins navigation and page subtitle should describe selected-project plugin manifests and Source Engine plugin scope; missing {snippet}"
        );
    }

    for forbidden in [
        "Self::Plugins => \"Plugin discovery and project extensions.\",",
        "Plugin discovery and project extensions.",
        "插件发现和项目扩展。",
        "No plugin manifests were found in the active source checkout.",
        "当前源码检出中没有发现插件清单。",
        "Checked the selected project plugin manifests and the active Source Engine plugin tree.",
        "Checked the active Source Engine plugin tree and local repository fallback roots.",
    ] {
        assert!(
            !navigation.contains(forbidden) && !localization.contains(forbidden),
            "Plugins copy must not return to generic discovery/project-extension wording: {forbidden}"
        );
    }
}

#[test]
fn cloud_navigation_copy_stays_local_and_offline() {
    let navigation = read_crate_file("src/state/navigation.rs");
    let cloud_ui = read_crate_file("ui/cloud.slint");
    let shared = read_crate_file("ui/shared.slint");
    let localization = read_crate_file("src/app/localization.rs");
    assert!(
        navigation.contains("Self::Cloud => \"Packages\","),
        "Cloud route should present as Packages in the navigation title while keeping the stable cloud id"
    );
    assert!(
        localization.contains("HubPage::Cloud => text(language, \"Packages\", \"本地包\"),"),
        "Localized Cloud page title should present as Packages/local package instead of a remote cloud service"
    );
    assert!(
        navigation.contains("Self::Cloud => \"Local packages, installs, and reserved service slots.\","),
        "Cloud navigation subtitle should describe local packages/install/service slots instead of an account surface"
    );
    for snippet in [
        "cloud_overview: text(language, \"Local Package Overview\", \"本地包状态概览\"),",
        "Local Packages - Selected Project",
        "Local packages, installs, output status, and reserved service slots.",
        "local-mode-status: string,",
        "cloud-local-mode: string,",
        "cloud_local_mode: text(language, \"Local Mode\", \"本地模式\"),",
        "label: root.ui-text.cloud-local-mode;",
        "primary: root.summary.local-mode-status;",
    ] {
        assert!(
            shared.contains(snippet) || localization.contains(snippet) || cloud_ui.contains(snippet),
            "Cloud visible local-mode metric should use local/offline field names while retaining additive compatibility; missing {snippet}"
        );
    }
    for alias in ["account-status: string,", "cloud-account: string,"] {
        assert!(
            shared.contains(alias),
            "Cloud compatibility aliases should remain additive while the UI consumes local-mode fields; missing {alias}"
        );
    }
    for forbidden in [
        "Self::Cloud => \"Cloud\",",
        "HubPage::Cloud => text(language, \"Cloud\", \"云服务\"),",
        "Self::Cloud => \"Cloud services and account connections.\"",
        "account connections",
        "Local Cloud Overview",
        "Local Cloud - Selected Project",
        "Local cloud readiness",
        "本地云服务",
        "label: root.ui-text.cloud-account;",
        "primary: root.summary.account-status;",
    ] {
        assert!(
            !navigation.contains(forbidden)
                && !cloud_ui.contains(forbidden)
                && !localization.contains(forbidden),
            "Cloud navigation copy must not return to account-service wording: {forbidden}"
        );
    }
    assert!(
        cloud_ui.contains("trailing-tone: root.service.status == \"Ready\" ? \"ok\" : \"neutral\";"),
        "Cloud service rows should use local/offline neutral status tones without a stale Not connected branch"
    );
}

#[test]
fn team_navigation_copy_stays_local_git_only() {
    let navigation = read_crate_file("src/state/navigation.rs");
    let localization = read_crate_file("src/app/localization.rs");
    assert!(
        navigation.contains("Self::Team => \"Local Git identity and recent contributors.\","),
        "Team navigation subtitle should describe local Git identity/contributors instead of membership or collaboration settings"
    );
    for snippet in [
        "Checked the selected project's Git repository first, then Source Engine and local fallback repositories.",
        "Checked Source Engine and local fallback repositories.",
    ] {
        assert!(
            localization.contains(snippet),
            "Team empty-state detail should describe selected-project and Source Engine/local Git scope; missing {snippet}"
        );
    }
    for forbidden in [
        "Self::Team => \"Team membership and collaboration settings.\"",
        "Team membership",
        "collaboration settings",
        "Checked the active Source Engine repository and local fallback repositories.",
    ] {
        assert!(
            !navigation.contains(forbidden) && !localization.contains(forbidden),
            "Team navigation copy must not return to collaboration/account-management wording: {forbidden}"
        );
    }
}
