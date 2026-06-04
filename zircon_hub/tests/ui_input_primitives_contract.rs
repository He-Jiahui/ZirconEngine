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
fn search_box_delegates_placeholder_to_private_text_helper() {
    let text_inputs = read_ui_file("text_input_components.slint");
    let placeholder = text_inputs
        .split("component SearchBoxPlaceholderText")
        .nth(1)
        .and_then(|source| source.split("export component SearchBox").next())
        .expect(
            "text_input_components.slint must declare SearchBoxPlaceholderText before SearchBox",
        );
    for snippet in [
        "inherits MaterialText",
        "in property <string> value;",
        "in property <color> foreground;",
        "text: root.value;",
        "color: root.foreground;",
        "style: MaterialTypography.body_medium;",
        "vertical_alignment: center;",
        "overflow: elide;",
    ] {
        assert!(
            placeholder.contains(snippet),
            "SearchBoxPlaceholderText must own search placeholder typography; missing {snippet}"
        );
    }

    let search_box = text_inputs
        .split("export component SearchBox")
        .nth(1)
        .and_then(|source| source.split("export component HubTextField").next())
        .expect("text_input_components.slint must declare SearchBox before HubTextField");
    for snippet in [
        "search-field := TextInput",
        "StateLayerArea {",
        "if root.text == \"\": SearchBoxPlaceholderText",
        "x: search-field.x;",
        "width: search-field.width;",
        "height: parent.height;",
        "value: root.placeholder;",
        "foreground: root.placeholder-color;",
    ] {
        assert!(
            search_box.contains(snippet),
            "SearchBox must keep text input/focus behavior while delegating placeholder text; missing {snippet}"
        );
    }
    for forbidden in [
        "MaterialText {",
        "text: root.placeholder;",
        "style: MaterialTypography.body_medium;",
    ] {
        assert!(
            !search_box.contains(forbidden),
            "SearchBox should not own direct placeholder MaterialText after helper extraction: {forbidden}"
        );
    }
}

