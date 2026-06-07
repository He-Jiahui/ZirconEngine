//! Static API contracts for the React/MUI Hub input and navigation surface.

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
            "{source_name} should contain input/navigation API snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete input/navigation API snippet {snippet:?}"
        );
    }
}

#[test]
fn input_barrel_exports_stable_react_wrapper_api_surface() {
    let index = read_crate_file("web/src/components/inputs/index.ts");

    assert_contains_all(
        "components/inputs/index.ts",
        &index,
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
    );
}

#[test]
fn text_select_combo_and_binary_inputs_preserve_typed_props_and_callbacks() {
    let search = read_crate_file("web/src/components/inputs/HubSearchField.tsx");
    let text = read_crate_file("web/src/components/inputs/HubTextField.tsx");
    let select = read_crate_file("web/src/components/inputs/HubSelect.tsx");
    let combo = read_crate_file("web/src/components/inputs/HubComboBox.tsx");
    let checkbox = read_crate_file("web/src/components/inputs/HubCheckbox.tsx");
    let switch = read_crate_file("web/src/components/inputs/HubSwitch.tsx");

    assert_contains_all(
        "HubSearchField.tsx",
        &search,
        &[
            "export interface HubSearchFieldProps",
            "value: string;",
            "placeholder: string;",
            "compact?: boolean;",
            "onChange: (value: string) => void;",
            "onChange={(event) => onChange(event.target.value)}",
            "slotProps",
            "InputAdornment",
        ],
    );
    assert_contains_all(
        "HubTextField.tsx",
        &text,
        &[
            "export interface HubTextFieldProps extends Omit<TextFieldProps, \"variant\" | \"size\">",
            "minWidth?: number;",
            "variant=\"outlined\"",
            "size=\"small\"",
            "Array.isArray(sx) ? sx : sx ? [sx] : []",
        ],
    );
    assert_contains_all(
        "HubSelect.tsx",
        &select,
        &[
            "export interface HubSelectOption",
            "export interface HubSelectProps",
            "value: string;",
            "options: HubSelectOption[];",
            "minWidth?: number;",
            "onChange: (value: string) => void;",
            "const handleChange = (event: SelectChangeEvent) => {",
            "onChange(event.target.value);",
            "renderValue={(selected) =>",
            "MenuItem key={option.value} value={option.value}",
        ],
    );
    assert_contains_all(
        "HubComboBox.tsx",
        &combo,
        &[
            "export interface HubComboBoxOption",
            "export interface HubComboBoxProps",
            "value: string;",
            "options: HubComboBoxOption[];",
            "placeholder?: string;",
            "minWidth?: number;",
            "onChange: (value: string) => void;",
            "const selected = options.find((option) => option.value === value) ?? null;",
            "getOptionLabel={(option) => option.label}",
            "isOptionEqualToValue={(option, current) => option.value === current.value}",
            "onChange(option.value);",
        ],
    );
    for (name, source, primitive) in [
        ("HubCheckbox.tsx", checkbox, "Checkbox"),
        ("HubSwitch.tsx", switch, "Switch"),
    ] {
        assert_contains_all(
            name,
            &source,
            &[
                "checked: boolean;",
                "label: string;",
                "detail?: string;",
                "disabled?: boolean;",
                "onChange?: (checked: boolean) => void;",
                "const isDisabled = disabled || !onChange;",
                "disabled={isDisabled}",
                "onChange={(event) => onChange?.(event.target.checked)}",
                primitive,
                "FormControlLabel",
            ],
        );
    }
}

#[test]
fn button_icon_toggle_and_tabs_preserve_navigation_callback_contracts() {
    let button = read_crate_file("web/src/components/inputs/HubButton.tsx");
    let icon_button = read_crate_file("web/src/components/inputs/HubIconButton.tsx");
    let toggle = read_crate_file("web/src/components/inputs/HubToggle.tsx");
    let tabs = read_crate_file("web/src/components/inputs/HubTabs.tsx");

    assert_contains_all(
        "HubButton.tsx",
        &button,
        &[
            "export type HubButtonTone = \"primary\" | \"secondary\" | \"tertiary\" | \"danger\";",
            "export interface HubButtonProps extends Omit<ButtonProps, \"variant\">",
            "tone?: HubButtonTone;",
            "toneStyles[tone]",
            "variant=\"contained\"",
            "...asSxArray(sx)",
        ],
    );
    assert_contains_all(
        "HubIconButton.tsx",
        &icon_button,
        &[
            "export interface HubIconButtonProps extends IconButtonProps",
            "selected?: boolean;",
            "label: string;",
            "tooltip?: string;",
            "Tooltip title={tooltip ?? label}",
            "aria-label={label}",
            "\"&.Mui-disabled\"",
            "...asSxArray(sx)",
        ],
    );
    assert_contains_all(
        "HubToggle.tsx",
        &toggle,
        &[
            "export interface HubToggleOption",
            "value: string;",
            "label: string;",
            "icon: ReactNode;",
            "export interface HubToggleProps",
            "onChange: (value: string) => void;",
            "ToggleButtonGroup",
            "exclusive",
            "nextValue: string | null",
            "if (nextValue) {",
            "onChange(nextValue);",
            "aria-label={option.label}",
        ],
    );
    assert_contains_all(
        "HubTabs.tsx",
        &tabs,
        &[
            "export interface HubTabOption",
            "value: string;",
            "label: string;",
            "icon?: ReactElement;",
            "export interface HubTabsProps",
            "onChange: (value: string) => void;",
            "Tabs",
            "onChange={(_, nextValue: string) => onChange(nextValue)}",
            "Tab",
            "iconPosition=\"start\"",
        ],
    );
}

