//! Static contracts for the Zircon Hub Tauri, React, and Material UI foundation.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub should live below the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {path}: {error}");
        }),
    )
}

fn read_optional_crate_file(path: &str) -> Option<String> {
    fs::read_to_string(crate_dir().join(path))
        .ok()
        .map(normalize_newlines)
}

fn assert_contains_all(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{label} must contain foundation snippet: {snippet}"
        );
    }
}

fn assert_not_contains_any(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{label} must not keep obsolete foundation snippet: {snippet}"
        );
    }
}

#[test]
fn tauri_launcher_and_build_path_are_hard_cut_to_react_shell() {
    let cargo = read_crate_file("Cargo.toml");
    let build = read_crate_file("build.rs");
    let lib = read_crate_file("src/lib.rs");
    let main = read_crate_file("src/main.rs");
    let launcher = read_crate_file("src/tauri_app/mod.rs");

    assert_contains_all(
        &cargo,
        &[
            "autobins = false",
            "[[bin]]",
            "name = \"zircon_hub\"",
            "path = \"src/main.rs\"",
            "tauri = { version = \"2.11.2\", features = [] }",
            "tauri-build = { version = \"2.6.2\", features = [] }",
        ],
        "Cargo.toml",
    );
    assert_not_contains_any(
        &cargo.to_ascii_lowercase(),
        &["slint", "i-slint"],
        "Cargo.toml",
    );

    assert_contains_all(&build, &["fn main()", "tauri_build::build()"], "build.rs");
    assert_not_contains_any(
        &build.to_ascii_lowercase(),
        &["i_slint_compiler", "include_generated"],
        "build.rs",
    );

    assert_contains_all(
        &lib,
        &["pub mod tauri_app;", "pub use error::HubError;"],
        "src/lib.rs",
    );
    assert!(
        !lib.contains(&["pub mod ", "app;"].concat()),
        "src/lib.rs must not expose the removed compiled UI module"
    );
    assert_contains_all(
        &main,
        &[
            "fn main() -> Result<(), zircon_hub::HubError>",
            "zircon_hub::tauri_app::run()",
        ],
        "src/main.rs",
    );
    assert_contains_all(
        &launcher,
        &[
            "mod commands;",
            "mod runtime_state;",
            "mod view_model;",
            "#[tauri::command]",
            "fn hub_state(state: tauri::State<'_, HubCommandState>) -> Result<HubViewModel, String>",
            "request: HubActionRequest",
            "tauri::Builder::default()",
            ".manage(HubCommandState::load()?)",
            "tauri::generate_handler![hub_state, hub_action]",
            "tauri::generate_context!()",
        ],
        "src/tauri_app/mod.rs",
    );

    assert!(
        !crate_dir()
            .join(["src", "app", "mod.rs"].iter().collect::<PathBuf>())
            .exists(),
        "the removed compiled UI module root must stay absent"
    );
    assert!(
        !crate_dir()
            .join(["src", "tauri_app.rs"].iter().collect::<PathBuf>())
            .exists(),
        "the Tauri boundary should stay folder-backed instead of collapsing into a one-file launcher"
    );
}

#[test]
fn tauri_configuration_uses_vite_window_and_capability_boundary() {
    let tauri_config = read_crate_file("tauri.conf.json");
    let capability = read_crate_file("capabilities/default.json");
    let vite = read_crate_file("vite.config.ts");
    let package = read_crate_file("package.json");

    assert_contains_all(
        &tauri_config,
        &[
            "\"$schema\": \"https://schema.tauri.app/config/2\"",
            "\"beforeDevCommand\": \"npm run dev\"",
            "\"beforeBuildCommand\": \"npm run build\"",
            "\"devUrl\": \"http://localhost:1420\"",
            "\"frontendDist\": \"web/dist\"",
            "\"label\": \"main\"",
            "\"width\": 1568",
            "\"height\": 1003",
            "\"minWidth\": 960",
            "\"minHeight\": 680",
            "\"decorations\": false",
            "\"transparent\": true",
            "\"icon\": [\"icons/icon.ico\"]",
        ],
        "tauri.conf.json",
    );
    assert_contains_all(
        &capability,
        &[
            "\"$schema\": \"../gen/schemas/desktop-schema.json\"",
            "\"identifier\": \"default\"",
            "\"local\": true",
            "\"windows\": [\"main\"]",
            "\"core:default\"",
            "\"core:window:allow-minimize\"",
            "\"core:window:allow-toggle-maximize\"",
            "\"core:window:allow-close\"",
        ],
        "capabilities/default.json",
    );
    assert_contains_all(
        &vite,
        &[
            "import react from \"@vitejs/plugin-react\";",
            "root: \"web\"",
            "plugins: [react()]",
            "port: 1420",
            "strictPort: true",
            "envPrefix: [\"VITE_\", \"TAURI_\"]",
            "outDir: \"dist\"",
        ],
        "vite.config.ts",
    );
    assert_contains_all(
        &package,
        &[
            "\"dev\": \"vite --host 127.0.0.1 --port 1420\"",
            "\"build\": \"npm run typecheck && vite build\"",
            "\"typecheck\": \"tsc --noEmit -p tsconfig.json && tsc --noEmit -p tsconfig.node.json\"",
            "\"tauri:dev\": \"tauri dev\"",
            "\"tauri:build\": \"tauri build\"",
            "\"@mui/material\": \"9.0.1\"",
            "\"@mui/icons-material\": \"9.0.1\"",
            "\"@tauri-apps/api\": \"2.11.0\"",
            "\"react\": \"19.2.7\"",
            "\"@vitejs/plugin-react\": \"6.0.2\"",
        ],
        "package.json",
    );
}

