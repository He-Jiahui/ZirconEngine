//! Static contracts for the React + Material UI Hub component stack.
//! 核对主题、控件、数据、浮层、反馈与页面的底层库使用关系，防止页面承担独立控件实现。

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub must live under the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

/// 读取相对 Hub 包根的受审源码作为结构证据；调用方依赖仓库检出完整，读取失败应暴露契约来源缺失。
fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {path}: {error}");
        }),
    )
}

/// 读取仓库级交接文档或工具证据；约定 Hub 包位于仓库根下一层，不能依赖测试启动时的工作目录。
fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read repository file {path}: {error}");
        }),
    )
}

fn assert_contains_all(source_path: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_path} must contain React/MUI component contract snippet `{snippet}`"
        );
    }
}

fn assert_not_contains_any(source_path: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_path} must not keep old Hub UI contract reference `{snippet}`"
        );
    }
}

/// 核对应用对底层库的显式依赖与主题安装，参考源码只用于组件分类证据而非运行时导入。
#[test]
fn react_material_ui_packages_are_directly_composed_through_theme_and_tokens() {
    let package_json = read_crate_file("package.json");
    assert_contains_all(
        "package.json",
        &package_json,
        &[
            "\"@mui/material\": \"9.0.1\"",
            "\"@mui/icons-material\": \"9.0.1\"",
            "\"@emotion/react\": \"latest\"",
            "\"@emotion/styled\": \"latest\"",
            "\"@vitejs/plugin-react\": \"6.0.2\"",
        ],
    );

    let material_ui_package = read_repo_file("dev/material-ui/package.json");
    assert_contains_all(
        "dev/material-ui/package.json",
        &material_ui_package,
        &[
            "\"name\": \"@mui/monorepo\"",
            "\"version\": \"9.2.0\"",
            "\"private\": true",
        ],
    );
    assert!(
        repo_dir()
            .join("dev/material-ui/packages/mui-material/src")
            .is_dir(),
        "the repository-local Material UI reference tree must expose packages/mui-material/src"
    );

    let tokens = read_crate_file("web/src/theme/tokens.ts");
    assert_contains_all(
        "web/src/theme/tokens.ts",
        &tokens,
        &[
            "width: 1568",
            "height: 1003",
            "topBarHeight: 73",
            "sidebarWidth: 222",
            "sidebarCollapsedWidth: 78",
            "pagePaddingX: 30",
            "pagePaddingY: 28",
            "radius",
            "colors",
            "shadows",
        ],
    );

    let theme = read_crate_file("web/src/theme/muiTheme.ts");
    assert_contains_all(
        "web/src/theme/muiTheme.ts",
        &theme,
        &[
            "createTheme",
            "MuiButton",
            "MuiCard",
            "MuiIconButton",
            "MuiMenu",
            "MuiOutlinedInput",
            "MuiSelect",
            "MuiTooltip",
            "textTransform: \"none\"",
            "letterSpacing: 0",
        ],
    );

    let app = read_crate_file("web/src/App.tsx");
    let main = read_crate_file("web/src/main.tsx");
    for snippet in ["ThemeProvider", "hubTheme", "CssBaseline", "<App />"] {
        assert!(
            app.contains(snippet) || main.contains(snippet),
            "React app root must install the shared Material UI theme before rendering Hub components; missing {snippet}"
        );
    }
}

