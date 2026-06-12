//! Static guardrail contracts for the React + Material UI Hub frontend.

use std::{
    fs,
    path::{Path, PathBuf},
};

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

fn web_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_web_files(&crate_dir().join("web/src"), &mut files);
    files.sort();
    files
}

fn collect_web_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("failed to read {dir:?}: {error}"))
    {
        let path = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {dir:?}: {error}"))
            .path();
        if path.is_dir() {
            collect_web_files(&path, files);
            continue;
        }
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| matches!(extension, "ts" | "tsx" | "css"))
        {
            files.push(path);
        }
    }
}

fn display_path(path: &Path) -> String {
    path.strip_prefix(crate_dir())
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn imports_symbol(line: &str, symbol: &str) -> bool {
    if !line.contains("@mui/material") {
        return false;
    }
    line.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == symbol)
}

fn material_primitive_owner_allowed(path: &str, primitive: &str) -> bool {
    match primitive {
        "Button" => path == "web/src/components/inputs/HubButton.tsx",
        "IconButton" => matches!(
            path,
            "web/src/components/inputs/HubIconButton.tsx"
                | "web/src/components/data/ProjectCard.tsx"
                | "web/src/components/data/ProjectTable.tsx"
        ),
        "TextField" => matches!(
            path,
            "web/src/components/inputs/HubTextField.tsx"
                | "web/src/components/inputs/HubSearchField.tsx"
                | "web/src/components/inputs/HubComboBox.tsx"
        ),
        "Autocomplete" => path == "web/src/components/inputs/HubComboBox.tsx",
        "Select" => path == "web/src/components/inputs/HubSelect.tsx",
        "Checkbox" => path == "web/src/components/inputs/HubCheckbox.tsx",
        "Switch" => path == "web/src/components/inputs/HubSwitch.tsx",
        "Tabs" => path == "web/src/components/inputs/HubTabs.tsx",
        "ToggleButton" => path == "web/src/components/inputs/HubToggle.tsx",
        "Card" => path.starts_with("web/src/components/data/"),
        "Table" => path == "web/src/components/data/ProjectTable.tsx",
        "ListItemButton" => {
            path.starts_with("web/src/components/data/")
                || path == "web/src/components/shell/NavigationDrawer.tsx"
        }
        "Dialog" => path == "web/src/components/overlays/HubDialog.tsx",
        "Menu" => path == "web/src/components/overlays/HubMenu.tsx",
        "Popover" => path == "web/src/components/overlays/HubPopover.tsx",
        "Drawer" => path == "web/src/components/shell/NavigationDrawer.tsx",
        "Snackbar" => path == "web/src/components/feedback/HubSnackbar.tsx",
        "Alert" => path.starts_with("web/src/components/feedback/"),
        _ => false,
    }
}

fn assert_contains_all(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_name} should contain global React/MUI guardrail snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete or page-local global guardrail snippet {snippet:?}"
        );
    }
}