#[test]
fn react_root_installs_mui_theme_and_backend_state_flow() {
    let main = read_crate_file("web/src/main.tsx");
    let app_path = ["web/src", "App.tsx"].join("/");
    let app = read_crate_file(&app_path);
    let hub_api = read_crate_file("web/src/tauri/hubApi.ts");

    assert_contains_all(
        &main,
        &[
            "import React from \"react\";",
            "import ReactDOM from \"react-dom/client\";",
            "import { CssBaseline, ThemeProvider } from \"@mui/material\";",
            "import { App } from \"./App\";",
            "import { hubTheme } from \"./theme/muiTheme\";",
            "ReactDOM.createRoot(document.getElementById(\"root\") as HTMLElement).render(",
            "<React.StrictMode>",
            "<ThemeProvider theme={hubTheme}>",
            "<CssBaseline />",
            "<App />",
        ],
        "web/src/main.tsx",
    );
    assert_contains_all(
        &app,
        &[
            "import { HubErrorBoundary, HubSnackbar } from \"./components/feedback\";",
            "import { HubWindow } from \"./components/shell\";",
            "import { fallbackShellState } from \"./data/hubData\";",
            "import { dispatchHubAction, loadHubState, subscribeHubStateChanged } from \"./tauri/hubApi\";",
            "const [state, setState] = useState<HubShellState>(fallbackShellState);",
            "const stateGenerationRef = useRef(0);",
            "const actionSequenceRef = useRef(0);",
            "function applyHubState(nextState: HubShellState) {",
            "stateGenerationRef.current += 1;",
            "loadHubState().then((nextState) =>",
            "subscribeHubStateChanged((nextState) =>",
            "unlisten?.();",
            "const handleAction: HubActionHandler = async (actionId, targetId, payload) =>",
            "const actionSequence = actionSequenceRef.current + 1;",
            "const stateGenerationAtDispatch = stateGenerationRef.current;",
            "const nextState = await dispatchHubAction(actionId, targetId, payload);",
            "applyHubState(nextState);",
            "<HubWindow state={state} onAction={handleAction} />",
            "<HubSnackbar task={state.taskSummary} open={snackbarOpen}",
        ],
        app_path.as_str(),
    );
    assert_not_contains_any(&app, &["setState(nextState);"], app_path.as_str());
    assert_contains_all(
        &hub_api,
        &[
            "import { invoke } from \"@tauri-apps/api/core\";",
            "import { listen } from \"@tauri-apps/api/event\";",
            "import type { UnlistenFn } from \"@tauri-apps/api/event\";",
            "import { fallbackShellState } from \"../data/hubData\";",
            "import type { HubActionId, HubActionPayload, HubShellState } from \"../types/hub\";",
            "export async function loadHubState(): Promise<HubShellState>",
            "return assertHubShellState(await invoke<unknown>(\"hub_state\"));",
            "export async function dispatchHubAction<TActionId extends HubActionId>(",
            "payload?: HubActionPayload<TActionId>,",
            "await invoke<unknown>(\"hub_action\",",
            "request: { actionId, targetId, payload }",
            "export async function subscribeHubStateChanged",
            "Promise<UnlistenFn>",
            "listen<unknown>(\"hub-state-changed\"",
            "function isTauriRuntime()",
            "\"__TAURI_INTERNALS__\" in window",
        ],
        "web/src/tauri/hubApi.ts",
    );
}

