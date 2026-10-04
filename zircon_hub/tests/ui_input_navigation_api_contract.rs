//! Static API contracts for the React/MUI Hub input and navigation surface.
//! 固定输入包装器的受控回调及动作标识集合，使页面操作沿同一派发链到达 Rust 命令入口。

use std::{collections::BTreeSet, fs, path::PathBuf};

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

/// 读取相对 Hub 包根的受审源码作为结构证据；调用方依赖仓库检出完整，读取失败应暴露契约来源缺失。
fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read Hub crate file {path}: {error}")),
    )
}

/// 读取仓库级交接文档或工具证据；约定 Hub 包位于仓库根下一层，不能依赖测试启动时的工作目录。
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

// TODO: [CR-HUBTESTA-0014] 确认动作集合提取是否会把固定区段的注释或非动作字符串误作协议值；当前按引号分片；下一步用结构化表或负例验证。
/// 为两端动作表提取固定区段内的字符串集合；调用方必须保持标记唯一且区段只包含动作值。
fn quoted_values_between(source: &str, begin: &str, end: &str) -> BTreeSet<String> {
    let start = source
        .find(begin)
        .unwrap_or_else(|| panic!("marker {begin:?} should exist"))
        + begin.len();
    let stop = source[start..]
        .find(end)
        .unwrap_or_else(|| panic!("marker {end:?} should exist after {begin:?}"))
        + start;
    source[start..stop]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// 检查前后端规范动作标识集合双向一致，避免前端发出不可解析动作或遗漏后端可用入口。
#[test]
fn hub_action_id_table_matches_react_hub_action_map_bidirectionally() {
    let action_id = read_crate_file("src/tauri_app/action_id.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    let rust_ids = quoted_values_between(
        &action_id,
        "pub(crate) fn as_str(self) -> &'static str {",
        "pub(crate) fn from_str(",
    );
    let web_ids = quoted_values_between(&types, "export const HUB_ACTION = {", "} as const;");

    assert!(
        !rust_ids.is_empty(),
        "Rust action id table must not be empty"
    );
    assert_eq!(
        rust_ids, web_ids,
        "HubActionId::as_str() table and web HUB_ACTION map must expose identical id sets"
    );
}

/// 固定历史输入别名只在后端解析，前端始终提交规范动作以便协议和调用日志保持唯一表示。
#[test]
fn hub_action_legacy_aliases_stay_rust_side_only() {
    let action_id = read_crate_file("src/tauri_app/action_id.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "action_id.rs",
        &action_id,
        &[
            "\"page\" => Some(Self::ShowPage)",
            "\"project-subpage\" => Some(Self::ShowProjectSubpage)",
            "\"open-project\" => Some(Self::SelectProject)",
        ],
    );
    assert_not_contains_any(
        "types/hub.ts",
        &types,
        &["\"page\",", "\"project-subpage\",", "\"open-project\","],
    );
}

/// 要求需要结构数据的动作有对应前端载荷类型，限制调用方把无关参数提交到另一个动作入口。
#[test]
fn payload_carrying_actions_keep_typed_entries_in_react_payload_map() {
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "[HUB_ACTION.searchProjects]: SearchProjectsPayload;",
            "[HUB_ACTION.updateNewProjectDraft]: NewProjectDraftPayload;",
            "[HUB_ACTION.createProject]: CreateProjectPayload;",
            "[HUB_ACTION.importProject]: ImportProjectPayload;",
            "[HUB_ACTION.pinProject]: ProjectTargetPayload;",
            "[HUB_ACTION.unpinProject]: ProjectTargetPayload;",
            "[HUB_ACTION.removeFromHub]: ProjectTargetPayload;",
            "[HUB_ACTION.requestDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.cancelDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.confirmDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.buildProject]: ProjectTargetPayload;",
            "[HUB_ACTION.packageProject]: ProjectTargetPayload;",
            "[HUB_ACTION.installDevice]: ProjectTargetPayload;",
            "[HUB_ACTION.openEditor]: ProjectTargetPayload;",
            "[HUB_ACTION.updateSettingsDraft]: UpdateSettingsDraftPayload;",
            "[HUB_ACTION.saveSettings]: SaveSettingsPayload;",
            "[HUB_ACTION.browseSettingsFolder]: BrowseSettingsFolderPayload;",
            "[HUB_ACTION.openResource]: OpenResourcePayload;",
            "[HUB_ACTION.openOutputFolder]: OpenOutputFolderPayload;",
        ],
    );
}