#[test]
fn pages_are_composition_surfaces_not_raw_material_owner_layers() {
    let mut violations = Vec::new();
    let forbidden_page_imports = [
        "Button",
        "IconButton",
        "TextField",
        "Select",
        "Autocomplete",
        "Checkbox",
        "Switch",
        "Tabs",
        "Tab",
        "ToggleButton",
        "Card",
        "Paper",
        "Table",
        "ListItemButton",
        "Drawer",
        "Dialog",
        "Menu",
        "Popover",
        "Snackbar",
        "Alert",
    ];

    for path in web_files()
        .into_iter()
        .filter(|path| display_path(path).starts_with("web/src/pages/"))
    {
        let source = normalize_newlines(fs::read_to_string(&path).unwrap());
        let name = display_path(&path);
        assert_contains_all(
            &name,
            &source,
            &["../components/data", "../components/inputs"],
        );
        for line in source.lines().filter(|line| line.contains("@mui/material")) {
            for forbidden in forbidden_page_imports {
                if imports_symbol(line, forbidden) {
                    violations.push(format!(
                        "{name}: page imports raw MUI primitive {forbidden}: {line}"
                    ));
                }
            }
        }
        for forbidden_tag in [
            "<Button ",
            "<IconButton",
            "<TextField",
            "<Select",
            "<Autocomplete",
            "<Checkbox",
            "<Switch",
            "<Tabs",
            "<Tab ",
            "<ToggleButton",
            "<Card",
            "<Paper",
            "<Table",
            "<ListItemButton",
            "<Drawer",
            "<Dialog",
            "<Menu",
            "<Popover",
            "<Snackbar",
            "<Alert",
        ] {
            if source.contains(forbidden_tag) {
                violations.push(format!("{name}: page renders raw MUI tag {forbidden_tag}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "React pages must assemble shared Hub component families instead of owning raw Material primitives:\n{}",
        violations.join("\n")
    );
}

#[test]
fn material_primitive_ownership_stays_in_matching_component_families() {
    let guarded_primitives = [
        "Button",
        "IconButton",
        "TextField",
        "Autocomplete",
        "Select",
        "Checkbox",
        "Switch",
        "Tabs",
        "ToggleButton",
        "Card",
        "Table",
        "ListItemButton",
        "Dialog",
        "Menu",
        "Popover",
        "Drawer",
        "Snackbar",
        "Alert",
    ];

    let mut violations = Vec::new();
    for path in web_files() {
        let name = display_path(&path);
        let source = normalize_newlines(fs::read_to_string(&path).unwrap());
        for line in source.lines().filter(|line| line.contains("@mui/material")) {
            for primitive in guarded_primitives {
                if imports_symbol(line, primitive)
                    && !material_primitive_owner_allowed(&name, primitive)
                {
                    violations.push(format!(
                        "{name}: {primitive} is imported outside the allowed React/MUI owner set: `{line}`"
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Raw Material primitive imports must stay in their wrapper family:\n{}",
        violations.join("\n")
    );
}

#[test]
fn page_layouts_avoid_absolute_positioning_and_viewport_arithmetic() {
    let mut violations = Vec::new();
    for path in web_files()
        .into_iter()
        .filter(|path| display_path(path).starts_with("web/src/pages/"))
    {
        let name = display_path(&path);
        let source = normalize_newlines(fs::read_to_string(&path).unwrap());
        for forbidden in [
            "position: \"absolute\"",
            "position: \"fixed\"",
            "left:",
            "right:",
            "top:",
            "bottom:",
            "width: \"100vw\"",
            "height: \"100vh\"",
            "calc(100vw",
            "calc(100vh",
        ] {
            if source.contains(forbidden) {
                violations.push(format!("{name}: page layout contains {forbidden}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "React page layouts should use grids/flex and shell-provided geometry instead of absolute positioning or viewport arithmetic:\n{}",
        violations.join("\n")
    );
}

#[test]
fn app_and_shell_keep_state_loading_routing_and_feedback_boundaries() {
    let app = read_crate_file("web/src/App.tsx");
    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");
    let top_bar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");

    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "loadHubState().then",
            "dispatchHubAction(actionId, targetId, payload)",
            "actionSequenceRef",
            "stateGenerationRef",
            "applyHubState(nextState)",
            "HubWindow state={state} onAction={handleAction}",
            "HubSnackbar task={state.taskSummary}",
            "const shellText = stateRef.current.ui.shell;",
            "shellText.actionFailed",
        ],
    );
    assert_not_contains_any("App.tsx", &app, &["setState(nextState);"]);
    assert_contains_all(
        "HubWindow.tsx",
        &hub_window,
        &[
            "width: \"100vw\"",
            "height: \"100vh\"",
            "overflow: \"hidden\"",
            "TopBar state={state} onAction={onAction}",
            "<NavigationDrawer",
            "activePage={state.activePage}",
            "text={state.ui.shell}",
            "engineVersion={state.engineVersion}",
            "sourceEngines={state.sourceEngines}",
            "activeSourceEngineId={state.activeSourceEngineId}",
            "onAction={onAction}",
            "component=\"main\"",
            "const pageRoutes: Record<HubPageId, HubPageComponent> = {",
            "projects: ProjectsDashboard,",
            "assets: CatalogPage,",
            "const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;",
            "<PageComponent state={state} onAction={onAction} />",
        ],
    );
    assert_contains_all(
        "TopBar.tsx",
        &top_bar,
        &[
            "HubIconButton",
            "SourceEnginePopover",
            "UserMenuPopover",
            "void onAction",
        ],
    );
    assert_contains_all(
        "NavigationDrawer.tsx",
        &drawer,
        &[
            "Drawer",
            "ListItemButton",
            "void onAction(HUB_ACTION.showPage, id)",
        ],
    );
}

#[test]
fn hub_window_capture_scripts_cover_tauri_visual_state_matrix() {
    let capture = read_repo_file(
        ".codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-window.ps1",
    );
    let project_pages = read_repo_file(
        ".codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1",
    );
    let matrix = read_repo_file(
        ".codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-visual-state-matrix.ps1",
    );
    let comparison = read_repo_file(
        ".codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/compare-hub-tauri-references.ps1",
    );
    let skill_doc = read_repo_file(
        ".codex/skills/zircon-project-skills/capture-hub-window-screenshot/SKILL.md",
    );

    assert_contains_all(
        "capture-hub-window.ps1",
        &capture,
        &[
            "[string]$VisualTaskState = \"\"",
            "$env:ZIRCON_HUB_VISUAL_TASK_STATE = $VisualTaskState",
            "Remove-Item Env:\\ZIRCON_HUB_VISUAL_TASK_STATE",
            "Title -eq \"Zircon Hub\"",
            "[string]$RequireWindowTitle = \"\"",
            "Expected window title '$RequireWindowTitle'",
        ],
    );
    assert_contains_all(
        "capture-hub-project-pages.ps1",
        &project_pages,
        &[
            "$browserX = [int][Math]::Round($WindowWidth * 0.607)",
            "$script:MinimumCaptureWidth = [int][Math]::Floor($WindowWidth * 0.90)",
            "Refusing to capture '$Path' from window titled '$($Window.Title)'; expected 'Zircon Hub'.",
            "Refusing to capture '$Path' because window width $($Window.Width) is below minimum $script:MinimumCaptureWidth.",
        ],
    );
    assert_contains_all(
        "capture-hub-visual-state-matrix.ps1",
        &matrix,
        &[
            "\"hub-state-$Name.png\"",
            "Invoke-VisualStateCapture -Name \"editor\"",
            "Invoke-VisualStateCapture -Name \"assets\"",
            "Invoke-VisualStateCapture -Name \"builds\"",
            "Invoke-VisualStateCapture -Name \"plugins\"",
            "Invoke-VisualStateCapture -Name \"cloud\"",
            "Invoke-VisualStateCapture -Name \"team\"",
            "Invoke-VisualStateCapture -Name \"learn\"",
            "Invoke-VisualStateCapture -Name \"settings\"",
            "Invoke-VisualStateCapture -Name \"source-engine-popup\"",
            "Invoke-VisualStateCapture -Name \"user-menu\"",
            "Invoke-VisualStateCapture -Name \"project-browser-empty\"",
            "Invoke-VisualStateCapture -Name \"loading\"",
            "Invoke-VisualStateCapture -Name \"error\"",
            "Visual state capture '$Name' must require state-specific WebView text before capture.",
            "-Name \"editor\" -Page \"editor\" -RequireWebViewText \"Launch Target\"",
            "-Name \"assets\" -Page \"assets\" -RequireWebViewText \"Assets Catalog\"",
            "-Name \"builds\" -Page \"builds\" -RequireWebViewText \"Build Workflow\"",
            "-Name \"plugins\" -Page \"plugins\" -RequireWebViewText \"Plugins Catalog\"",
            "-Name \"cloud\" -Page \"cloud\" -RequireWebViewText \"Package Outputs\"",
            "-Name \"team\" -Page \"team\" -RequireWebViewText \"Team Members\"",
            "-Name \"learn\" -Page \"learn\" -RequireWebViewText \"Learn Catalog\"",
            "-Name \"settings\" -Page \"settings\" -RequireWebViewText \"Build Defaults\"",
            "-Name \"project-browser-empty\" -Page \"projects\" -ProjectSubpage \"project-browser\" -ProjectViewMode \"list\" -IncludeProject $false -RequireWebViewText \"No projects found\"",
            "-Name \"loading\" -Page \"builds\" -VisualTaskState \"loading\" -RequireWebViewText \"Loading Hub state\"",
            "-Name \"error\" -Page \"builds\" -VisualTaskState \"error\" -RequireWebViewText \"Visual verification error state\"",
            "-ConfigMode Current",
            "-VisualTaskState \"loading\"",
            "-VisualTaskState \"error\"",
            "-RequireWindowTitle \"Zircon Hub\"",
            "Test-HubScreenshotMostlyWhite",
            "Test-HubScreenshotMissingAccent",
            "Screenshot for '$Name' is mostly white and cannot be trusted",
            "does not contain enough Hub accent pixels",
        ],
    );
    assert_contains_all(
        "compare-hub-tauri-references.ps1",
        &comparison,
        &[
            "hub-tauri-reference-comparison.json",
            "hub-tauri-reference-comparison.md",
            "target\\hub-visual-check\\tauri-project-pages-full-matrix",
            "hub-state-editor.png",
            "hub-state-learn.png",
            "hub-projects-new-project.png",
            "hub-projects-detail-delete-confirm.png",
            "hub-state-project-browser-empty.png",
            "$MaxMeanDelta = 35.0",
            "$MaxRmsDelta = 75.0",
            "$MinimumActualWidth = 1400",
            "AI draft PNGs are checked for inventory presence",
            "Final similarity metrics compare against the HTML/CSS-finalized",
        ],
    );
    assert_contains_all(
        "capture Hub skill doc",
        &skill_doc,
        &[
            "capture-hub-visual-state-matrix.ps1",
            "compare-hub-tauri-references.ps1",
            "Editor, Assets, Builds, Plugins, Cloud, Team, Learn, Settings, Source Engine popup, User menu, empty Project Browser, loading, and error-state screenshots",
            "Hub Tauri reference comparison",
            "ZIRCON_HUB_VISUAL_TASK_STATE",
            "real Tauri `hub_state` view-model",
        ],
    );
}

#[test]
fn typography_visual_assets_and_global_css_stay_centralized() {
    let theme = read_crate_file("web/src/theme/muiTheme.ts");
    let tokens = read_crate_file("web/src/theme/tokens.ts");
    let styles = read_crate_file("web/src/styles.css");
    let data = read_crate_file("web/src/data/hubData.ts");

    assert_contains_all(
        "muiTheme.ts",
        &theme,
        &[
            "createTheme",
            "typography: {",
            "letterSpacing: 0",
            "textTransform: \"none\"",
            "MuiButton",
            "MuiOutlinedInput",
            "MuiTooltip",
        ],
    );
    assert_contains_all(
        "tokens.ts",
        &tokens,
        &[
            "colors",
            "radius",
            "window",
            "pagePaddingX",
            "topBarHeight",
            "sidebarWidth",
        ],
    );
    assert_contains_all(
        "styles.css",
        &styles,
        &[
            "font-family: Inter, Roboto, \"Segoe UI\", Arial, sans-serif;",
            "button,",
            "input,",
            "textarea,",
            "select",
            "font: inherit;",
        ],
    );
    assert_contains_all(
        "hubData.ts",
        &data,
        &[
            "../../../assets/brand/zircon-mark.svg",
            "../../../assets/covers/reference/",
            "fallbackShellState",
        ],
    );
    assert_not_contains_any(
        "hubData.ts",
        &data,
        &["docs/ui-and-layout", "hub-ai-drafts", "hub.png"],
    );
}

#[test]
fn global_rules_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_global_rules_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_global_rules_contract",
            "## Global Rules Contract Cutover",
            "React/MUI global guardrails",
            "web/src/App.tsx",
            "web/src/components",
            "web/src/pages",
            "web/src/theme",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_global_rules_contract.rs`",
            "React/MUI global guardrails",
            "raw Material primitive ownership",
            "pages remain composition surfaces",
            "absolute positioning stays out of page layouts",
        ],
    );
}

#[test]
fn global_rules_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_global_rules_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_global_rules_contract.rs",
        &contract,
        &[
            "web/src/App.tsx",
            "web/src/components",
            "web/src/pages",
            "web/src/theme",
            "web/src/data/hubData.ts",
        ],
    );
    assert_not_contains_any(
        "ui_global_rules_contract.rs",
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
