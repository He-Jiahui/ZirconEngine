//! Static contracts for React + Material UI Hub input primitives.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn assert_contains_all(source_path: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_path} must contain React/MUI input contract snippet `{snippet}`"
        );
    }
}

fn assert_not_contains_any(source_path: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_path} must not keep obsolete React/MUI input contract snippet `{snippet}`"
        );
    }
}

#[test]
fn input_components_are_reexported_from_the_react_barrel() {
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

#[test]
fn button_wrappers_own_material_button_and_icon_button_chrome() {
    for (source_path, snippets) in [
        (
            "web/src/components/inputs/HubButton.tsx",
            vec![
                "ButtonProps",
                "Button",
                "HubButtonTone",
                "\"primary\" | \"secondary\" | \"tertiary\" | \"danger\"",
                "toneStyles",
                "variant=\"contained\"",
                "border: \"1px solid\"",
                "hubTokens.colors.accentDim",
                "asSxArray",
            ],
        ),
        (
            "web/src/components/inputs/HubIconButton.tsx",
            vec![
                "IconButtonProps",
                "IconButton",
                "Tooltip",
                "selected?: boolean",
                "label: string",
                "tooltip?: string",
                "Tooltip title={tooltip ?? label}",
                "aria-label={label}",
                "width: 50",
                "height: 42",
                "hubTokens.colors.textSoft",
                "\"&.Mui-disabled\"",
                "asSxArray",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

#[test]
fn text_select_and_combo_wrappers_own_material_form_primitives() {
    for (source_path, snippets) in [
        (
            "web/src/components/inputs/HubSearchField.tsx",
            vec![
                "SearchIcon",
                "InputAdornment",
                "TextField",
                "compact?: boolean",
                "onChange: (value: string) => void",
                "onChange={(event) => onChange(event.target.value)}",
                "width: compact ? 260 : 307",
                "height: compact ? 36 : 47",
                "hubTokens.shadows.accent",
            ],
        ),
        (
            "web/src/components/inputs/HubTextField.tsx",
            vec![
                "TextFieldProps",
                "TextField",
                "Omit<TextFieldProps, \"variant\" | \"size\">",
                "minWidth?: number",
                "variant=\"outlined\"",
                "size=\"small\"",
                "& .MuiInputBase-root",
                "minHeight: 42",
            ],
        ),
        (
            "web/src/components/inputs/HubSelect.tsx",
            vec![
                "SelectChangeEvent",
                "MenuItem",
                "Select",
                "ExpandMoreIcon",
                "HubSelectOption",
                "IconComponent={ExpandMoreIcon}",
                "renderValue",
                "onChange(event.target.value)",
                "height: 42",
            ],
        ),
        (
            "web/src/components/inputs/HubComboBox.tsx",
            vec![
                "Autocomplete",
                "TextField",
                "HubComboBoxOption",
                "disableClearable={options.length > 0}",
                "getOptionLabel={(option) => option.label}",
                "isOptionEqualToValue",
                "onChange(option.value)",
                "height: 42",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

#[test]
fn binary_toggle_and_tab_wrappers_own_material_state_primitives() {
    for (source_path, snippets) in [
        (
            "web/src/components/inputs/HubCheckbox.tsx",
            vec![
                "Checkbox",
                "FormControlLabel",
                "HubCheckboxProps",
                "checked: boolean",
                "detail?: string",
                "disabled?: boolean",
                "const isDisabled = disabled || !onChange;",
                "disabled={isDisabled}",
                "onChange?.(event.target.checked)",
                "&.Mui-checked",
                "hubTokens.colors.accent",
            ],
        ),
        (
            "web/src/components/inputs/HubSwitch.tsx",
            vec![
                "Switch",
                "FormControlLabel",
                "HubSwitchProps",
                "checked: boolean",
                "detail?: string",
                "disabled?: boolean",
                "const isDisabled = disabled || !onChange;",
                "disabled={isDisabled}",
                "onChange?.(event.target.checked)",
                "MuiSwitch-switchBase.Mui-checked",
                "rgba(33,213,207,0.44)",
            ],
        ),
        (
            "web/src/components/inputs/HubToggle.tsx",
            vec![
                "ToggleButton",
                "ToggleButtonGroup",
                "Tooltip",
                "exclusive",
                "HubToggleOption",
                "aria-label={option.label}",
                "width: 50",
                "height: 42",
                "&.Mui-selected",
            ],
        ),
        (
            "web/src/components/inputs/HubTabs.tsx",
            vec![
                "Tab",
                "Tabs",
                "HubTabOption",
                "variant=\"scrollable\"",
                "scrollButtons=\"auto\"",
                "minHeight: 38",
                "MuiTabs-indicator",
                "iconPosition=\"start\"",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }
}

#[test]
fn theme_and_tokens_define_shared_input_density_and_state() {
    let theme = read_crate_file("web/src/theme/muiTheme.ts");
    assert_contains_all(
        "web/src/theme/muiTheme.ts",
        &theme,
        &[
            "MuiButton",
            "height: 42",
            "textTransform: \"none\"",
            "MuiIconButton",
            "MuiOutlinedInput",
            "MuiSelect",
            "MuiTooltip",
            "borderRadius: hubTokens.radius.compact",
            "letterSpacing: 0",
        ],
    );

    let tokens = read_crate_file("web/src/theme/tokens.ts");
    assert_contains_all(
        "web/src/theme/tokens.ts",
        &tokens,
        &[
            "compact: 7",
            "accent: \"#21d5cf\"",
            "accentDim",
            "lineStrong",
            "textSoft",
            "textMuted",
            "accent:",
        ],
    );
}

#[test]
fn pages_compose_shared_input_components_instead_of_raw_material_inputs() {
    for (source_path, snippets) in [
        (
            "web/src/pages/ProjectsDashboard.tsx",
            vec![
                "from \"../components/inputs\"",
                "HubButton",
                "HubComboBox",
                "HubSearchField",
                "HubSelect",
                "HubTextField",
                "HubToggle",
            ],
        ),
        (
            "web/src/pages/ProjectBrowserPage.tsx",
            vec![
                "from \"../components/inputs\"",
                "HubButton",
                "HubSearchField",
                "HubSelect",
                "HubToggle",
            ],
        ),
        (
            "web/src/pages/ProjectDetailPage.tsx",
            vec!["from \"../components/inputs\"", "HubButton", "HubTabs"],
        ),
        (
            "web/src/pages/CatalogPage.tsx",
            vec!["from \"../components/inputs\"", "HubSearchField", "HubTabs"],
        ),
        (
            "web/src/pages/SettingsPage.tsx",
            vec![
                "from \"../components/inputs\"",
                "HubButton",
                "HubCheckbox",
                "HubComboBox",
                "HubSwitch",
                "HubTabs",
                "HubTextField",
            ],
        ),
        (
            "web/src/pages/EditorPage.tsx",
            vec![
                "from \"../components/inputs\"",
                "HubButton",
                "HubCheckbox",
                "HubSwitch",
                "HubTabs",
            ],
        ),
        (
            "web/src/pages/BuildsPage.tsx",
            vec!["from \"../components/inputs\"", "HubButton", "HubTabs"],
        ),
        (
            "web/src/pages/CloudPage.tsx",
            vec![
                "from \"../components/inputs\"",
                "HubButton",
                "HubCheckbox",
                "HubSwitch",
                "HubTabs",
            ],
        ),
        (
            "web/src/pages/TeamPage.tsx",
            vec!["from \"../components/inputs\"", "HubTabs"],
        ),
        (
            "web/src/pages/WorkspacePage.tsx",
            vec![
                "from \"../components/inputs\"",
                "HubButton",
                "HubCheckbox",
                "HubSwitch",
                "HubTabs",
            ],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
        assert_page_material_imports_do_not_include_input_primitives(source_path, &source);
    }
}

#[test]
fn shell_and_dialog_surfaces_use_input_wrappers_for_commands() {
    for (source_path, snippets) in [
        (
            "web/src/components/shell/TopBar.tsx",
            vec![
                "HubIconButton",
                "label={state.ui.shell.notifications} tooltip={state.ui.shell.notificationsDetail} disabled",
                "label={state.ui.shell.help} onClick={() => void onAction(HUB_ACTION.showPage, \"learn\")}",
                "label={state.ui.shell.settings}",
                "label={state.ui.shell.minimize} onClick={handleMinimize}",
                "label={state.ui.shell.maximize} onClick={handleToggleMaximize}",
                "label={state.ui.shell.close} onClick={handleClose}",
            ],
        ),
        (
            "web/src/components/overlays/HubDialog.tsx",
            vec!["Dialog", "DialogActions", "DialogContent", "DialogTitle"],
        ),
        (
            "web/src/components/overlays/HubMenu.tsx",
            vec!["Menu", "MenuItem", "HubMenuItem", "onSelect(item.id)"],
        ),
    ] {
        let source = read_crate_file(source_path);
        assert_contains_all(source_path, &source, &snippets);
    }

    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    assert_not_contains_any(
        "web/src/pages/ProjectsDashboard.tsx",
        &dashboard,
        &[
            "ButtonStatesPanel",
            "text.buttonStates",
            "buttonStatePrimary",
        ],
    );
}

#[test]
fn input_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_inputs_contract.rs");
    let obsolete_suffix = [".", "slint"].concat();
    let obsolete_reader = ["read", "_ui", "_file"].concat();
    let obsolete_root_reader = ["ui", "_dir"].concat();
    let obsolete_app_path = ["src", "app"].join("/");

    for forbidden in [
        obsolete_suffix,
        obsolete_reader,
        obsolete_root_reader,
        obsolete_app_path,
    ] {
        assert!(
            !contract.contains(&forbidden),
            "React input contract must not keep old UI contract reference `{forbidden}`"
        );
    }
}

fn assert_page_material_imports_do_not_include_input_primitives(source_path: &str, source: &str) {
    let forbidden_material_imports = [
        "Autocomplete",
        "Button",
        "Checkbox",
        "FormControlLabel",
        "IconButton",
        "InputAdornment",
        "MenuItem",
        "Select",
        "Switch",
        "Tab",
        "Tabs",
        "TextField",
        "ToggleButton",
        "ToggleButtonGroup",
    ];

    for line in source.lines().filter(|line| line.contains("@mui/material")) {
        for forbidden in forbidden_material_imports {
            assert!(
                !line.contains(forbidden),
                "{source_path} must consume shared input components instead of importing raw Material input primitive `{forbidden}`"
            );
        }
    }
}