#[test]
fn theme_tokens_define_window_density_palette_and_mui_overrides() {
    let tokens = read_crate_file("web/src/theme/tokens.ts");
    let theme = read_crate_file("web/src/theme/muiTheme.ts");
    let package = read_crate_file("package.json");

    assert_contains_all(
        &tokens,
        &[
            "export const hubTokens =",
            "width: 1568",
            "height: 1003",
            "topBarHeight: 73",
            "sidebarWidth: 222",
            "sidebarCollapsedWidth: 78",
            "pagePaddingX: 30",
            "pagePaddingY: 28",
            "compact: 7",
            "panel: 8",
            "card: 8",
            "background: \"#111212\"",
            "chrome: \"#151515\"",
            "panel: \"#202020\"",
            "lineStrong: \"rgba(255,255,255,0.16)\"",
            "text: \"#eeeeee\"",
            "accent: \"#21d5cf\"",
            "success: \"#77d77a\"",
            "warning: \"#ffc24d\"",
            "error: \"#ef655e\"",
            "panel: \"inset 0 0 0 1px rgba(255,255,255,0.04)",
            "as const",
        ],
        "web/src/theme/tokens.ts",
    );
    assert_contains_all(
        &theme,
        &[
            "import { createTheme } from \"@mui/material/styles\";",
            "export const hubTheme = createTheme({",
            "mode: \"dark\"",
            "default: hubTokens.colors.background",
            "paper: hubTokens.colors.panel",
            "main: hubTokens.colors.accent",
            "shape:",
            "borderRadius: hubTokens.radius.compact",
            "fontFamily: 'Inter, Roboto, \"Segoe UI\", Arial, sans-serif'",
            "letterSpacing: 0",
            "textTransform: \"none\"",
            "MuiButton:",
            "MuiCard:",
            "MuiIconButton:",
            "MuiMenu:",
            "MuiOutlinedInput:",
            "MuiSelect:",
            "MuiTooltip:",
        ],
        "web/src/theme/muiTheme.ts",
    );
    assert_contains_all(
        &package,
        &[
            "\"@emotion/react\": \"latest\"",
            "\"@emotion/styled\": \"latest\"",
            "\"@mui/material\": \"9.0.1\"",
            "\"@mui/icons-material\": \"9.0.1\"",
        ],
        "package.json",
    );
    assert!(
        repo_dir()
            .join("dev/material-ui/packages/mui-material/src")
            .is_dir(),
        "the checked-in Material UI reference source must remain available for Hub component taxonomy"
    );
}

#[test]
fn component_family_barrels_match_bottom_up_material_layers() {
    let inputs = read_crate_file("web/src/components/inputs/index.ts");
    let data = read_crate_file("web/src/components/data/index.ts");
    let feedback = read_crate_file("web/src/components/feedback/index.ts");
    let overlays = read_crate_file("web/src/components/overlays/index.ts");
    let shell = read_crate_file("web/src/components/shell/index.ts");

    assert_contains_all(
        &inputs,
        &[
            "export * from \"./HubButton\";",
            "export * from \"./HubCheckbox\";",
            "export * from \"./HubComboBox\";",
            "export * from \"./HubIconButton\";",
            "export * from \"./HubSearchField\";",
            "export * from \"./HubSelect\";",
            "export * from \"./HubSwitch\";",
            "export * from \"./HubTabs\";",
            "export * from \"./HubTextField\";",
            "export * from \"./HubToggle\";",
        ],
        "components/inputs barrel",
    );
    assert_contains_all(
        &data,
        &[
            "export * from \"./EmptyStateBlock\";",
            "export * from \"./HubList\";",
            "export * from \"./HubPanel\";",
            "export * from \"./HubTreeView\";",
            "export * from \"./MetricCard\";",
            "export * from \"./ProjectCard\";",
            "export * from \"./ProjectCover\";",
            "export * from \"./ProjectTable\";",
            "export * from \"./QuickActions\";",
            "export * from \"./SourceEngineList\";",
            "export * from \"./StatusBadge\";",
        ],
        "components/data barrel",
    );
    assert_not_contains_any(&data, &["ButtonStatesPanel"], "components/data barrel");
    assert_contains_all(
        &feedback,
        &[
            "export * from \"./HubSnackbar\";",
            "export * from \"./HubStatusBanner\";",
        ],
        "components/feedback barrel",
    );
    assert_contains_all(
        &overlays,
        &[
            "export * from \"./HubDialog\";",
            "export * from \"./HubMenu\";",
            "export * from \"./HubPopover\";",
            "export * from \"./SourceEnginePopover\";",
            "export * from \"./UserMenuPopover\";",
        ],
        "components/overlays barrel",
    );
    assert_contains_all(
        &shell,
        &[
            "export * from \"./HubWindow\";",
            "export * from \"./NavigationDrawer\";",
            "export * from \"./TopBar\";",
        ],
        "components/shell barrel",
    );
}

