//! Static contracts for Zircon Hub documentation ownership and handoff.

use std::{fs, path::PathBuf};

fn repo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_hub should have a repository parent")
        .to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_dir().join(path))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

const HUB_DOCS: &[&str] = &[
    "docs/zircon_hub/index.md",
    "docs/zircon_hub/ui/responsive-component-system.md",
    "docs/zircon_hub/state/foundations.md",
    "docs/zircon_hub/projects/lifecycle-workflows.md",
    "docs/zircon_hub/pages/actionable-pages.md",
    "docs/zircon_hub/pages/settings-status.md",
];

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
        "Dashboard-style Open Editor can fall back to the latest recent project only when no selected project exists",
        "HubConfig.runtime",
        "ui_project_navigation_contract.rs",
    ] {
        assert!(
            lifecycle.contains(snippet),
            "Project lifecycle docs must state current project workflow contracts; missing {snippet}"
        );
    }

    let pages = read_repo_file("docs/zircon_hub/pages/actionable-pages.md");
    for snippet in [
        "## Docs Refresh Handoff",
        "WorkspaceActionReadinessData",
        "OperationTimelinePanel",
        "selected-project-only",
        "hub_docs_contract.rs",
    ] {
        assert!(
            pages.contains(snippet),
            "Actionable pages docs must state page-scope and timeline contracts; missing {snippet}"
        );
    }

    let settings = read_repo_file("docs/zircon_hub/pages/settings-status.md");
    for snippet in [
        "## Docs Refresh Handoff",
        "HubSnapshot",
        "SettingStatusData",
        "save-settings",
        "hub_docs_contract.rs",
    ] {
        assert!(
            settings.contains(snippet),
            "Settings status docs must state snapshot-derived status ownership; missing {snippet}"
        );
    }
}