#[test]
fn navigation_components_share_one_action_dispatcher_api() {
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");
    let app = read_crate_file("web/src/App.tsx");
    let hub_api = read_crate_file("web/src/tauri/hubApi.ts");

    assert_contains_all(
        "NavigationDrawer.tsx",
        &drawer,
        &[
            "export interface NavigationDrawerProps",
            "activePage: string;",
            "onAction: HubActionHandler;",
            "const [collapsed, setCollapsed] = useState(false);",
            "text.navItems.map",
            "const selected = activePage === id;",
            "selected={selected}",
            "onClick={() => void onAction(HUB_ACTION.showPage, id)}",
            "onClick={() => setCollapsed((current) => !current)}",
            "@media (max-width: 980px)",
        ],
    );
    assert_contains_all(
        "TopBar.tsx",
        &topbar,
        &[
            "export interface TopBarProps",
            "state: HubShellState;",
            "onAction: HubActionHandler;",
            "const handleUserAction = (actionId: string) => {",
            "void onAction(HUB_ACTION.showPage, \"settings\")",
            "void onAction(HUB_ACTION.showPage, \"learn\")",
            "void onAction(HUB_ACTION.showPage, \"team\")",
            "void onAction(HUB_ACTION.selectEngine, engineId);",
            "SourceEnginePopover",
            "UserMenuPopover",
            "HubIconButton label={state.ui.shell.settings}",
        ],
    );
    assert_contains_all(
        "HubWindow.tsx",
        &hub_window,
        &[
            "export interface HubWindowProps",
            "state: HubShellState;",
            "onAction: HubActionHandler;",
            "<TopBar state={state} onAction={onAction} />",
            "<NavigationDrawer activePage={state.activePage} text={state.ui.shell} engineVersion={state.engineVersion} onAction={onAction}",
            "ProjectsDashboard state={state} onAction={onAction}",
            "CatalogPage state={state} onAction={onAction}",
            "WorkspacePage state={state} onAction={onAction}",
        ],
    );
    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "const handleAction: HubActionHandler = async (actionId, targetId, payload) =>",
            "dispatchHubAction(actionId, targetId, payload)",
            "setState(nextState)",
            "<HubWindow state={state} onAction={handleAction} />",
        ],
    );
    assert_contains_all(
        "hubApi.ts",
        &hub_api,
        &[
            "dispatchHubAction<TActionId extends HubActionId>",
            "invoke<HubShellState>(\"hub_action\"",
            "request: { actionId, targetId, payload }",
        ],
    );
}

#[test]
fn routed_pages_use_input_callbacks_for_navigation_and_filters() {
    let projects = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &projects,
        &[
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "HubComboBox",
            "HubTextField",
            "void onAction(HUB_ACTION.searchProjects, undefined, { query: value });",
            "void onAction(HUB_ACTION.setProjectFilter, value)",
            "void onAction(HUB_ACTION.setProjectSort, value)",
            "void onAction(HUB_ACTION.setProjectViewMode, value)",
            "void onAction(HUB_ACTION.viewAllProjects)",
            "void onAction(HUB_ACTION.newProject)",
        ],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "void onAction(HUB_ACTION.showProjectSubpage, \"dashboard\")",
            "void onAction(HUB_ACTION.newProject)",
            "void onAction(HUB_ACTION.openProjectDetail, project.id)",
        ],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "HubTabs",
            "onChange={setTab}",
            "void onAction(HUB_ACTION.viewAllProjects)",
            "void onAction(HUB_ACTION.openEditor, undefined, projectTarget)",
        ],
    );
    assert_contains_all(
        "CatalogPage.tsx",
        &catalog,
        &[
            "HubSearchField",
            "onChange={setQuery}",
            "HubTabs value={tab}",
            "onChange={setTab}",
        ],
    );
    assert_contains_all(
        "SettingsPage.tsx",
        &settings,
        &[
            "HubTextField",
            "HubComboBox",
            "HubCheckbox",
            "HubSwitch",
            "HubTabs",
            "void onAction(HUB_ACTION.saveSettings, undefined, { settings: draft })",
        ],
    );
}

#[test]
fn input_navigation_api_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_input_navigation_api_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_input_navigation_api_contract",
            "## Input Navigation API Contract Cutover",
            "React/MUI input/navigation API",
            "web/src/components/inputs/index.ts",
            "web/src/components/shell/NavigationDrawer.tsx",
            "web/src/components/shell/TopBar.tsx",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/App.tsx",
            "web/src/tauri/hubApi.ts",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_input_navigation_api_contract.rs`",
            "React/MUI input/navigation API",
            "TypeScript props replace Slint exported input structs",
            "NavigationDrawer, TopBar, HubWindow, App, and hubApi keep one action dispatcher",
        ],
    );
}

#[test]
fn input_navigation_api_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_input_navigation_api_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_input_navigation_api_contract.rs",
        &contract,
        &[
            "web/src/components/inputs/index.ts",
            "web/src/components/inputs/HubButton.tsx",
            "web/src/components/inputs/HubSearchField.tsx",
            "web/src/components/inputs/HubSelect.tsx",
            "web/src/components/inputs/HubComboBox.tsx",
            "web/src/components/inputs/HubTabs.tsx",
            "web/src/components/shell/NavigationDrawer.tsx",
            "web/src/components/shell/TopBar.tsx",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/App.tsx",
            "web/src/tauri/hubApi.ts",
        ],
    );
    assert_not_contains_any(
        "ui_input_navigation_api_contract.rs",
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
