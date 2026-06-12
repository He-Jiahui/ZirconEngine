//! Static contracts for React/MUI Hub page chrome and routed page surfaces.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub crate should live under the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read Hub crate file {path}: {error}")),
    )
}

fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read repository file {path}: {error}")),
    )
}

fn assert_contains_all(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_name} should contain shell-page snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete shell-page snippet {snippet:?}"
        );
    }
}

#[test]
fn hub_window_owns_page_router_and_main_surface_slot() {
    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");
    let shell_index = read_crate_file("web/src/components/shell/index.ts");

    assert_contains_all(
        "HubWindow.tsx",
        &hub_window,
        &[
            "export interface HubWindowProps",
            "state: HubShellState;",
            "onAction: HubActionHandler;",
            "export function HubWindow({ state, onAction }: HubWindowProps)",
            "width: \"100vw\"",
            "height: \"100vh\"",
            "<TopBar state={state} onAction={onAction} />",
            "<NavigationDrawer",
            "activePage={state.activePage}",
            "text={state.ui.shell}",
            "engineVersion={state.engineVersion}",
            "sourceEngines={state.sourceEngines}",
            "activeSourceEngineId={state.activeSourceEngineId}",
            "onAction={onAction}",
            "component=\"main\"",
            "overflow: \"hidden\"",
            "const pageRoutes: Record<HubPageId, HubPageComponent> = {",
            "projects: ProjectsDashboard,",
            "editor: EditorPage,",
            "builds: BuildsPage,",
            "cloud: CloudPage,",
            "assets: CatalogPage,",
            "plugins: CatalogPage,",
            "learn: CatalogPage,",
            "team: TeamPage,",
            "settings: SettingsPage,",
            "const activeRoute = toHubPageId(state.activePage);",
            "const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;",
            "<PageComponent state={state} onAction={onAction} />",
        ],
    );
    assert_contains_all(
        "components/shell/index.ts",
        &shell_index,
        &[
            "export * from \"./HubWindow\";",
            "export * from \"./NavigationDrawer\";",
            "export * from \"./TopBar\";",
        ],
    );
}

#[test]
fn rust_navigation_ids_feed_localized_page_title_subtitle_projection() {
    let navigation = read_crate_file("src/state/navigation.rs");
    let localized = read_crate_file("src/tauri_app/view_model/localized.rs");
    let view_model = read_crate_file("src/tauri_app/view_model.rs");

    assert_contains_all(
        "navigation.rs",
        &navigation,
        &[
            "pub enum HubPage",
            "Projects",
            "Editor",
            "Assets",
            "Builds",
            "Plugins",
            "Cloud",
            "Team",
            "Learn",
            "Settings",
            "pub fn id(self) -> &'static str",
            "pub fn from_id(id: &str) -> Option<Self>",
            "hub_page_parses_known_navigation_ids",
        ],
    );
    assert_not_contains_any(
        "navigation.rs",
        &navigation,
        &["pub fn title(self)", "pub fn subtitle(self)"],
    );
    assert_contains_all(
        "localized.rs",
        &localized,
        &[
            "pub(crate) fn page_title(self, page: HubPage) -> &'static str",
            "HubLanguage::English => match page",
            "HubPage::Projects => \"Projects\"",
            "HubPage::Cloud => \"Local Delivery\"",
            "HubLanguage::Chinese => match page",
            "HubPage::Projects => \"项目\"",
            "pub(crate) fn page_subtitle(self, page: HubPage) -> &'static str",
            "HubPage::Projects => \"Manage your projects and start building worlds.\"",
            "HubPage::Settings => \"Configure toolchains, source paths, and defaults.\"",
            "HubPage::Projects => \"管理本地项目并启动世界构建流程。\"",
            "HubPage::Settings => \"配置工具链、源码路径、构建默认值和语言。\"",
        ],
    );
    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "pub page_title: String",
            "pub page_subtitle: String",
            "page_title: text.page_title(snapshot.selected_page).to_string()",
            "page_subtitle: text.page_subtitle(snapshot.selected_page).to_string()",
            "active_page: snapshot.selected_page.id().to_string()",
        ],
    );
}