#[test]
fn shell_and_pages_are_structural_composition_surfaces() {
    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");
    let top_bar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");

    assert_contains_all(
        &hub_window,
        &[
            "import { BuildsPage } from \"../../pages/BuildsPage\";",
            "import { CatalogPage } from \"../../pages/CatalogPage\";",
            "import { CloudPage } from \"../../pages/CloudPage\";",
            "import { EditorPage } from \"../../pages/EditorPage\";",
            "import { ProjectsDashboard } from \"../../pages/ProjectsDashboard\";",
            "import { SettingsPage } from \"../../pages/SettingsPage\";",
            "import { TeamPage } from \"../../pages/TeamPage\";",
            "import { WorkspacePage } from \"../../pages/WorkspacePage\";",
            "import { NavigationDrawer } from \"./NavigationDrawer\";",
            "import { TopBar } from \"./TopBar\";",
            "width: \"100vw\"",
            "height: \"100vh\"",
            "height: `calc(100vh - ${hubTokens.window.topBarHeight}px)`",
            "component=\"main\"",
            "projects: ProjectsDashboard,",
            "projects: ProjectsDashboard,",
            "<PageComponent state={state} onAction={onAction} />",
        ],
        "HubWindow",
    );
    assert_contains_all(
        &top_bar,
        &[
            "import { StatusBadge } from \"../data\";",
            "import { HubIconButton } from \"../inputs\";",
            "import { SourceEnginePopover, UserMenuPopover } from \"../overlays\";",
            "height: hubTokens.window.topBarHeight",
            "gridTemplateColumns: \"222px minmax(0, 1fr) auto\"",
            "gridTemplateColumns: \"78px minmax(0, 1fr) auto\"",
            "state.taskStatus.map((status) =>",
            "void onAction(HUB_ACTION.selectEngine, engineId);",
            "void onAction(HUB_ACTION.showPage, \"settings\");",
        ],
        "TopBar",
    );
    assert_contains_all(
        &drawer,
        &[
            "import { Box, ButtonBase, Drawer, List, ListItemButton, ListItemIcon, Tooltip, Typography } from \"@mui/material\";",
            "import { useState } from \"react\";",
            "const [collapsed, setCollapsed] = useState(false);",
            "const drawerWidth = collapsed ? hubTokens.window.sidebarCollapsedWidth : hubTokens.window.sidebarWidth;",
            "width: drawerWidth",
            "transition: \"width 160ms ease\"",
            "text.navItems.map(({ id, label }) =>",
            "const Icon = navIcons[id];",
            "onClick={() => void onAction(HUB_ACTION.showPage, id)}",
            "{text.engineStatus}",
            "{text.checkForUpdates}",
            "{text.checkForUpdatesDetail}",
            "disabled",
            "const collapseLabel = collapsed ? text.expand : text.collapse;",
            "{collapseLabel}",
            "onClick={() => setCollapsed((current) => !current)}",
        ],
        "NavigationDrawer",
    );

    for page in [
        "ProjectsDashboard",
        "ProjectBrowserPage",
        "ProjectDetailPage",
        "CatalogPage",
        "EditorPage",
        "BuildsPage",
        "CloudPage",
        "TeamPage",
        "SettingsPage",
        "WorkspacePage",
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}.tsx"));
        assert!(
            source.contains(&format!("export function {page}(")),
            "{page} must expose a single named page composition function"
        );
        assert!(
            source.contains("from \"../components/"),
            "{page} must assemble shared component families instead of becoming a local widget owner"
        );
        assert!(
            source.contains("type {") || source.contains("import type"),
            "{page} must consume typed backend DTOs"
        );
        assert!(
            source.contains("@media (max-width:"),
            "{page} must keep responsive layout constraints"
        );
    }
}

