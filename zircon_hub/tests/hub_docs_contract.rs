//! Static contracts for Zircon Hub documentation ownership and handoff.
//! 把 Hub 文档的实现归属、测试矩阵和验收交接作为维护契约，防止实现迁移后文档失去可追溯入口。

use std::{fs, path::PathBuf};

fn repo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_hub should have a repository parent")
        .to_path_buf()
}

/// 读取仓库级交接文档或工具证据；约定 Hub 包位于仓库根下一层，不能依赖测试启动时的工作目录。
fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_dir().join(path))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

/// 本轮文档交接契约的固定拥有者集合；新增文档是否进入门槛需要连同实现归属重新审查。
const HUB_DOCS: &[&str] = &[
    "docs/zircon_hub/index.md",
    "docs/zircon_hub/ui/responsive-component-system.md",
    "docs/zircon_hub/state/foundations.md",
    "docs/zircon_hub/projects/lifecycle-workflows.md",
    "docs/zircon_hub/pages/actionable-pages.md",
    "docs/zircon_hub/pages/settings-status.md",
];

/// 将固定文档集合的机器可读归属头与刷新来源作为交接门槛；标题词存在不等于文档语义已经核验。
#[test]
fn hub_docs_keep_machine_readable_headers_and_refresh_sources() {
    for path in HUB_DOCS {
        let doc = read_repo_file(path);
        assert!(
            doc.starts_with("---\nrelated_code:\n"),
            "{path} must start with the machine-readable related_code header"
        );
        for snippet in [
            "\nimplementation_files:\n",
            "\nplan_sources:\n",
            "\ntests:\n",
            "\ndoc_type:",
            "hub-docs-contract-refresh/plan.md",
            "hub-docs-contract-refresh/review-surface.md",
            "zircon_hub/tests/hub_docs_contract.rs",
        ] {
            assert!(
                doc.contains(snippet),
                "{path} must record Hub docs refresh ownership; missing {snippet}"
            );
        }
    }
}

/// 检查状态、项目、页面及设置文档的当前职责和验收入口，并阻止已移除的拥有者继续出现在流程说明中。
#[test]
fn hub_docs_record_current_contract_matrix_and_acceptance_handoff() {
    let index = read_repo_file("docs/zircon_hub/index.md");
    for snippet in [
        "## Docs And Contract Refresh",
        "component ownership model",
        "runtime-state ownership map",
        "visual-standard handoff",
        "hub-acceptance-validation",
    ] {
        assert!(
            index.contains(snippet),
            "Hub index must summarize the docs/contract refresh handoff; missing {snippet}"
        );
    }

    let responsive = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");
    for snippet in [
        "## Docs/Contracts Refresh Gate",
        "tokens -> layout primitives -> surfaces -> inputs/navigation/data display/overlays",
        "Slint 1.16.1 constraints",
        "fallback Workspace header button routes to Settings instead of saving without a draft",
        "HubCheckbox` and `HubSwitch` treat a missing `onChange` callback as a read-only contract",
        "SourceEngineList` treats a missing `onSelect` callback as read-only",
        "HubList` derives row disabled state from row DTOs plus `onSelect` presence",
        "ui_global_rules_contract.rs",
        "ui_visual_standard_contract.rs",
        "hub_docs_contract.rs",
    ] {
        assert!(
            responsive.contains(snippet),
            "Responsive component docs must record the current contract matrix; missing {snippet}"
        );
    }

    let state = read_repo_file("docs/zircon_hub/state/foundations.md");
    for snippet in [
        "## Runtime-State Docs Refresh Handoff",
        "hub-runtime-state-integration-contract-docs/review-surface.md",
        "HubConfig.runtime",
        "HubSnapshot::scope()",
        "hub_docs_contract.rs",
    ] {
        assert!(
            state.contains(snippet),
            "State foundations docs must preserve runtime-state handoff evidence; missing {snippet}"
        );
    }

    let lifecycle = read_repo_file("docs/zircon_hub/projects/lifecycle-workflows.md");
    for snippet in [
        "## Docs Refresh Handoff",
        "React/MUI project lifecycle surface",
        "CreateProjectActionPayload",
        "create_project_from_payload",
        "web/src/pages/ProjectsDashboard.tsx",
        "projectTargetPayload",
        "request-delete",
        "confirm-delete",
        "Recycle Bin",
        "Dashboard-style Open Editor can fall back to the latest recent project only when no selected project exists",
        "HubConfig.runtime",
        "ui_project_navigation_contract.rs",
    ] {
        assert!(
            lifecycle.contains(snippet),
            "Project lifecycle docs must state current project workflow contracts; missing {snippet}"
        );
    }
    for obsolete in [
        "Slint page composition",
        "Slint form state",
        "project_dashboard.slint",
        "project_new_page.slint",
        "project_detail_page.slint",
        "view_model::projects",
        "project_workspace.rs",
        "zircon_hub/src/app/",
    ] {
        assert!(
            !lifecycle.contains(obsolete),
            "Project lifecycle docs must not reference obsolete Slint/app lifecycle ownership; found {obsolete}"
        );
    }

    let pages = read_repo_file("docs/zircon_hub/pages/actionable-pages.md");
    for snippet in [
        "## Docs Refresh Handoff",
        "React/MUI actionable page surface",
        "src/tauri_app/runtime_state/build_actions.rs",
        "web/src/pages/BuildsPage.tsx",
        "projectTargetPayload",
        "detailRows",
        "comingSoon",
        "settingsDraft",
        "hub_docs_contract.rs",
    ] {
        assert!(
            pages.contains(snippet),
            "Actionable pages docs must state page-scope and timeline contracts; missing {snippet}"
        );
    }
    for obsolete in [
        "WorkspaceActionReadinessData",
        "OperationTimelinePanel",
        "SettingStatusData",
        "HubWindow.workspace-action-readiness",
        "view_model/workspace_actions.rs",
        "builds.slint",
        "cloud.slint",
        "settings.slint",
        "zircon_hub/src/app/",
    ] {
        assert!(
            !pages.contains(obsolete),
            "Actionable pages docs must not reference obsolete Slint/app page ownership; found {obsolete}"
        );
    }

    let settings = read_repo_file("docs/zircon_hub/pages/settings-status.md");
    for snippet in [
        "## Docs Refresh Handoff",
        "HubSettingsSummary",
        "settingsDraft",
        "browse-settings-folder",
        "save-settings",
        "save_settings_refreshes_source_scoped_catalogs_in_returned_view_model",
        "keeps_first_source_engine_root_before_fallback_limit",
        "hub_docs_contract.rs",
    ] {
        assert!(
            settings.contains(snippet),
            "Settings status docs must state React/Tauri settings draft ownership; missing {snippet}"
        );
    }
    for obsolete in [
        "SettingStatusData",
        "settings.slint",
        "settings_page_components.slint",
        "view_model::settings_statuses",
        "browse-project-location",
        "browse-output",
        "browse-device-install",
        "zircon_hub/src/app/",
    ] {
        assert!(
            !settings.contains(obsolete),
            "Settings status docs must not reference obsolete Slint/app status ownership; found {obsolete}"
        );
    }
}
