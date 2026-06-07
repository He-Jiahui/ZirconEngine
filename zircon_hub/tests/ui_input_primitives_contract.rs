//! Static contracts for React + Material UI low-level input primitives.

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
            "{source_name} should contain input primitive contract snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete or page-local input primitive snippet {snippet:?}"
        );
    }
}

#[test]
fn input_barrel_exports_every_low_level_react_wrapper() {
    let index = read_crate_file("web/src/components/inputs/index.ts");

    assert_contains_all(
        "components/inputs/index.ts",
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
fn text_search_select_and_combo_wrap_mui_text_entry_primitives() {
    let search = read_crate_file("web/src/components/inputs/HubSearchField.tsx");
    let text = read_crate_file("web/src/components/inputs/HubTextField.tsx");
    let select = read_crate_file("web/src/components/inputs/HubSelect.tsx");
    let combo = read_crate_file("web/src/components/inputs/HubComboBox.tsx");

    assert_contains_all(
        "HubSearchField.tsx",
        &search,
        &[
            "SearchIcon",
            "InputAdornment",
            "TextField",
            "HubSearchFieldProps",
            "compact?: boolean",
            "onChange(event.target.value)",
            "width: compact ? 260 : 307",
            "height: compact ? 36 : 47",
            "& input::placeholder",
        ],
    );
    assert_contains_all(
        "HubTextField.tsx",
        &text,
        &[
            "TextFieldProps",
            "Omit<TextFieldProps, \"variant\" | \"size\">",
            "variant=\"outlined\"",
            "size=\"small\"",
            "minWidth",
            "\"& .MuiInputBase-root\": { minHeight: 42 }",
        ],
    );
    assert_contains_all(
        "HubSelect.tsx",
        &select,
        &[
            "SelectChangeEvent",
            "Select,",
            "MenuItem",
            "IconComponent={ExpandMoreIcon}",
            "renderValue={(selected) => (",
            "Typography variant=\"body2\" color=\"text.secondary\"",
            "height: 42",
            "onChange(event.target.value)",
        ],
    );
    assert_contains_all(
        "HubComboBox.tsx",
        &combo,
        &[
            "Autocomplete",
            "TextField",
            "HubComboBoxOption",
            "disableClearable={options.length > 0}",
            "getOptionLabel={(option) => option.label}",
            "isOptionEqualToValue",
            "renderInput={(params) => <TextField {...params} placeholder={placeholder} />}",
            "height: 42",
        ],
    );
}

#[test]
fn button_icon_toggle_and_tabs_wrap_mui_click_targets_with_shared_tokens() {
    let button = read_crate_file("web/src/components/inputs/HubButton.tsx");
    let icon_button = read_crate_file("web/src/components/inputs/HubIconButton.tsx");
    let toggle = read_crate_file("web/src/components/inputs/HubToggle.tsx");
    let tabs = read_crate_file("web/src/components/inputs/HubTabs.tsx");

    assert_contains_all(
        "HubButton.tsx",
        &button,
        &[
            "ButtonProps",
            "HubButtonTone",
            "\"primary\" | \"secondary\" | \"tertiary\" | \"danger\"",
            "variant=\"contained\"",
            "border: \"1px solid\"",
            "toneStyles[tone]",
            "asSxArray",
        ],
    );
    assert_contains_all(
        "HubIconButton.tsx",
        &icon_button,
        &[
            "IconButtonProps",
            "Tooltip",
            "selected?: boolean",
            "label: string",
            "tooltip?: string",
            "Tooltip title={tooltip ?? label}",
            "aria-label={label}",
            "width: 50",
            "height: 42",
            "backgroundColor: selected ?",
            "\"&.Mui-disabled\"",
        ],
    );
    assert_contains_all(
        "HubToggle.tsx",
        &toggle,
        &[
            "ToggleButtonGroup",
            "ToggleButton",
            "Tooltip",
            "exclusive",
            "nextValue: string | null",
            "aria-label={option.label}",
            "width: 50",
            "height: 42",
            "\"&.Mui-selected\"",
        ],
    );
    assert_contains_all(
        "HubTabs.tsx",
        &tabs,
        &[
            "Tab,",
            "Tabs",
            "HubTabOption",
            "variant=\"scrollable\"",
            "scrollButtons=\"auto\"",
            "minHeight: 38",
            "\"& .MuiTabs-indicator\"",
            "iconPosition=\"start\"",
            "\"&.Mui-selected\"",
        ],
    );
}

#[test]
fn checkbox_and_switch_use_mui_form_controls_for_label_detail_rows() {
    let checkbox = read_crate_file("web/src/components/inputs/HubCheckbox.tsx");
    let switch = read_crate_file("web/src/components/inputs/HubSwitch.tsx");

    assert_contains_all(
        "HubCheckbox.tsx",
        &checkbox,
        &[
            "Checkbox",
            "FormControlLabel",
            "HubCheckboxProps",
            "checked: boolean",
            "detail?: string",
            "const isDisabled = disabled || !onChange;",
            "disabled={isDisabled}",
            "onChange?.(event.target.checked)",
            "\"&.Mui-checked\": { color: hubTokens.colors.accent }",
            "color: isDisabled ? hubTokens.colors.textMuted : hubTokens.colors.text",
            "Typography variant=\"body2\" noWrap",
            "Typography variant=\"caption\" noWrap",
            "minHeight: 38",
        ],
    );
    assert_contains_all(
        "HubSwitch.tsx",
        &switch,
        &[
            "Switch",
            "FormControlLabel",
            "HubSwitchProps",
            "checked: boolean",
            "detail?: string",
            "const isDisabled = disabled || !onChange;",
            "disabled={isDisabled}",
            "onChange?.(event.target.checked)",
            "\"& .MuiSwitch-switchBase.Mui-checked\"",
            "color: isDisabled ? hubTokens.colors.textMuted : hubTokens.colors.text",
            "Typography variant=\"body2\" noWrap",
            "Typography variant=\"caption\" noWrap",
            "justifyContent: \"space-between\"",
        ],
    );
}

#[test]
fn routed_pages_consume_input_wrappers_instead_of_raw_material_inputs() {
    for (page, expected_wrappers) in [
        (
            "ProjectsDashboard.tsx",
            vec![
                "HubButton",
                "HubComboBox",
                "HubSearchField",
                "HubSelect",
                "HubTextField",
                "HubToggle",
            ],
        ),
        (
            "ProjectBrowserPage.tsx",
            vec!["HubButton", "HubSearchField", "HubSelect", "HubToggle"],
        ),
        ("ProjectDetailPage.tsx", vec!["HubButton", "HubTabs"]),
        ("BuildsPage.tsx", vec!["HubButton", "HubTabs"]),
        ("CatalogPage.tsx", vec!["HubSearchField", "HubTabs"]),
        (
            "CloudPage.tsx",
            vec!["HubButton", "HubCheckbox", "HubSwitch", "HubTabs"],
        ),
        (
            "EditorPage.tsx",
            vec!["HubButton", "HubCheckbox", "HubSwitch", "HubTabs"],
        ),
        (
            "SettingsPage.tsx",
            vec![
                "HubButton",
                "HubCheckbox",
                "HubComboBox",
                "HubSwitch",
                "HubTabs",
                "HubTextField",
            ],
        ),
        ("TeamPage.tsx", vec!["HubTabs"]),
        (
            "WorkspacePage.tsx",
            vec!["HubButton", "HubCheckbox", "HubSwitch", "HubTabs"],
        ),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(page, &source, &expected_wrappers);

        for import_line in source.lines().filter(|line| line.contains("@mui/material")) {
            assert_not_contains_any(
                page,
                import_line,
                &[
                    "Button",
                    "TextField",
                    "Select",
                    "Autocomplete",
                    "Checkbox",
                    "Switch",
                    "Tabs",
                    "Tab",
                    "ToggleButton",
                    "IconButton",
                ],
            );
        }
        assert_not_contains_any(
            page,
            &source,
            &[
                "<Button ",
                "<TextField",
                "<Select",
                "<Autocomplete",
                "<Checkbox",
                "<Switch",
                "<Tabs",
                "<Tab ",
                "<ToggleButton",
                "<IconButton",
            ],
        );
    }
}

#[test]
fn input_primitives_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_input_primitives_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_input_primitives_contract",
            "## Input Primitive Contract Cutover",
            "React/MUI input primitive system",
            "web/src/components/inputs/HubButton.tsx",
            "web/src/components/inputs/HubSearchField.tsx",
            "web/src/components/inputs/HubCheckbox.tsx",
            "web/src/components/inputs/HubSwitch.tsx",
            "web/src/components/inputs/HubTabs.tsx",
            "web/src/pages",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_input_primitives_contract.rs`",
            "React/MUI input primitive system",
            "button, icon-button, search, text field, select, combo box, checkbox, switch, tabs, and toggle wrappers",
            "pages consume the shared input wrappers",
        ],
    );
}

#[test]
fn input_primitives_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_input_primitives_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_material_typography = format!("Material{}", "Typography");

    assert_contains_all(
        "ui_input_primitives_contract.rs",
        &contract,
        &[
            "web/src/components/inputs/HubButton.tsx",
            "web/src/components/inputs/HubCheckbox.tsx",
            "web/src/components/inputs/HubComboBox.tsx",
            "web/src/components/inputs/HubIconButton.tsx",
            "web/src/components/inputs/HubSearchField.tsx",
            "web/src/components/inputs/HubSelect.tsx",
            "web/src/components/inputs/HubSwitch.tsx",
            "web/src/components/inputs/HubTabs.tsx",
            "web/src/components/inputs/HubTextField.tsx",
            "web/src/components/inputs/HubToggle.tsx",
            "web/src/pages",
        ],
    );
    assert_not_contains_any(
        "ui_input_primitives_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
            old_material_typography.as_str(),
        ],
    );
}