/// 固定输入家族出口，页面可从同一边界获得包装器类型和回调契约。
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

/// 核对受控值、选项和布尔状态的回调类型及只读语义，底层事件应在包装器边界变成业务值。
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

/// 核对点击、单选和页签回调的契约及无障碍名称，让上层导航通过业务值派发动作。
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

// BUG: [CR-HUBTESTA-0007] 窗口已改为按需加载页面并传播窗口动作失败回调，旧页面路由片段检查失败；证据：HubWindow.tsx。
/// 沿抽屉、顶栏、窗口和顶层状态接收检查同一派发器，避免导航组件直接另建 IPC 出口。
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
            "<NavigationDrawer",
            "activePage={state.activePage}",
            "text={state.ui.shell}",
            "engineVersion={state.engineVersion}",
            "sourceEngines={state.sourceEngines}",
            "activeSourceEngineId={state.activeSourceEngineId}",
            "onAction={onAction}",
            "const pageRoutes: Record<HubPageId, HubPageComponent> = {",
            "projects: ProjectsDashboard,",
            "assets: CatalogPage,",
            "const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;",
            "<PageComponent state={state} onAction={onAction} />",
        ],
    );
    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "const handleAction: HubActionHandler = async (actionId, targetId, payload) =>",
            "dispatchHubAction(actionId, targetId, payload)",
            "actionSequenceRef",
            "stateGenerationRef",
            "applyHubState(nextState)",
            "<HubWindow state={state} onAction={handleAction} />",
        ],
    );
    assert_not_contains_any("App.tsx", &app, &["setState(nextState);"]);
    assert_contains_all(
        "hubApi.ts",
        &hub_api,
        &[
            "dispatchHubAction<TActionId extends HubActionId>",
            "invoke<unknown>(\"hub_action\"",
            "request: { actionId, targetId, payload }",
        ],
    );
}

/// 固定页面筛选、选择和设置保存通过包装器回调提交到后端，页面局部交互状态不能替代持久化事实。
#[test]
fn routed_pages_use_input_callbacks_for_navigation_and_filters() {
    let projects = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let projects_toolbar = read_crate_file("web/src/components/inputs/ProjectsToolbar.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");
    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &projects,
        &[
            "ProjectsToolbar",
            "search={search}",
            "filter={filter}",
            "sort={sort}",
            "viewMode={viewMode}",
            "void onAction(HUB_ACTION.searchProjects, undefined, { query: value });",
            "void onAction(HUB_ACTION.setProjectFilter, value)",
            "void onAction(HUB_ACTION.setProjectSort, value)",
            "void onAction(HUB_ACTION.setProjectViewMode, value)",
            "void onAction(HUB_ACTION.viewAllProjects)",
            "void onAction(HUB_ACTION.newProject)",
        ],
    );
    assert_contains_all(
        "ProjectsToolbar.tsx",
        &projects_toolbar,
        &[
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "value={search}",
            "onChange={onSearch}",
            "value={filter}",
            "onChange={onFilter}",
            "value={sort}",
            "onChange={onSort}",
            "value={viewMode}",
            "onChange={onViewMode}",
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
            "HubTabs",
            "SettingsSection",
            "void onAction(HUB_ACTION.saveSettings, undefined, { settings: draft })",
        ],
    );
    assert_contains_all(
        "SettingsSection.tsx",
        &settings_section,
        &[
            "HubTextField",
            "HubComboBox",
            "HubCheckbox",
            "HubSwitch",
            "updateDraft",
            "browseFolder",
        ],
    );
}

/// 要求文档记录动作派发和输入回调的拥有者及验证入口，使协议调整能同时定位两端。
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

/// 自读测试源码核对受审目标仍指向当前前端；禁用词分段构造，新增注释也不能携带其完整旧引用。
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
