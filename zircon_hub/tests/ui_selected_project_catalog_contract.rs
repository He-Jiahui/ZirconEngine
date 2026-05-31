//! Static contracts for selected-project catalog/page scope copy.
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

fn read_repo_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join("..").join(name)).unwrap_or_else(|error| {
            panic!("failed to read repo file {name}: {error}");
        }),
    )
}

fn assert_catalog_page_inherits_geometry(page: &str, component: &str, source: &str) {
    assert!(
        source.contains(&format!(
            "export component {component} inherits CatalogPage"
        )),
        "{page} must expose its page root by inheriting the shared CatalogPage"
    );
    for forbidden in [
        "export component AssetsPage inherits Rectangle",
        "export component PluginsPage inherits Rectangle",
        "export component LearnPage inherits Rectangle",
        "    CatalogPage {",
        "list-scroll-y <=> root.list-scroll-y",
        "width: root.width;",
        "height: root.height;",
    ] {
        assert!(
            !source.contains(forbidden),
            "{page} should let CatalogPage own page geometry and internal list scroll state instead of using {forbidden}"
        );
    }
}

#[test]
fn page_family_docs_record_contract_gate_and_acceptance_scope() {
    let docs = read_repo_file("docs/zircon_hub/pages/actionable-pages.md");

    for snippet in [
        "hub-pages-contracts-docs/plan.md",
        "hub-pages-contracts-docs/decomposition.md",
        "## Contracts and validation evidence",
        "The `hub-pages-contracts-docs` gate is intentionally not another feature implementation branch.",
        "**Builds and Editor:**",
        "**Assets, Plugins, and Learn:**",
        "**Team and Cloud:**",
        "**Settings:**",
        "**Cross-page foundations:**",
        "This gate did not identify a missing page feature requiring a new milestone split.",
        "cargo test -p zircon_hub --test ui_selected_project_runtime_contract --locked -- --nocapture",
        "cargo test -p zircon_hub --test ui_selected_project_catalog_contract --locked -- --nocapture",
        "cargo test -p zircon_hub --test ui_page_surface_coverage_contract --locked -- --nocapture",
        "cargo test -p zircon_hub --test ui_inputs_contract --locked -- --nocapture",
        "cargo build -p zircon_hub --bin zircon_hub --locked",
        "cargo check -p zircon_hub --locked",
    ] {
        assert!(
            docs.contains(snippet),
            "Hub actionable-pages docs must record the page-family contract gate and acceptance scope; missing {snippet}"
        );
    }
}

