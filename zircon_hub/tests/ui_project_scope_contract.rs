//! Static contracts for passive selected-project scope projection.
//!
//! This milestone owns Slint DTO shape and view-model copy only. Catalog refresh,
//! Cloud/package/install status, and command behavior are intentionally outside
//! this file.

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
fn shared_scope_dtos_are_available_to_pages() {
    let shared = read_ui_file("shared.slint");
    for snippet in [
        "export struct ProjectDetailData",
        "selected: bool,",
        "missing: bool,",
        "can-open: bool,",
        "can-build: bool,",
        "export struct QuickActionData",
        "detail: string,",
        "enabled: bool,",
        "export struct SourceEngineData",
        "status: string,",
        "export struct WorkspaceActionReadinessData",
        "selected-project-status: string,",
        "source-engine-status: string,",
        "build-disabled-reason: string,",
    ] {
        assert!(
            shared.contains(snippet),
            "shared.slint must expose passive scope DTO field {snippet}"
        );
    }
}

#[test]
fn app_forwards_selected_project_and_source_engine_scope_copy() {
    let app = read_ui_file("app.slint");
    for snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "readiness: root.workspace-action-readiness;",
        "quick-actions: root.quick-actions;",
        "has-selected-project: root.project-detail.selected;",
    ] {
        assert!(
            app.contains(snippet),
            "app.slint must forward Rust-projected scope DTOs without recomputing them; missing {snippet}"
        );
    }
}

#[test]
fn project_card_and_detail_labels_are_view_model_data() {
    let shared = read_ui_file("shared.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
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

    let project_card = dashboard_components
        .split("export component ProjectCard")
        .nth(1)
        .and_then(|source| source.split("export component ProjectFlow").next())
        .expect("project_dashboard_components.slint must export ProjectCard before ProjectFlow");
    for snippet in [
        "text: root.visible-modified-label;",
        "text: root.visible-platform;",
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
        "fn project_detail_status_label(",
        "fn empty_project_detail(language: HubLanguage) -> ProjectDetailData",
    ] {
        assert!(
            project_view_model.contains(snippet),
            "Project view-model copy must own visible project labels and detail state; missing {snippet}"
        );
    }
}