#[test]
fn backend_commands_and_view_model_keep_rust_state_as_source_of_truth() {
    let tauri_app = read_crate_file("src/tauri_app/mod.rs");
    let commands = read_crate_file("src/tauri_app/commands.rs");
    let action_request = read_crate_file("src/tauri_app/action_request.rs");
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let scoped_views = read_crate_file("src/tauri_app/runtime_state/scoped_views.rs");
    let action_tasks = read_crate_file("src/tauri_app/runtime_state/action_tasks.rs");
    let build_actions = read_crate_file("src/tauri_app/runtime_state/build_actions.rs");
    let editor_launch_actions =
        read_crate_file("src/tauri_app/runtime_state/editor_launch_actions.rs");
    let project_delivery_actions =
        read_crate_file("src/tauri_app/runtime_state/project_delivery_actions.rs");
    let quick_actions = read_crate_file("src/tauri_app/runtime_state/quick_actions.rs");
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let action_history_dto = read_crate_file("src/tauri_app/view_model/action_history.rs");
    let coming_soon = read_crate_file("src/tauri_app/view_model/coming_soon.rs");
    let ui_text = read_crate_file("src/tauri_app/view_model/ui_text.rs");
    let types = read_crate_file("web/src/types/hub.ts");
    let data = read_crate_file("web/src/data/hubData.ts");

    assert_contains_all(
        &commands,
        &[
            "pub(super) struct HubCommandState",
            "session: Arc<Mutex<HubRuntimeSession>>",
            "focus_refresh_pending: Arc<AtomicBool>",
            "HubRuntimeSession::load()",
            "pub(super) fn refresh_recent_projects_on_window_focus",
            "thread::spawn(move ||",
            "pub(super) fn hub_state(",
            "pub(super) fn hub_action(",
            "HubRuntimeSession::should_run_action_in_background(&request)",
            "let should_spawn = session.start_background_action_or_record_error(&request)?;",
            "if should_spawn {",
            "spawn_background_action(request, session_handle, app.clone());",
            "let emit_state = |view_model: &HubViewModel|",
            "run_background_worker_loop(request, &session_handle, &emit_state);",
            "app.emit(\"hub-state-changed\", &view_model)",
        ],
        "tauri_app/commands.rs",
    );
    assert_contains_all(
        &runtime_state,
        &[
            "pub(super) struct HubRuntimeSession",
            "HubConfig::load(&config_path)?",
            "hub_recent_projects_path",
            "reconcile_shared_recent_projects(",
            "shared_recent_projects_snapshot",
            "refresh_shared_recent_projects_on_focus",
            "load_shared_recent_projects(&self.shared_recent_projects_path)",
            "mod scoped_views;",
            "refresh_project_context_views",
            "self.refresh_source_scoped_views()",
            "self.refresh_selected_project_scoped_views()",
            "pub(super) fn view_model(&self) -> HubViewModel",
            "pub(super) fn apply_action(",
            "let action_id = request.action()?;",
            "request.parse_as(action_id)",
            "record_action_payload_failure",
            "HubAction::ShowPage { target_id }",
            "HubAction::SearchProjects { query }",
            "HubAction::SelectEngine { target_id }",
            "HubAction::BuildProject { target_id, payload }",
            "HubAction::PackageProject { target_id, payload }",
            "HubAction::InstallDevice { target_id, payload }",
            "HubAction::OpenEditor { target_id, payload }",
            "fn persist(",
        ],
        "tauri_app/runtime_state.rs",
    );
    assert_contains_all(
        &tauri_app,
        &[
            ".on_window_event(|window, event|",
            "tauri::WindowEvent::Focused(true)",
            "refresh_recent_projects_on_window_focus",
        ],
        "tauri_app/mod.rs",
    );
    assert_contains_all(
        &action_request,
        &[
            "pub(crate) struct HubActionRequest",
            "pub(crate) enum HubAction",
            "pub(crate) fn action(&self) -> Result<HubActionId, HubError>",
            "pub(in crate::tauri_app) fn parse_as(",
            "pub(crate) trait ValidatePayload",
            "fn parse_payload<T>(action: HubActionId, payload: Option<&Value>) -> Result<T, HubError>",
            "HubActionId::ShowPage",
            "HubActionId::SearchProjects",
            "HubActionId::SelectEngine",
            "HubActionId::BuildProject",
            "HubActionId::PackageProject",
            "HubActionId::InstallDevice",
            "HubActionId::OpenEditor",
            "CreateProjectActionPayload",
            "BrowseSettingsFolderPayload",
            "OpenResourcePayload",
            "OpenOutputFolderPayload",
        ],
        "tauri_app/action_request.rs",
    );
    assert_contains_all(
        &scoped_views,
        &[
            "discover_asset_catalog_for_scope",
            "discover_learn_catalog_for_scope",
            "discover_plugin_catalog_with_project_roots",
            "discover_team_overview",
            "pub(super) fn refresh_source_scoped_views",
            "pub(super) fn refresh_selected_project_scoped_views",
            "fn selected_project_catalog_root(&self) -> Option<PathBuf>",
            "fn source_engine_catalog_roots(&self) -> Vec<PathBuf>",
            "push_development_roots(&mut roots, engine.source_dir.clone());",
        ],
        "tauri_app/runtime_state/scoped_views.rs",
    );
    assert_contains_all(
        &action_tasks,
        &[
            "enum BackgroundHubAction",
            "HubActionId::BuildProject => Some(Self::BuildProject)",
            "HubActionId::PackageProject => Some(Self::PackageProject)",
            "HubActionId::InstallDevice => Some(Self::InstallDevice)",
            "HubActionId::OpenEditor => Some(Self::OpenEditor)",
            "TaskStatus::running_operation(",
            "pub(in crate::tauri_app) trait BackgroundTask",
            "pub(in crate::tauri_app) fn execute_background_task",
            "pub(in crate::tauri_app) fn run_background_worker_loop",
            "record_background_action_error",
        ],
        "runtime_state/action_tasks.rs",
    );
    assert_contains_all(
        &build_actions,
        &[
            "pub(in crate::tauri_app) struct PendingEditorRuntimeBuild",
            "impl BackgroundTask for PendingEditorRuntimeBuild",
            "pub(in crate::tauri_app) fn prepare_background_editor_runtime_build",
            "pub(in crate::tauri_app) fn complete_background_editor_runtime_build",
            "selected_or_latest_recent_project_with_engine_for_action",
            "record_active_build(",
        ],
        "runtime_state/build_actions.rs",
    );
    assert_contains_all(
        &editor_launch_actions,
        &[
            "pub(in crate::tauri_app) struct PendingEditorLaunch",
            "pub(super) fn open_selected_project_or_editor",
            "pub(in crate::tauri_app) fn prepare_background_editor_launch",
            "pub(in crate::tauri_app) fn complete_background_editor_launch",
            "launch_editor(command)?",
            "Command::new(executable).spawn()?",
            "record_editor_launch_failure(",
        ],
        "runtime_state/editor_launch_actions.rs",
    );
    assert_contains_all(
        &project_delivery_actions,
        &[
            "pub(in crate::tauri_app) struct PendingProjectPackage",
            "pub(in crate::tauri_app) struct PendingDeviceInstall",
            "pub(super) fn package_recent_project",
            "pub(super) fn install_recent_project_to_device",
            "pub(in crate::tauri_app) fn prepare_background_project_package",
            "pub(in crate::tauri_app) fn complete_background_project_package",
            "pub(in crate::tauri_app) fn prepare_background_device_install",
            "pub(in crate::tauri_app) fn complete_background_device_install",
            "package_project(&self.request)",
            "install_package_to_device(&install_request)",
            "record_package_success(",
        ],
        "runtime_state/project_delivery_actions.rs",
    );
    assert_contains_all(
        &quick_actions,
        &["record_action_and_persist"],
        "runtime_state/quick_actions.rs",
    );
    assert_contains_all(
        &view_model,
        &[
            "#[derive(Debug, Clone, Serialize)]",
            "pub(crate) struct HubViewModel",
            "pub active_page: String",
            "pub task_summary: HubTaskSummary",
            "pub task_id: u64",
            "pub queued: usize",
            "pub browser_projects: Vec<HubRecentProject>",
            "pub selected_project: Option<HubProjectDetail>",
            "pub quick_actions: Vec<HubQuickAction>",
            "pub source_engines: Vec<HubSourceEngineSummary>",
            "pub assets: Vec<HubAssetItem>",
            "pub plugins: Vec<HubPluginItem>",
            "pub learn_resources: Vec<HubLearnItem>",
            "pub team: HubTeamSummary",
            "pub action_history: Vec<HubActionHistoryItem>",
            "pub coming_soon: Vec<HubComingSoonEntry>",
            "pub settings: HubSettingsSummary",
            "pub(crate) fn from_snapshot(snapshot: &HubSnapshot) -> Self",
            "coming_soon: coming_soon_entries(snapshot.settings.language)",
            "action_history: action_history_rows(",
            "snapshot.settings.language",
            "snapshot.filtered_recent_projects()",
        ],
        "tauri_app/view_model.rs",
    );
    assert_contains_all(
        &action_history_dto,
        &[
            "pub(crate) struct HubActionHistoryItem",
            "pub kind: String",
            "record.action.id()",
            "kind: record.action.id().to_string()",
            "let text = HubTextBundle::new(language);",
            "action: text.action_label(record.action).to_string()",
            "status: text.action_status_label(record.status).to_string()",
            "let detail = text.render_message(&record.detail);",
            "let log_excerpt = text.render_message(&record.log_excerpt);",
            "let detail_rows = action_history_detail_rows(",
            ".map(|recovery| text.render_message(recovery))",
            "command_line: record.command_line.clone()",
        ],
        "tauri_app/view_model/action_history.rs",
    );
    assert_contains_all(
        &ui_text,
        &[
            "pub(crate) struct HubActionText",
            "pub open_resource: String",
            "open_resource: text.pair(\"Open Resource\", \"打开资源\").to_string()",
        ],
        "tauri_app/view_model/ui_text.rs",
    );
    assert_contains_all(
        &coming_soon,
        &[
            "pub(crate) struct HubComingSoonEntry",
            "pub(crate) fn coming_soon_entries(language: HubLanguage) -> Vec<HubComingSoonEntry>",
            "pub category_label: String",
            "pub meta: String",
            "\"asset-import\"",
            "\"plugin-install\"",
            "\"plugin-toggle\"",
            "\"marketplace-download\"",
            "\"remote-sync\"",
            "\"account-service\"",
            "\"cloud-repository\"",
            "\"team-invite\"",
            "\"team-permissions\"",
            "\"remote-collaboration\"",
            "fn coming_soon_category_label",
            "\"local-delivery\" => text.pair(\"Local Delivery\", \"本地交付\")",
            "let category_label = coming_soon_category_label(category, text).to_string();",
            "let status = text.pair(\"Coming Soon\", \"敬请期待\").to_string();",
            "meta: coming_soon_meta(&category_label, &status, text)",
            "disabled: true",
        ],
        "tauri_app/view_model/coming_soon.rs",
    );
    assert_contains_all(
        &types,
        &[
            "export interface HubShellState",
            "export interface HubComingSoonEntry",
            "export type HubActionHistoryKind",
            "kind: HubActionHistoryKind;",
            "export interface HubSettingsOptionText",
            "buildProfileOptions: HubSettingsOptionText[];",
            "languageOptions: HubSettingsOptionText[];",
            "export interface OpenResourcePayload",
            "openResource: string;",
            "[HUB_ACTION.openResource]: OpenResourcePayload;",
            "taskSummary: HubTaskSummary;",
            "taskId: number;",
            "queued: number;",
            "browserProjects: HubRecentProject[];",
            "selectedProject: HubProjectDetail | null;",
            "sourceEngines: HubSourceEngineSummary[];",
            "actionHistory: HubActionHistoryItem[];",
            "comingSoon: HubComingSoonEntry[];",
        ],
        "web/src/types/hub.ts",
    );
    assert_contains_all(
        &data,
        &[
            "import brandMarkAsset from \"../../../assets/brand/zircon-mark.svg\";",
            "import elysiumCover from \"../../../assets/covers/reference/project-elysium.png\";",
            "export const coverById: Record<string, string>",
            "export const fallbackShellState: HubShellState",
            "demoMode: true",
            "projects: []",
            "selectedProject: null",
            "rustupPath: \"rustup\"",
            "id: \"rustup-path\"",
            "title: \"Rustup\"",
            "category: \"assets\"",
            "category: \"plugins\"",
            "category: \"local-delivery\"",
            "category: \"team\"",
            "status: \"敬请期待\"",
            "disabled: true",
        ],
        "web/src/data/hubData.ts",
    );
    assert_not_contains_any(
        &data,
        &[
            "docs/ui-and-layout/hub.png",
            "hub-web-reference-1568x1003.png",
            "hub-ai-drafts",
            "name: \"Elysium Chronicles\"",
            "status: \"Active\"",
            "kind: \"Environment\"",
            "source: \"Project\"",
            "displayName: \"Render Pipeline\"",
            "Forward and deferred renderer modules for editor and runtime builds.",
            "title: \"Getting Started\"",
            "category: \"Guide\"",
        ],
        "web/src/data/hubData.ts",
    );
}