#[test]
fn selected_project_catalog_pages_surface_scope_copy() {
    let shared = read_ui_file("shared.slint");
    assert!(
        shared.contains("scope: string,"),
        "PluginData must expose project/engine scope for Hub workflow grouping"
    );
    for snippet in [
        "asset-catalog-selected: string,",
        "asset-empty-selected-detail: string,",
        "asset-empty-global-detail: string,",
        "plugin-catalog-selected: string,",
        "plugin-empty-selected-detail: string,",
        "plugin-empty-global-detail: string,",
        "learn-library-selected: string,",
        "learn-empty-selected-detail: string,",
        "learn-empty-global-detail: string,",
        "team-workspace-selected: string,",
        "team-empty-selected-detail: string,",
        "team-empty-global-detail: string,",
        "cloud-overview-selected: string,",
        "cloud-local-selected-detail: string,",
        "cloud-local-mode: string,",
        "install-to-device: string,",
        "select-project-before-building: string,",
        "select-project-before-opening: string,",
        "select-project-before-packaging: string,",
        "select-project-before-installing: string,",
        "current-project: string,",
        "active-source-engine: string,",
        "no-project-selected: string,",
    ] {
        assert!(
            shared.contains(snippet),
            "UiTextData must expose selected-project-aware Assets/Plugins/Team/Cloud and Builds action copy; missing {snippet}"
        );
    }
    let localization = read_crate_file("src/app/localization.rs");
    assert!(
        localization.contains("cloud_local_mode: text(language,"),
        "Rust localization must initialize UiTextData.cloud-local-mode when shared.slint exposes the field"
    );

    let assets_page = read_ui_file("assets.slint");
    assert_catalog_page_inherits_geometry("assets.slint", "AssetsPage", &assets_page);
    for snippet in [
        "in property <bool> has-selected-project: false;",
        "root.has-selected-project ? root.ui-text.asset-catalog-selected : root.ui-text.asset-catalog",
        "root.has-selected-project ? root.ui-text.asset-empty-selected-detail : root.ui-text.asset-empty-global-detail",
        "empty-detail: root.assets-empty-detail;",
    ] {
        assert!(
            assets_page.contains(snippet),
            "AssetsPage must explain whether the list is scoped to the selected project or global local roots; missing {snippet}"
        );
    }

    let asset_projection = read_crate_file("src/app/view_model/assets.rs");
    for snippet in [
        "asset_source_priority(&left.source)",
        "SELECTED_PROJECT_ASSET_SOURCE => 0",
        "PROJECT_ASSET_SOURCE => 1",
        "Source Engine / {source}",
    ] {
        assert!(
            asset_projection.contains(snippet),
            "AssetData projection must prioritize selected-project assets and keep source-engine group labels; missing {snippet}"
        );
    }

    let asset_catalog = read_crate_file("src/assets/catalog.rs");
    for snippet in [
        "source_priority(&left.source)",
        "SELECTED_PROJECT_ASSET_SOURCE => 0",
        "PROJECT_ASSET_SOURCE => 1",
    ] {
        assert!(
            asset_catalog.contains(snippet),
            "Asset catalog discovery must emit selected-project assets before project/engine groups; missing {snippet}"
        );
    }
    let scoped_views = read_crate_file("src/app/runtime/source_scoped_views.rs");
    for snippet in [
        "pub(super) fn selected_project_catalog_root(&self)",
        ".selected_project()",
        "pub(super) fn source_engine_catalog_roots(&self)",
        "scope.source_engine.engine_id()",
        ".find(|engine| engine.id == engine_id)",
        "push_development_roots(&mut roots, engine.source_dir.clone());",
        "HubScope::resolve(",
        "source_engine_catalog_roots_do_not_fallback_for_unbound_selected_project",
    ] {
        assert!(
            scoped_views.contains(snippet),
            "Runtime scoped scanners must derive selected-project and Source Engine roots from HubScope; missing {snippet}"
        );
    }

    let plugins = read_ui_file("plugins.slint");
    let catalog_components = read_ui_file("catalog_page_components.slint");
    let plugin_surface = format!("{plugins}\n{catalog_components}");
    assert_catalog_page_inherits_geometry("plugins.slint", "PluginsPage", &plugins);
    assert!(
        plugin_surface.contains("root.plugin.scope +"),
        "PluginsPage rows must display whether a plugin came from the selected project or engine"
    );
    for snippet in [
        "in property <bool> has-selected-project: false;",
        "root.has-selected-project ? root.ui-text.plugin-catalog-selected : root.ui-text.plugin-catalog",
        "root.has-selected-project ? root.ui-text.plugin-empty-selected-detail : root.ui-text.plugin-empty-global-detail",
        "empty-detail: root.plugins-empty-detail;",
    ] {
        assert!(
            plugins.contains(snippet),
            "PluginsPage must explain whether discovery includes selected-project plugin manifests; missing {snippet}"
        );
    }

    let plugin_projection = read_crate_file("src/app/view_model/plugins.rs");
    for snippet in [
        "plugin_scope_priority(&left.scope)",
        "PROJECT_PLUGIN_SCOPE => 0",
        "ENGINE_PLUGIN_SCOPE => 1",
        "PROJECT_PLUGIN_SCOPE =>",
        "ENGINE_PLUGIN_SCOPE =>",
        "\"Selected Project\"",
        "\"Source Engine\"",
    ] {
        assert!(
            plugin_projection.contains(snippet),
            "PluginData projection must prioritize and label selected-project and Source Engine plugin scopes; missing {snippet}"
        );
    }

    let learn = read_ui_file("learn.slint");
    let learn_surface = format!("{learn}\n{catalog_components}");
    assert_catalog_page_inherits_geometry("learn.slint", "LearnPage", &learn);
    for snippet in [
        "source: string,",
        "in property <bool> has-selected-project: false;",
        "root.has-selected-project ? root.ui-text.learn-library-selected : root.ui-text.learn-library",
        "root.has-selected-project ? root.ui-text.learn-empty-selected-detail : root.ui-text.learn-empty-global-detail",
        "empty-detail: root.resources-empty-detail;",
        "root.resource.source +",
    ] {
        assert!(
            shared.contains(snippet) || learn_surface.contains(snippet),
            "LearnPage must explain whether documentation includes selected-project docs or Source Engine docs; missing {snippet}"
        );
    }
    let learn_runtime = read_crate_file("src/app/runtime/learn_catalog.rs");
    for snippet in [
        "discover_learn_catalog_for_scope(",
        "self.selected_project_catalog_root()",
        "self.source_engine_catalog_roots()",
        "fn learn_resource_path_for_open(&self, resource_path: &str) -> Result<PathBuf, HubError>",
        "project_filesystem_path_key(&requested_path)",
        "Learn resource is not in the current catalog",
        "Learn resource is no longer available",
    ] {
        assert!(
            learn_runtime.contains(snippet),
            "Learn runtime refresh/open actions must use selected-project scope and only open current catalog files; missing {snippet}"
        );
    }
    let runtime_root = read_crate_file("src/app/runtime.rs");
    assert!(
        runtime_root.contains("ui.on_open_learn_resource")
            && runtime_root.contains("runtime.open_learn_resource(&path)"),
        "HubWindow must route Learn row open callbacks into the runtime open validator"
    );
    let learn_catalog = read_crate_file("src/learn/catalog.rs");
    for snippet in [
        "pub const SELECTED_PROJECT_LEARN_SOURCE: &str = \"Selected Project\";",
        "pub const SOURCE_ENGINE_LEARN_SOURCE: &str = \"Source Engine\";",
        "pub fn discover_learn_catalog_for_scope",
        "selected_project_root: Option<PathBuf>",
        "collect_docs_root(",
        "project_filesystem_path_key(&docs_root)",
        "source_priority(&left.source)",
        "SELECTED_PROJECT_LEARN_SOURCE => 0",
        "SOURCE_ENGINE_LEARN_SOURCE => 1",
        "source: source.to_string(),",
    ] {
        assert!(
            learn_catalog.contains(snippet),
            "Learn catalog discovery must scan selected-project docs before source docs and dedupe roots through the shared filesystem key; missing {snippet}"
        );
    }
    let learn_projection = read_crate_file("src/app/view_model/learn.rs");
    for snippet in [
        "learn_source_priority(&left.source)",
        "SELECTED_PROJECT_LEARN_SOURCE => 0",
        "SOURCE_ENGINE_LEARN_SOURCE => 1",
        "\"Selected Project\"",
        "\"Source Engine\"",
        "learn_items_orders_selected_project_docs_before_engine_docs",
    ] {
        assert!(
            learn_projection.contains(snippet),
            "Learn view-model projection must prioritize and label selected-project documentation; missing {snippet}"
        );
    }

    let team = read_ui_file("team.slint");
    for snippet in [
        "in property <bool> has-selected-project: false;",
        "root.has-selected-project ? root.ui-text.team-workspace-selected : root.ui-text.team-workspace",
        "root.has-selected-project ? root.ui-text.team-empty-selected-detail : root.ui-text.team-empty-global-detail",
        "title: root.workspace-title;",
        "detail: root.members-empty-detail;",
    ] {
        assert!(
            team.contains(snippet),
            "TeamPage must explain whether local Git data is scoped to the selected project or Source Engine fallback; missing {snippet}"
        );
    }
    let team_projection = read_crate_file("src/app/view_model/team.rs");
    for snippet in [
        "paths_share_repository_scope(project_path, &team.repository_path)",
        "project_path.canonicalize()",
        "project_metadata_key(project_path)",
        "project_metadata_key(repository_path)",
        "project_key.starts_with(&format!(\"{repository_key}/\"))",
        "\"Selected project repository\"",
        "\"Selected project repository unavailable; showing Source Engine repository\"",
        "\"Source Engine repository\"",
        "fn team_summary_labels_source_engine_fallback_for_missing_selected_project_repository()",
    ] {
        assert!(
            team_projection.contains(snippet),
            "TeamData projection must use normalized selected-project repository matching before falling back to Source Engine labels; missing {snippet}"
        );
    }

    let cloud_page = read_ui_file("cloud.slint");
    let cloud_components = read_ui_file("cloud_page_components.slint");
    for snippet in [
        "in property <bool> has-selected-project: false;",
        "callback package-project();",
        "callback install-device();",
        "root.has-selected-project ? root.ui-text.cloud-overview-selected : root.ui-text.cloud-overview",
        "root.has-selected-project ? root.ui-text.cloud-local-selected-detail : root.ui-text.cloud-local-only",
        "title: root.overview-title;",
        "subtitle: root.overview-detail;",
        "action-id: \"package-selected-project\";",
        "action-title: root.ui-text.cloud-package-action;",
        "action-detail: root.summary.package-action-detail;",
        "action-enabled: root.summary.package-action-enabled;",
        "root.package-project();",
        "action-id: \"install-selected-project\";",
        "action-title: root.ui-text.cloud-install-action;",
        "action-detail: root.summary.install-action-detail;",
        "action-enabled: root.summary.install-action-enabled;",
        "root.install-device();",
    ] {
        assert!(
            cloud_page.contains(snippet) || cloud_components.contains(snippet),
            "CloudPage must make local package/install/output status read as selected-project scoped when a project is selected; missing {snippet}"
        );
    }

    let app = read_ui_file("app.slint");
    for snippet in [
        "package-project => { root.package-selected-project(); }",
        "install-device => { root.install-selected-project(); }",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must route Cloud package/install actions through selected-project-only callbacks; missing {snippet}"
        );
    }

    let cloud_projection = read_crate_file("src/app/view_model/cloud.rs");
    for snippet in [
        "let package_action = selected_project_action_readiness(",
        "let install_action = selected_project_action_readiness(",
        "package_action_detail: package_action.detail,",
        "package_action_enabled: package_action.enabled,",
        "install_action_detail: install_action.detail,",
        "install_action_enabled: install_action.enabled,",
        "fn selected_project_action_readiness(",
        "let scope = snapshot.scope();",
        "Select a project before packaging",
        "Select a project before installing",
        "Configure package output root before packaging",
        "Configure device install directory before installing",
        "Selected project is no longer in the recent-project registry",
    ] {
        assert!(
            cloud_projection.contains(snippet),
            "CloudSummaryData projection must expose selected-project package/install action readiness; missing {snippet}"
        );
    }
}
