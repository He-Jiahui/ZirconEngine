//! Static contracts for Hub selected-project scope projection and page data surfacing.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_ui_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(ui_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub UI file {name}: {error}");
        }),
    )
}

fn read_crate_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {name}: {error}");
        }),
    )
}

#[test]
fn cloud_package_status_uses_package_manifest_for_selected_project_scope() {
    let cloud = read_crate_file("src/app/view_model/cloud.rs");
    let cloud_page = read_ui_file("cloud.slint");
    let localization = read_crate_file("src/app/localization.rs");
    for snippet in [
        "const PACKAGE_MANIFEST_FILE: &str = \"zircon-package.toml\";",
        "const PACKAGE_SOURCE_PROJECT_KEY: &str = \"source_project\";",
        "selected_project_package_count(package_root, path)",
        "selected_project_install_count(install_root, selected_project)",
        "package_matches_selected_project",
        "manifest.parse::<toml::Value>()",
        "use crate::projects::project_paths_match;",
        "project_paths_match(Path::new(source_project), selected_project_path)",
        "{package_count} local {noun} for selected project",
        "{install_count} local {noun} for selected project",
        "No local install for selected project yet",
    ] {
        assert!(
            cloud.contains(snippet),
            "Cloud package/install status must scope selected-project local outputs through zircon-package.toml source_project; missing {snippet}"
        );
    }
    for forbidden in [
        "fn paths_match_for_summary(",
        "fn normalized_path_key(",
        "left.canonicalize(), right.canonicalize()",
    ] {
        assert!(
            !cloud.contains(forbidden),
            "Cloud selected-project package/install matching must reuse project_paths_match instead of a local duplicate path matcher; found {forbidden}"
        );
    }
    for snippet in [
        "HubPage::Cloud => text(language, \"Packages\", \"本地包\"),",
        "cloud_overview: text(language, \"Local Package Overview\", \"本地包状态概览\"),",
        "Local Packages - Selected Project",
        "Local packages, installs, output status, and reserved service slots.",
        "local_mode_status: localization::text(language, \"Local only\", \"仅本地\"),",
        "Profile Sync Slot",
        "Remote Build Slot",
        "Artifact Upload Slot",
        "cloud_local_mode: text(language, \"Local Mode\", \"本地模式\"),",
        "label: root.ui-text.cloud-local-mode;",
        "primary: root.summary.local-mode-status;",
    ] {
        assert!(
            cloud.contains(snippet) || cloud_page.contains(snippet) || localization.contains(snippet),
            "Cloud page copy must stay local/offline rather than presenting a real cloud account surface; missing {snippet}"
        );
    }
    for alias in [
        "account_status: localization::text(language, \"Local only\", \"仅本地\"),",
        "cloud_account: text(language, \"Local Mode\", \"本地模式\"),",
    ] {
        assert!(
            cloud.contains(alias) || localization.contains(alias),
            "Cloud retains additive legacy local-mode aliases for generated binding compatibility; missing {alias}"
        );
    }
    for forbidden in [
        "label: root.ui-text.cloud-account;",
        "primary: root.summary.account-status;",
    ] {
        assert!(
            !cloud_page.contains(forbidden),
            "CloudPage visible metric bindings should use local-mode field names instead of account aliases: {forbidden}"
        );
    }
    for forbidden in [
        "HubPage::Cloud => text(language, \"Cloud\", \"云服务\"),",
        "account_status: localization::text(language, \"Not connected\"",
        "Account Sync",
        "Reserved for sign-in, license",
        "cloud_account: text(language, \"Account\"",
        "Local Cloud Overview",
        "Local Cloud - Selected Project",
        "Local cloud readiness",
        "本地云服务",
    ] {
        assert!(
            !cloud.contains(forbidden) && !localization.contains(forbidden),
            "Cloud page copy must not return to account-service wording: {forbidden}"
        );
    }
}

#[test]
fn dashboard_project_card_labels_are_view_model_data() {
    let shared = read_ui_file("shared.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let project_view_model = read_crate_file("src/app/view_model/projects.rs");

    let project_card_data = shared
        .split("export struct ProjectCardData")
        .nth(1)
        .and_then(|source| source.split("export struct RecentProjectRowData").next())
        .expect("shared.slint must declare ProjectCardData before RecentProjectRowData");
    for snippet in [
        "modified-label: string,",
        "pinned-label: string,",
        "missing-label: string,",
    ] {
        assert!(
            project_card_data.contains(snippet),
            "ProjectCardData must carry card-visible labels from the view model; missing {snippet}"
        );
    }

    let project_card = dashboard
        .split("component ProjectCard")
        .nth(1)
        .and_then(|source| source.split("component ProjectFlow").next())
        .expect("project_dashboard.slint must declare ProjectCard before ProjectFlow");
    for snippet in [
        "text: root.project.modified-label;",
        "text: root.project.pinned-label;",
        "text: root.project.missing-label;",
    ] {
        assert!(
            project_card.contains(snippet),
            "ProjectCard must render localized card labels from ProjectCardData; missing {snippet}"
        );
    }
    for forbidden in [
        "root.ui-text.modified",
        "root.ui-text.pinned",
        "root.ui-text.missing",
        "\"Modified \" +",
        "\"Pinned\"",
        "\"Missing\"",
    ] {
        assert!(
            !project_card.contains(forbidden),
            "ProjectCard should not rebuild localized labels inside repeated UI items: {forbidden}"
        );
    }

    for snippet in [
        "modified_label: shared(project_card_modified_label(&modified, language)),",
        "pinned_label: localization::text(language, \"Pinned\", \"置顶\"),",
        "missing_label: localization::text(language, \"Missing\", \"缺失\"),",
        "fn project_card_modified_label(modified: &str, language: HubLanguage) -> String",
    ] {
        assert!(
            project_view_model.contains(snippet),
            "Project card label localization must stay in the Rust view model; missing {snippet}"
        );
    }
}