#[test]
fn coming_soon_entries_expose_visible_localized_categories() {
    let coming_soon = read_optional_crate_file("src/tauri_app/view_model/coming_soon.rs")
        .unwrap_or_else(|| read_crate_file("src/tauri_app/view_model/ui_text.rs"));
    let types = read_crate_file("web/src/types/hub.ts");
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let team = read_crate_file("web/src/pages/TeamPage.tsx");
    let data = read_crate_file("web/src/data/hubData.ts");

    assert_contains_all(
        &coming_soon,
        &[
            "pub category_label: String",
            "fn coming_soon_category_label",
            "\"assets\" => text.pair(\"Assets\", \"资产\")",
            "\"plugins\" => text.pair(\"Plugins\", \"插件\")",
            "\"local-delivery\" => text.pair(\"Local Delivery\", \"本地交付\")",
            "\"team\" => text.pair(\"Team\", \"团队\")",
            "let category_label = coming_soon_category_label(category, text).to_string();",
            "meta: coming_soon_meta(&category_label, &status, text)",
            "fn coming_soon_meta",
        ],
        "coming soon DTO",
    );
    assert_contains_all(
        &types,
        &["categoryLabel: string;", "meta: string;"],
        "web/src/types/hub.ts",
    );
    for (label, source) in [
        ("CatalogPage.tsx", catalog.as_str()),
        ("CloudPage.tsx", cloud.as_str()),
        ("EditorPage.tsx", editor.as_str()),
        ("TeamPage.tsx", team.as_str()),
    ] {
        assert_contains_all(source, &["entry.meta"], label);
        assert_not_contains_any(
            source,
            &["`${entry.categoryLabel} / ${entry.status}`"],
            label,
        );
    }
    assert_contains_all(
        &data,
        &[
            "categoryLabel: \"资产\"",
            "categoryLabel: \"插件\"",
            "categoryLabel: \"本地交付\"",
            "categoryLabel: \"团队\"",
            "meta: \"资产 / 敬请期待\"",
            "meta: \"插件 / 敬请期待\"",
            "meta: \"本地交付 / 敬请期待\"",
            "meta: \"团队 / 敬请期待\"",
        ],
        "web/src/data/hubData.ts",
    );
}

