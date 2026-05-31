//! Static contracts for low-level Hub Material input primitives.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
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

#[test]
fn hub_low_level_state_inputs_wrap_material_checkbox_switch_and_combo() {
    let components = read_ui_file("components.slint");
    for snippet in [
        "HubCheckBox,",
        "HubCheckBoxRow,",
        "HubSwitch,",
        "HubToggleRow,",
        "HubComboBox,",
    ] {
        assert!(
            components.contains(snippet),
            "components.slint must re-export low-level Material state input wrapper {snippet}"
        );
    }

    let inputs = read_ui_file("inputs.slint");
    for snippet in [
        "CheckBox as MaterialCheckBox,",
        "CheckBoxTile as MaterialCheckBoxTile,",
        "Switch as MaterialSwitch,",
        "import { HubDropDownSurface, HubPopupMenu } from \"overlays.slint\";",
        "export component HubCheckBox",
        "check_state <=> root.check-state;",
        "checked_state_changed(state) =>",
        "export component HubCheckBoxRow",
        "MaterialCheckBoxTile {",
        "text: root.label;",
        "supporting_text: root.supporting-text;",
        "export component HubSwitch",
        "MaterialSwitch {",
        "checked <=> root.checked;",
        "checked_state_changed(checked) =>",
        "export component HubToggleRow",
        "HubSwitch {",
        "horizontal_alignment: left;",
        "toggled(checked) =>",
        "export component HubComboBox",
        "HubDropDownSurface {",
        "dropdown-width: parent.width;",
        "dropdown-height: parent.height;",
        "dropdown-items: root.items;",
        "current_index <=> root.current-index;",
        "selected(index) =>",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep checkbox, switch/toggle, and combo box wrappers backed by Material primitives; missing {snippet}"
        );
    }

    for wrapper_name in [
        "HubCheckBox",
        "HubCheckBoxRow",
        "HubSwitch",
        "HubToggleRow",
        "HubComboBox",
    ] {
        let wrapper = inputs
            .split(&format!("export component {wrapper_name}"))
            .nth(1)
            .and_then(|source| source.split("export component ").next())
            .unwrap_or_else(|| panic!("inputs.slint must declare {wrapper_name}"));
        for forbidden in ["TouchArea", "area.has-hover", "LineEdit"] {
            assert!(
                !wrapper.contains(forbidden),
                "{wrapper_name} must not reintroduce hand-rolled input behavior: {forbidden}"
            );
        }
    }
}

#[test]
fn settings_build_defaults_consume_hub_combobox_choices() {
    let settings_components = read_ui_file("settings_page_components.slint");

    for snippet in [
        "HubComboBox,",
        "MenuItem,",
        "export component SettingsComboChoice inherits Rectangle",
        "private property <int> desired-index: root.selected-value == root.second-value ? 1 : 0;",
        "private property <int> selected-index: -1;",
        "private property <[MenuItem]> choice-items:",
        "HubComboBox {",
        "items: root.choice-items;",
        "current-index: root.selected-index;",
        "root.selected-index = index;",
        "root.selected-value = index == 1 ? root.second-value : root.first-value;",
        "init =>",
        "root.selected-index = root.desired-index;",
        "changed selected-value =>",
    ] {
        assert!(
            settings_components.contains(snippet),
            "Settings build defaults must consume the shared HubComboBox primitive; missing {snippet}"
        );
    }

    assert_eq!(
        settings_components
            .matches("SettingsComboChoice {")
            .count(),
        2,
        "SettingsBuildDefaultsPanel should render build profile and language through SettingsComboChoice"
    );

    for forbidden in [
        "export component SettingsSegmentChoice",
        "SegmentButton,",
        "SegmentButton {",
    ] {
        assert!(
            !settings_components.contains(forbidden),
            "Settings build defaults should not keep page-local segment-choice controls: {forbidden}"
        );
    }
}

#[test]
fn project_template_choices_consume_hub_checkbox_selection() {
    let project_components = read_ui_file("project_page_components.slint");
    let template_row = project_components
        .split("export component TemplateChoiceRow")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .unwrap_or_else(|| panic!("project_page_components.slint must declare TemplateChoiceRow"));

    for snippet in [
        "CheckState,",
        "HubCheckBox,",
        "private property <CheckState> selection-state: root.template.selected ? CheckState.checked : CheckState.unchecked;",
        "StateLayerArea {",
        "HubCheckBox {",
        "check-state: root.selection-state;",
        "changed(state) =>",
        "root.selected(root.template.id);",
        "StatusBadge {",
    ] {
        assert!(
            project_components.contains(snippet) || template_row.contains(snippet),
            "Project template choices must consume the shared Material checkbox primitive; missing {snippet}"
        );
    }

    for forbidden in ["MaterialCheckBox {", "MaterialCheckBoxTile {", "TouchArea"] {
        assert!(
            !template_row.contains(forbidden),
            "TemplateChoiceRow must not bypass HubCheckBox with raw or hand-rolled selection controls: {forbidden}"
        );
    }
}

#[test]
fn project_detail_pin_state_consumes_hub_toggle_row() {
    let detail_components = read_ui_file("project_detail_components.slint");
    let detail_page = read_ui_file("project_detail_page.slint");
    let pin_toggle = detail_components
        .split("export component ProjectDetailPinToggleRow")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .unwrap_or_else(|| {
            panic!("project_detail_components.slint must declare ProjectDetailPinToggleRow")
        });

    for snippet in [
        "HubToggleRow,",
        "export component ProjectDetailPinToggleRow inherits HubToggleRow",
        "checked: root.detail.pinned;",
        "label: root.detail.pinned ? root.copy.pinned-label : root.copy.not-pinned-label;",
        "supporting-text: root.detail.pinned ? root.copy.unpin-project : root.copy.pin-project;",
    ] {
        assert!(
            detail_components.contains(snippet) || pin_toggle.contains(snippet),
            "Project Detail pin state must consume the shared Material toggle primitive; missing {snippet}"
        );
    }

    for snippet in [
        "ProjectDetailPinToggleRow,",
        "ProjectDetailPinToggleRow {",
        "row-height: root.pin-toggle-row-height;",
        "detail: root.project;",
        "copy: root.ui-text;",
        "toggled(checked) => { root.toggle-pin(); }",
    ] {
        assert!(
            detail_page.contains(snippet),
            "ProjectDetailPage must route pin/unpin through ProjectDetailPinToggleRow; missing {snippet}"
        );
    }

    let standard_actions = detail_page
        .split("if !root.project.pending-delete: ProjectDetailPinToggleRow")
        .nth(1)
        .and_then(|source| source.split("if !root.project.pending-delete: ProjectDetailEngineSection").next())
        .unwrap_or_else(|| panic!("ProjectDetailPage must render ProjectDetailPinToggleRow before ProjectDetailEngineSection"));
    for forbidden in [
        "text: root.project.pinned ? root.ui-text.unpin-project : root.ui-text.pin-project;",
        "clicked => { root.toggle-pin(); }",
    ] {
        assert!(
            !standard_actions.contains(forbidden),
            "Project Detail pin state should not remain a generic action button: {forbidden}"
        );
    }
}