#[test]
fn routed_pages_render_page_title_subtitle_and_status_surfaces() {
    for (page, snippets) in [
        (
            "ProjectsDashboard.tsx",
            vec![
                "<Typography variant=\"h4\">{text.title}</Typography>",
                "{state.pageSubtitle}",
                "ProjectBrowserPage state={state} onAction={onAction}",
                "ProjectDetailPage state={state} onAction={onAction}",
            ],
        ),
        (
            "ProjectBrowserPage.tsx",
            vec![
                "<Typography variant=\"h4\">{text.browserTitle}</Typography>",
                "{state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
        (
            "ProjectDetailPage.tsx",
            vec![
                "<Typography variant=\"h4\">{project?.name ?? text.detailTitle}</Typography>",
                "{project?.path ?? state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
        (
            "EditorPage.tsx",
            vec![
                "<Typography variant=\"h4\">{state.pageTitle}</Typography>",
                "{state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
        (
            "BuildsPage.tsx",
            vec![
                "<Typography variant=\"h4\">{state.pageTitle}</Typography>",
                "{state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
        (
            "CatalogPage.tsx",
            vec![
                "<Typography variant=\"h4\">{state.pageTitle}</Typography>",
                "{state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
        (
            "CloudPage.tsx",
            vec![
                "<Typography variant=\"h4\">{state.pageTitle}</Typography>",
                "{state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
        (
            "TeamPage.tsx",
            vec![
                "<Typography variant=\"h4\">{state.pageTitle}</Typography>",
                "{state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
        (
            "SettingsPage.tsx",
            vec![
                "<Typography variant=\"h4\">{settingsText.heading}</Typography>",
                "{state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
        (
            "WorkspacePage.tsx",
            vec![
                "<Typography variant=\"h4\">{state.pageTitle}</Typography>",
                "{state.pageSubtitle}",
                "<HubStatusBanner task={state.taskSummary} />",
            ],
        ),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(page, &source, &snippets);
        assert_not_contains_any(
            page,
            &source,
            &["position: \"absolute\"", "<Snackbar", "<Alert severity"],
        );
    }
}

#[test]
fn feedback_components_own_status_banner_and_snackbar_chrome() {
    let status_banner = read_crate_file("web/src/components/feedback/HubStatusBanner.tsx");
    let snackbar = read_crate_file("web/src/components/feedback/HubSnackbar.tsx");
    let feedback_index = read_crate_file("web/src/components/feedback/index.ts");
    let app = read_crate_file("web/src/App.tsx");

    assert_contains_all(
        "HubStatusBanner.tsx",
        &status_banner,
        &[
            "import { Alert, Box, LinearProgress, Typography } from \"@mui/material\";",
            "export interface HubStatusBannerProps",
            "task: HubTaskSummary;",
            "export function HubStatusBanner({ task }: HubStatusBannerProps)",
            "const severity = task.tone === \"neutral\" || task.tone === \"running\" ? \"info\" : task.tone;",
            "const shouldShowProgress = task.running || task.progressPercent > 0;",
            "severity={severity}",
            "variant=\"outlined\"",
            "<Typography variant=\"subtitle2\">{task.label}</Typography>",
            "<Typography variant=\"body2\">{task.detail}</Typography>",
            "<Typography variant=\"caption\" color=\"text.secondary\">",
            "{task.operation}",
            "variant=\"determinate\"",
            "value={task.progressPercent}",
            "{task.recovery ? (",
            "{task.recovery}",
        ],
    );
    assert_contains_all(
        "HubSnackbar.tsx",
        &snackbar,
        &[
            "import { Alert, Box, Snackbar, Typography } from \"@mui/material\";",
            "export interface HubSnackbarProps",
            "open: boolean;",
            "onClose: () => void;",
            "const severity = task.tone === \"neutral\" || task.tone === \"running\" ? \"info\" : task.tone;",
            "<Snackbar open={open} autoHideDuration={4200} onClose={onClose} anchorOrigin={{ vertical: \"bottom\", horizontal: \"right\" }}>",
            "<Alert severity={severity} variant=\"filled\" onClose={onClose} sx={{ maxWidth: 520 }}>",
            "<Typography variant=\"subtitle2\">{task.label}</Typography>",
            "<Typography variant=\"body2\">{task.detail}</Typography>",
            "{task.recovery ? (",
            "{task.recovery}",
        ],
    );
    assert_contains_all(
        "components/feedback/index.ts",
        &feedback_index,
        &[
            "export * from \"./HubSnackbar\";",
            "export * from \"./HubStatusBanner\";",
        ],
    );
    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "if (state.taskSummary.running || state.taskSummary.tone !== \"neutral\" || state.taskSummary.recovery)",
            "setSnackbarOpen(true);",
            "<HubSnackbar task={state.taskSummary} open={snackbarOpen} onClose={() => setSnackbarOpen(false)} />",
        ],
    );
}

#[test]
fn shell_page_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_shell_page_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_page_contract",
            "## Shell Page Contract Cutover",
            "React/MUI page chrome and routed page surfaces",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/components/feedback/HubStatusBanner.tsx",
            "web/src/components/feedback/HubSnackbar.tsx",
            "src/state/navigation.rs",
            "src/tauri_app/view_model.rs",
            "web/src/pages",
        ],
    );
    assert_contains_all(
            "responsive-component-system.md",
            &responsive_doc,
            &[
                "`ui_shell_page_contract.rs`",
                "React/MUI page chrome and routed page surfaces",
            "localized Rust page title/subtitle projection, HubWindow routing, shared status feedback, and routed page header surfaces",
        ],
    );
}

#[test]
fn shell_page_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_shell_page_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_shell_page_contract.rs",
        &contract,
        &[
            "web/src/components/shell/HubWindow.tsx",
            "web/src/components/feedback/HubStatusBanner.tsx",
            "web/src/components/feedback/HubSnackbar.tsx",
            "src/state/navigation.rs",
            "src/tauri_app/view_model.rs",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/App.tsx",
        ],
    );
    assert_not_contains_any(
        "ui_shell_page_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
            old_taffy_name.as_str(),
        ],
    );
}