#[test]
fn hub_low_level_state_inputs_wrap_material_checkbox_switch_and_combo() {
    let components = read_ui_file("components.slint");
    for snippet in [
        "HubTextField,",
        "HubPathFieldRow,",
        "SearchBox,",
        "} from \"text_input_components.slint\";",
        "HubSelectTrigger,",
        "} from \"inputs.slint\";",
        "HubCheckBox,",
        "HubCheckBoxRow,",
        "HubSwitch,",
        "HubToggleRow,",
        "HubComboBox,",
        "} from \"input_state_components.slint\";",
    ] {
        assert!(
            components.contains(snippet),
            "components.slint must re-export low-level Material state input wrapper {snippet}"
        );
    }

    let inputs = read_ui_file("inputs.slint");
    let text_inputs = read_ui_file("text_input_components.slint");
    let input_state_components = read_ui_file("input_state_components.slint");
    for snippet in [
        "CheckBox as MaterialCheckBox,",
        "CheckBoxTile as MaterialCheckBoxTile,",
        "Switch as MaterialSwitch,",
        "import { HubSelectDropDownSurface } from \"dropdown_components.slint\";",
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
        "HubSelectDropDownSurface {",
        "select-width: parent.width;",
        "select-height: parent.height;",
        "select-items: root.items;",
        "current_index <=> root.current-index;",
        "selected(index) =>",
    ] {
        assert!(
            input_state_components.contains(snippet),
            "input_state_components.slint must keep checkbox, switch/toggle, and combo box wrappers backed by Material primitives; missing {snippet}"
        );
    }

    for snippet in [
        "export component HubPathFieldRow",
        "HubTextField {",
        "text <=> root.text;",
        "HubCommandButton {",
        "button-width: root.action-width;",
        "button-height: root.action-height;",
        "enabled: root.action-enabled && root.enabled;",
    ] {
        assert!(
            text_inputs.contains(snippet),
            "text_input_components.slint must keep text/path wrappers backed by Material primitives; missing {snippet}"
        );
    }

    for snippet in [
        "import { HubSelectMenu } from \"dropdown_components.slint\";",
        "export component HubSelectTrigger",
        "trigger := OutlineButton {",
        "opacity: 0%;",
        "select-visual := Rectangle {",
        "StateLayerArea {",
        "trailing-chevron := Icon {",
        "source: @image-url(\"../assets/icons/ui/chevron-down.svg\");",
        "callback activated();",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep select wrappers backed by Material primitives; missing {snippet}"
        );
    }

    for forbidden in [
        "component SearchBoxPlaceholderText",
        "export component SearchBox",
        "export component HubTextField",
        "export component HubPathFieldRow",
        "TextField,",
        "HubCommandButton",
        "CheckBox as MaterialCheckBox,",
        "CheckBoxTile as MaterialCheckBoxTile,",
        "Switch as MaterialSwitch,",
        "HubSelectDropDownSurface",
        "export component HubCheckBox",
        "export component HubCheckBoxRow",
        "export component HubSwitch",
        "component HubToggleRowTextStack",
        "export component HubToggleRow",
        "export component HubComboBox",
    ] {
        assert!(
            !inputs.contains(forbidden),
            "inputs.slint must not retain state-control or text-input ownership after focused module splits: {forbidden}"
        );
    }

    for wrapper_name in [
        "HubCheckBox",
        "HubCheckBoxRow",
        "HubSwitch",
        "HubToggleRow",
        "HubComboBox",
    ] {
        let wrapper = input_state_components
            .split(&format!("export component {wrapper_name}"))
            .nth(1)
            .and_then(|source| source.split("export component ").next())
            .unwrap_or_else(|| panic!("input_state_components.slint must declare {wrapper_name}"));
        for forbidden in ["TouchArea", "area.has-hover", "LineEdit"] {
            assert!(
                !wrapper.contains(forbidden),
                "{wrapper_name} must not reintroduce hand-rolled input behavior: {forbidden}"
            );
        }
    }

    for wrapper_name in ["HubPathFieldRow"] {
        let wrapper = text_inputs
            .split(&format!("export component {wrapper_name}"))
            .nth(1)
            .and_then(|source| source.split("export component ").next())
            .unwrap_or_else(|| panic!("text_input_components.slint must declare {wrapper_name}"));
        for forbidden in ["TouchArea", "area.has-hover", "LineEdit"] {
            assert!(
                !wrapper.contains(forbidden),
                "{wrapper_name} must not reintroduce hand-rolled input behavior: {forbidden}"
            );
        }
    }

    for wrapper_name in ["HubSelectTrigger"] {
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
fn hub_toggle_row_delegates_text_to_private_stack() {
    let input_state_components = read_ui_file("input_state_components.slint");
    let text_stack = input_state_components
        .split("component HubToggleRowTextStack")
        .nth(1)
        .and_then(|source| source.split("export component HubToggleRow").next())
        .expect(
            "input_state_components.slint must declare HubToggleRowTextStack before HubToggleRow",
        );
    for snippet in [
        "inherits VerticalLayout",
        "in property <string> label;",
        "in property <string> supporting-text;",
        "horizontal-stretch: 1;",
        "spacing: MaterialStyleMetrics.spacing_2;",
        "text: root.label;",
        "text: root.supporting-text;",
        "color: MaterialPalette.on_surface;",
        "color: MaterialPalette.on_surface_variant;",
        "style: MaterialTypography.label_large;",
        "style: MaterialTypography.body_small;",
        "horizontal_alignment: left;",
        "overflow: elide;",
    ] {
        assert!(
            text_stack.contains(snippet),
            "HubToggleRowTextStack must own toggle-row label/supporting typography; missing {snippet}"
        );
    }
    assert_eq!(
        text_stack.matches("MaterialText {").count(),
        2,
        "HubToggleRowTextStack should own the label and optional supporting text nodes"
    );

    let toggle_row = input_state_components
        .split("export component HubToggleRow")
        .nth(1)
        .and_then(|source| source.split("export component HubComboBox").next())
        .expect("input_state_components.slint must declare HubToggleRow before HubComboBox");
    for snippet in [
        "HubToggleRowTextStack {",
        "label: root.label;",
        "supporting-text: root.supporting-text;",
        "HubSwitch {",
        "checked <=> root.checked;",
        "enabled: root.enabled;",
        "toggled(checked) =>",
    ] {
        assert!(
            toggle_row.contains(snippet),
            "HubToggleRow must delegate label copy to HubToggleRowTextStack and keep switch forwarding; missing {snippet}"
        );
    }
    for forbidden in [
        "MaterialText {",
        "style: MaterialTypography.label_large;",
        "style: MaterialTypography.body_small;",
    ] {
        assert!(
            !toggle_row.contains(forbidden),
            "HubToggleRow should not own direct label/supporting text after helper extraction: {forbidden}"
        );
    }
    for forbidden_line in ["text: root.label;", "text: root.supporting-text;"] {
        assert!(
            !toggle_row
                .lines()
                .any(|line| line.trim() == forbidden_line),
            "HubToggleRow should not own direct label/supporting text after helper extraction: {forbidden_line}"
        );
    }
}

#[test]
fn hub_select_trigger_delegates_visible_label_to_private_text_helper() {
    let inputs = read_ui_file("inputs.slint");
    let trigger_label = inputs
        .split("component HubSelectTriggerLabel")
        .nth(1)
        .and_then(|source| source.split("export component HubSelectTrigger").next())
        .expect("inputs.slint must declare HubSelectTriggerLabel before HubSelectTrigger");
    for snippet in [
        "inherits MaterialText",
        "in property <string> value;",
        "in property <color> foreground;",
        "in property <bool> dense: false;",
        "horizontal-stretch: 1;",
        "min-width: 1px;",
        "text: root.value;",
        "color: root.foreground;",
        "style: root.dense ? MaterialTypography.label_medium : MaterialTypography.label_large;",
        "vertical_alignment: center;",
        "overflow: elide;",
    ] {
        assert!(
            trigger_label.contains(snippet),
            "HubSelectTriggerLabel must own visible select-trigger label typography; missing {snippet}"
        );
    }

    let select_trigger = inputs
        .split("export component HubSelectTrigger")
        .nth(1)
        .and_then(|source| source.split("export component ToolbarSelect").next())
        .expect("inputs.slint must declare HubSelectTrigger before ToolbarSelect");
    for snippet in [
        "trigger := OutlineButton",
        "text: root.text;",
        "select-visual := Rectangle",
        "HubSelectTriggerLabel {",
        "value: root.text;",
        "foreground: root.select-foreground;",
        "dense: root.dense-label;",
        "StateLayerArea {",
        "trailing-chevron := Icon",
    ] {
        assert!(
            select_trigger.contains(snippet),
            "HubSelectTrigger must keep Material anchor behavior while delegating visible text; missing {snippet}"
        );
    }
    for forbidden in [
        "MaterialText {",
        "style: root.dense-label ? MaterialTypography.label_medium : MaterialTypography.label_large;",
        "color: root.select-foreground;",
    ] {
        assert!(
            !select_trigger.contains(forbidden),
            "HubSelectTrigger should not own direct visible label text after helper extraction: {forbidden}"
        );
    }
}

#[test]
fn toolbar_select_and_combo_share_select_dropdown_surface_contracts() {
    let inputs = read_ui_file("inputs.slint");
    let input_state_components = read_ui_file("input_state_components.slint");

    for snippet in [
        "export component HubSelectTrigger",
        "export component ToolbarSelect",
        "HubSelectTrigger {",
        "trigger-width: parent.width;",
        "trigger-height: parent.height;",
        "menu-ready: root.menu-ready;",
        "activated =>",
        "menu := HubSelectMenu {",
        "anchor-width: root.select-width;",
        "anchor-height: root.height;",
        "select-items: root.menu-items;",
        "activated(index) =>",
    ] {
        assert!(
            inputs.contains(snippet),
            "ToolbarSelect must route select/dropdown behavior through shared Hub select surfaces; missing {snippet}"
        );
    }

    for snippet in [
        "export component HubComboBox",
        "material-combo := HubSelectDropDownSurface {",
        "select-width: parent.width;",
        "select-height: parent.height;",
        "select-items: root.items;",
        "current_index <=> root.current-index;",
    ] {
        assert!(
            input_state_components.contains(snippet),
            "HubComboBox must route select/dropdown behavior through the shared Hub select surface; missing {snippet}"
        );
    }

    let toolbar_select = inputs
        .split("export component ToolbarSelect")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .unwrap_or_else(|| panic!("inputs.slint must declare ToolbarSelect"));
    for forbidden in [
        "trigger := OutlineButton",
        "select-visual := Rectangle",
        "trailing-chevron := Icon",
        "StateLayerArea {",
        "menu := HubPopupMenu",
        "private property <length> menu-width:",
        "private property <length> menu-offset-x:",
    ] {
        assert!(
            !toolbar_select.contains(forbidden),
            "ToolbarSelect must not keep page-local popup geometry after HubSelectMenu extraction: {forbidden}"
        );
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
        "HubRowSelectionSlot,",
        "HubRowMainSlot,",
        "HubRowTrailingSlot,",
        "private property <CheckState> selection-state: root.template.selected ? CheckState.checked : CheckState.unchecked;",
        "export component TemplateChoiceRow inherits HubInteractiveRowSurface",
        "interaction-enabled: root.template.enabled;",
        "interaction-foreground: root.template.selected ? HubVisualSpec.accent-stroke : MaterialPalette.on_surface;",
        "clicked =>",
        "HubRowSelectionSlot {",
        "check-state: root.selection-state;",
        "changed(state) =>",
        "template-selected(id) => { root.selected(id); }",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "badge-text: root.trailing-label;",
        "root.template-selected(root.template.id);",
    ] {
        assert!(
            project_components.contains(snippet) || template_row.contains(snippet),
            "Project template choices must consume shared row selection/main/trailing slots backed by Material primitives; missing {snippet}"
        );
    }

    for forbidden in [
        "HubCheckBox {",
        "MaterialCheckBox {",
        "MaterialCheckBoxTile {",
        "TouchArea",
        "StateLayerArea {",
    ] {
        assert!(
            !template_row.contains(forbidden),
            "TemplateChoiceRow must not bypass HubRowSelectionSlot with raw or hand-rolled selection controls: {forbidden}"
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

    let action_stack = detail_components
        .split("export component ProjectDetailActionStack")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailDeleteActionStack").next())
        .unwrap_or_else(|| {
            panic!("project_detail_components.slint must declare ProjectDetailActionStack before ProjectDetailDeleteActionStack")
        });
    let actions_section = detail_components
        .split("export component ProjectDetailActionsSection")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailStatusStrip").next())
        .unwrap_or_else(|| {
            panic!("project_detail_components.slint must declare ProjectDetailActionsSection before ProjectDetailStatusStrip")
        });

    for snippet in [
        "ProjectDetailActionsSection,",
        "ProjectDetailActionsSection {",
        "pin-toggle-row-height: root.pin-toggle-row-height;",
        "project: root.project;",
        "copy: root.ui-text;",
        "toggle-pin => { root.toggle-pin(); }",
    ] {
        assert!(
            detail_page.contains(snippet),
            "ProjectDetailPage must route pin/unpin through ProjectDetailActionsSection; missing {snippet}"
        );
    }

    for snippet in [
        "ProjectDetailActionStack {",
        "pin-toggle-row-height: root.pin-toggle-row-height;",
        "toggle-pin => { root.toggle-pin(); }",
    ] {
        assert!(
            actions_section.contains(snippet),
            "ProjectDetailActionsSection must delegate standard action content to ProjectDetailActionStack; missing {snippet}"
        );
    }

    for snippet in [
        "ProjectDetailPinToggleRow {",
        "row-height: root.pin-toggle-row-height;",
        "detail: root.project;",
        "copy: root.copy;",
        "toggled(checked) => { root.toggle-pin(); }",
    ] {
        assert!(
            action_stack.contains(snippet),
            "ProjectDetailActionStack must route pin/unpin through ProjectDetailPinToggleRow; missing {snippet}"
        );
    }

    for forbidden in [
        "text: root.project.pinned ? root.ui-text.unpin-project : root.ui-text.pin-project;",
        "clicked => { root.toggle-pin(); }",
    ] {
        assert!(
            !action_stack.contains(forbidden),
            "Project Detail pin state should not remain a generic action button: {forbidden}"
        );
    }
}