#[test]
fn foundation_contract_is_cut_over_to_react_sources() {
    let source = read_crate_file("tests/ui_foundation_contract.rs");
    let obsolete_ui_suffix = [".", "slint"].concat();
    let obsolete_reader = ["read", "_ui", "_file"].concat();
    let obsolete_root_reader = ["ui", "_dir"].concat();
    let obsolete_app_path = ["src", "app"].join("/");

    for obsolete in [
        obsolete_ui_suffix.as_str(),
        obsolete_reader.as_str(),
        obsolete_root_reader.as_str(),
        obsolete_app_path.as_str(),
    ] {
        assert!(
            !source.contains(obsolete),
            "foundation contract must not inspect the removed UI-file or app-module surfaces: {obsolete}"
        );
    }

    assert_contains_all(
        &source,
        &[
            "web/src/main.tsx",
            "App.tsx",
            "web/src/theme/tokens.ts",
            "web/src/theme/muiTheme.ts",
            "web/src/components/inputs/index.ts",
            "web/src/components/data/index.ts",
            "web/src/components/feedback/index.ts",
            "web/src/components/overlays/index.ts",
            "web/src/components/shell/index.ts",
            "src/tauri_app/mod.rs",
            "src/tauri_app/commands.rs",
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/runtime_state/action_tasks.rs",
            "src/tauri_app/runtime_state/quick_actions.rs",
            "src/tauri_app/runtime_state/editor_launch_actions.rs",
            "src/tauri_app/runtime_state/project_delivery_actions.rs",
            "src/tauri_app/view_model/coming_soon.rs",
            "src/tauri_app/view_model.rs",
            "tauri.conf.json",
            "capabilities/default.json",
            "package.json",
            "vite.config.ts",
        ],
        "foundation contract",
    );
}