/// 检查每类基础输入控件由共享包装器承接，统一密度、颜色、回调和可访问名称。
#[test]
fn low_level_inputs_wrap_material_primitives_with_shared_tokens() {
    for (source_path, snippets) in [
        (
            "web/src/components/inputs/HubButton.tsx",
            vec![
                "@mui/material",
                "ButtonProps",
                "Button",
                "HubButtonTone",
                "hubTokens",
                "toneStyles",
            ],
        ),
        (
            "web/src/components/inputs/HubIconButton.tsx",
            vec![
                "@mui/material",
                "IconButtonProps",
                "IconButton",
                "Tooltip",
                "selected",
                "label",
            ],
        ),
        (
            "web/src/components/inputs/HubSearchField.tsx",
            vec![
                "@mui/material",
                "TextField",
                "InputAdornment",
                "SearchIcon",
                "compact",
                "hubTokens",
            ],
        ),
        (
            "web/src/components/inputs/HubSelect.tsx",
            vec![
                "@mui/material",
                "Select",
                "MenuItem",
                "SelectChangeEvent",
                "IconComponent",
                "renderValue",
            ],
        ),
        (
            "web/src/components/inputs/HubComboBox.tsx",
            vec![
                "@mui/material",
                "Autocomplete",
                "TextField",
                "disableClearable",
                "getOptionLabel",
            ],
        ),
        (
            "web/src/components/inputs/HubCheckbox.tsx",
            vec![
                "@mui/material",
                "Checkbox",
                "FormControlLabel",
                "checked",
                "detail",
                "hubTokens",
            ],
        ),
        (
            "web/src/components/inputs/HubSwitch.tsx",
            vec![
                "@mui/material",
                "Switch",
                "FormControlLabel",
                "checked",
                "detail",
                "hubTokens",
            ],
        ),
        (
            "web/src/components/inputs/HubTabs.tsx",
            vec![
                "@mui/material",
                "Tabs",
                "Tab",
                "variant=\"scrollable\"",
                "scrollButtons=\"auto\"",
            ],
        ),
        (
            "web/src/components/inputs/HubTextField.tsx",
            vec![
                "@mui/material",
                "TextFieldProps",
                "TextField",
                "variant=\"outlined\"",
                "size=\"small\"",
            ],
        ),
        (
            "web/src/components/inputs/HubToggle.tsx",
            vec![
                "@mui/material",
                "ToggleButton",
                "ToggleButtonGroup",
                "Tooltip",
                "exclusive",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }

    let index = read_crate_file("web/src/components/inputs/index.ts");
    assert_contains_all(
        "web/src/components/inputs/index.ts",
        &index,
        &[
            "HubButton",
            "HubCheckbox",
            "HubComboBox",
            "HubIconButton",
            "HubSearchField",
            "HubSelect",
            "HubSwitch",
            "HubTabs",
            "HubTextField",
            "HubToggle",
        ],
    );
}

/// 固定数据家族拥有列表、表格、树及交互行原语，使页面沿 DTO 组合信息。
#[test]
fn data_container_components_wrap_material_lists_tables_and_rows() {
    for (source_path, snippets) in [
        (
            "web/src/components/data/HubPanel.tsx",
            vec![
                "@mui/material",
                "Card",
                "component=\"section\"",
                "Typography",
                "hubTokens",
            ],
        ),
        (
            "web/src/components/data/HubList.tsx",
            vec![
                "@mui/material",
                "List",
                "ListItemButton",
                "ListItemIcon",
                "ListItemText",
                "selected",
            ],
        ),
        (
            "web/src/components/data/HubTreeView.tsx",
            vec![
                "@mui/material",
                "Collapse",
                "List",
                "ListItemButton",
                "defaultExpanded",
                "depth",
            ],
        ),
        (
            "web/src/components/data/ProjectTable.tsx",
            vec![
                "@mui/material",
                "Table",
                "TableBody",
                "TableCell",
                "TableHead",
                "TableRow",
            ],
        ),
        (
            "web/src/components/data/QuickActions.tsx",
            vec![
                "@mui/material",
                "ButtonBase",
                "gridTemplateColumns",
                "actionIcons",
                "ChevronRightIcon",
            ],
        ),
        (
            "web/src/components/data/SourceEngineList.tsx",
            vec![
                "@mui/material",
                "ButtonBase",
                "HubSourceEngineSummary",
                "StatusBadge",
                "StorageOutlinedIcon",
            ],
        ),
        (
            "web/src/components/data/StatusBadge.tsx",
            vec![
                "@mui/material",
                "toneMap",
                "StatusTone",
                "PlayArrowIcon",
                "CheckCircleIcon",
            ],
        ),
        (
            "web/src/components/data/MetricCard.tsx",
            vec![
                "@mui/material",
                "gridTemplateColumns",
                "toneColor",
                "hubTokens",
            ],
        ),
        (
            "web/src/components/data/EmptyStateBlock.tsx",
            vec!["@mui/material", "Typography", "icon", "detail"],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }

    let index = read_crate_file("web/src/components/data/index.ts");
    assert_contains_all(
        "web/src/components/data/index.ts",
        &index,
        &[
            "EmptyStateBlock",
            "HubList",
            "HubPanel",
            "HubTreeView",
            "MetricCard",
            "ProjectCard",
            "ProjectCover",
            "ProjectTable",
            "QuickActions",
            "SourceEngineList",
            "StatusBadge",
        ],
    );
    assert_not_contains_any(
        "web/src/components/data/index.ts",
        &index,
        &["ButtonStatesPanel"],
    );
}

/// 核对浮层、反馈和外壳表面沿底层组件体系实现，统一焦点、锚点及窗口呈现边界。
#[test]
fn overlays_feedback_and_shell_use_material_surfaces_not_standalone_html() {
    for (source_path, snippets) in [
        (
            "web/src/components/overlays/HubDialog.tsx",
            vec![
                "@mui/material",
                "Dialog",
                "DialogTitle",
                "DialogContent",
                "DialogActions",
                "slotProps",
            ],
        ),
        (
            "web/src/components/overlays/HubMenu.tsx",
            vec![
                "@mui/material",
                "Menu",
                "MenuItem",
                "anchorEl",
                "onSelect",
                "slotProps",
            ],
        ),
        (
            "web/src/components/overlays/HubPopover.tsx",
            vec![
                "@mui/material",
                "Popover",
                "anchorOrigin",
                "transformOrigin",
                "slotProps",
            ],
        ),
        (
            "web/src/components/overlays/SourceEnginePopover.tsx",
            vec![
                "HubPopover",
                "StatusBadge",
                "HubSourceEngineSummary",
                "onManage",
                "activeEngineId",
            ],
        ),
        (
            "web/src/components/overlays/UserMenuPopover.tsx",
            vec![
                "HubPopover",
                "Avatar",
                "ButtonBase",
                "menuItems",
                "userName",
                "onClose",
            ],
        ),
        (
            "web/src/components/feedback/HubStatusBanner.tsx",
            vec!["@mui/material", "Alert", "HubTaskSummary", "severity"],
        ),
        (
            "web/src/components/feedback/HubSnackbar.tsx",
            vec!["@mui/material", "Snackbar", "Alert", "task", "onClose"],
        ),
        (
            "web/src/components/shell/NavigationDrawer.tsx",
            vec![
                "@mui/material",
                "Drawer",
                "ListItemButton",
                "navItems",
                "sidebarCollapsedWidth",
            ],
        ),
        (
            "web/src/components/shell/TopBar.tsx",
            vec![
                "@mui/material",
                "Avatar",
                "ButtonBase",
                "StatusBadge",
                "StorageOutlinedIcon",
                "SourceEnginePopover",
                "UserMenuPopover",
            ],
        ),
        (
            "web/src/components/shell/HubWindow.tsx",
            vec![
                "@mui/material",
                "NavigationDrawer",
                "TopBar",
                "ProjectsDashboard",
                "SettingsPage",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }

    for (source_path, snippets) in [
        (
            "web/src/components/overlays/index.ts",
            vec![
                "HubDialog",
                "HubMenu",
                "HubPopover",
                "SourceEnginePopover",
                "UserMenuPopover",
            ],
        ),
        (
            "web/src/components/feedback/index.ts",
            vec!["HubSnackbar", "HubStatusBanner"],
        ),
        (
            "web/src/components/shell/index.ts",
            vec!["HubWindow", "NavigationDrawer", "TopBar"],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

// BUG: [CR-HUBTESTA-0009] 窗口路由值改为按需页面变量，旧直接页面映射断言必然失败；证据：HubWindow.tsx 当前映射。
/// 检查各业务页面对共享组件家族的消费和响应式布局，再核对窗口的集中路由接线。
#[test]
fn pages_compose_shared_components_with_responsive_layouts() {
    for (source_path, snippets) in [
        (
            "web/src/pages/ProjectsDashboard.tsx",
            vec![
                "ProjectCard",
                "ProjectTable",
                "QuickActions",
                "CreateProjectDialog",
                "ProjectsToolbar",
                "@media (max-width: 980px)",
            ],
        ),
        (
            "web/src/pages/ProjectBrowserPage.tsx",
            vec![
                "ProjectTable",
                "HubToggle",
                "HubSelect",
                "HubSearchField",
                "SourceEngineList",
            ],
        ),
        (
            "web/src/pages/ProjectDetailPage.tsx",
            vec![
                "ProjectMetricsGrid",
                "ProjectDetailSidebar",
                "HubTabs",
                "HubTreeView",
                "QuickActions",
            ],
        ),
        (
            "web/src/pages/SettingsPage.tsx",
            vec!["MetricCard", "HubTabs", "SettingsSection"],
        ),
        (
            "web/src/pages/CatalogPage.tsx",
            vec![
                "HubList",
                "HubTreeView",
                "MetricCard",
                "EmptyStateBlock",
                "HubTabs",
            ],
        ),
        (
            "web/src/pages/TeamPage.tsx",
            vec![
                "HubList",
                "MetricCard",
                "HubStatusBanner",
                "SourceEngineList",
                "QuickActions",
            ],
        ),
        (
            "web/src/pages/EditorPage.tsx",
            vec![
                "HubList",
                "HubPanel",
                "MetricCard",
                "SourceEngineList",
                "EmptyStateBlock",
            ],
        ),
        (
            "web/src/pages/BuildsPage.tsx",
            vec![
                "HubList",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "HubStatusBanner",
            ],
        ),
        (
            "web/src/pages/CloudPage.tsx",
            vec![
                "HubList",
                "HubTreeView",
                "MetricCard",
                "QuickActions",
                "EmptyStateBlock",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
        assert!(
            source.contains("@media (max-width:") || source.contains("gridTemplateColumns"),
            "{source_path} must keep page-level responsive grid rules"
        );
    }

    for (source_path, snippets) in [
        (
            "web/src/components/inputs/ProjectsToolbar.tsx",
            vec![
                "HubSearchField",
                "HubSelect",
                "HubToggle",
                "gridTemplateColumns",
            ],
        ),
        (
            "web/src/components/overlays/CreateProjectDialog.tsx",
            vec!["HubDialog", "HubButton", "HubComboBox", "HubTextField"],
        ),
        (
            "web/src/components/data/ProjectMetricsGrid.tsx",
            vec!["MetricCard", "gridTemplateColumns"],
        ),
        (
            "web/src/components/data/ProjectDetailSidebar.tsx",
            vec!["QuickActions", "SourceEngineList", "HubButton"],
        ),
        (
            "web/src/components/data/SettingsSection.tsx",
            vec![
                "HubComboBox",
                "HubTextField",
                "HubSwitch",
                "HubCheckbox",
                "HubTreeView",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }

    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");
    assert_contains_all(
        "web/src/components/shell/HubWindow.tsx",
        &hub_window,
        &[
            "projects: ProjectsDashboard,",
            "editor: EditorPage,",
            "builds: BuildsPage,",
            "cloud: CloudPage,",
            "assets: CatalogPage,",
            "plugins: CatalogPage,",
            "learn: CatalogPage,",
            "team: TeamPage,",
            "settings: SettingsPage,",
        ],
    );
}

/// 自读测试源码核对受审目标仍指向当前前端；禁用词分段构造，新增注释也不能携带其完整旧引用。
#[test]
fn material_usage_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_material_usage_contract.rs");
    let ui_suffix = [".", "slint"].concat();
    let old_app_path = ["src", "app"].join("/");
    let old_binding_path = ["src", "app", "binding.rs"].join("/");
    let old_reader = ["read", "_ui", "_file"].concat();
    assert_not_contains_any(
        "tests/ui_material_usage_contract.rs",
        &contract,
        &[
            ui_suffix.as_str(),
            old_app_path.as_str(),
            old_binding_path.as_str(),
            old_reader.as_str(),
        ],
    );
}
