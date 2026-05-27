//! Static contracts for Zircon Hub input primitives.

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
fn shared_hub_buttons_are_backed_by_material_button_primitives() {
    let shared = read_ui_file("shared.slint");
    for snippet in [
        "FilledButton,",
        "FilledIconButton,",
        "IconButton as MaterialIconButton,",
        "OutlineButton,",
        "TonalButton,",
        "TonalIconButton,",
        "export component PillButton",
        "FilledButton {",
        "TonalButton {",
        "export component IconButton",
        "FilledIconButton {",
        "TonalIconButton {",
        "export component WindowButton",
        "MaterialIconButton {",
    ] {
        assert!(
            shared.contains(snippet),
            "shared.slint must keep Hub button APIs backed by Material button primitives; missing {snippet}"
        );
    }

    let pill_start = shared
        .find("export component PillButton")
        .expect("shared.slint must declare PillButton");
    let icon_start = shared
        .find("export component IconButton")
        .expect("shared.slint must declare IconButton");
    let nav_start = shared
        .find("export component NavButton")
        .expect("shared.slint must declare NavButton after IconButton");
    let pill_button = &shared[pill_start..icon_start];
    let icon_button = &shared[icon_start..nav_start];
    for snippet in [
        "height: MaterialStyleMetrics.size_40;",
        "min-width: root.height * 3;",
        "preferred-width: root.height * 4;",
        "clip: true;",
    ] {
        assert!(
            pill_button.contains(snippet),
            "PillButton must derive Material text button geometry from Material metrics and proportions; missing {snippet}"
        );
    }
    assert!(
        !pill_button.contains("preferred-width: 150px;"),
        "PillButton must not return to the old fixed-width wrapper"
    );
    assert!(
        icon_button.contains("clip: true;"),
        "Hub IconButton must clip Material icon buttons to the requested atom size in compact rows"
    );
    assert!(
        !pill_button.contains("TouchArea") && !icon_button.contains("TouchArea"),
        "PillButton and IconButton should delegate pointer behavior to Material buttons instead of hand-rolled TouchArea layers"
    );

    let window_start = shared
        .find("export component WindowButton")
        .expect("shared.slint must declare WindowButton");
    let panel_start = shared
        .find("export component Panel")
        .expect("shared.slint must declare Panel after WindowButton");
    let window_button = &shared[window_start..panel_start];
    for snippet in [
        "MaterialIconButton {",
        "icon: root.has-icon-image ? root.icon-image : @image-url(\"../assets/icons/ui/close.svg\");",
        "inline: true;",
        "has_error: root.danger;",
        "clicked =>",
    ] {
        assert!(
            window_button.contains(snippet),
            "WindowButton must delegate title-bar icon layout and interaction to Material IconButton; missing {snippet}"
        );
    }
    for forbidden in ["CenteredIcon", "area := TouchArea", "area.has-hover"] {
        assert!(
            !window_button.contains(forbidden),
            "WindowButton should not return to custom painted title-bar icon behavior: {forbidden}"
        );
    }
}

#[test]
fn hub_form_text_inputs_use_material_text_field_wrapper() {
    let components = read_ui_file("components.slint");
    assert!(
        components.contains("HubTextField"),
        "components.slint must re-export the Hub Material-backed text field wrapper"
    );

    let inputs = read_ui_file("inputs.slint");
    for snippet in [
        "TextField",
        "MaterialStyleMetrics",
        "export component HubTextField",
        "material-field := TextField",
        "placeholder_text:",
        "text <=> root.text;",
        "height: HubTokens.input-field;",
        "preferred-width: HubTokens.input-width;",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep HubTextField backed by Material TextField; missing {snippet}"
        );
    }

    for page in [
        "settings.slint",
        "editor.slint",
        "project_page_components.slint",
    ] {
        let source = read_ui_file(page);
        assert!(
            source.contains("HubTextField"),
            "{page} form fields must use the HubTextField wrapper"
        );
        assert!(
            !source.contains("LineEdit"),
            "{page} should not reintroduce std-widgets LineEdit now that HubTextField owns Material input behavior"
        );
    }

    let settings = read_ui_file("settings.slint");
    for snippet in [
        "component SettingsToolchainField inherits HubTextField",
        "in property <string> field-label;",
        "in-out property <string> field-value;",
        "label: root.field-label;",
        "placeholder: root.field-label;",
        "text <=> root.field-value;",
        "SettingsToolchainField {",
        "field-label: root.ui-text.python-executable;",
        "field-value <=> root.python-path;",
        "field-label: root.ui-text.cargo-executable;",
        "field-value <=> root.cargo-path;",
        "field-label: root.ui-text.rustup-executable;",
        "field-value <=> root.rustup-path;",
    ] {
        assert!(
            settings.contains(snippet),
            "SettingsPage toolchain fields should use one local HubTextField wrapper while preserving bindings: {snippet}"
        );
    }
    assert_eq!(
        settings.matches("SettingsToolchainField {").count(),
        3,
        "SettingsPage should render python/cargo/rustup paths through SettingsToolchainField"
    );

    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let editor_controls = format!("{editor}\n{editor_components}");
    for snippet in [
        "export component EditorPathFieldRow inherits Rectangle",
        "in property <string> field-label;",
        "in-out property <string> field-text;",
        "callback button-clicked();",
        "root.button-clicked();",
        "EditorPathFieldRow {",
        "field-label: root.ui-text.active-engine-name;",
        "field-text <=> root.active-engine-name;",
        "button-text: root.ui-text.rename;",
        "root.rename-active-engine(root.active-engine-name);",
        "field-label: root.ui-text.source-checkout-path;",
        "field-text <=> root.source-path;",
        "root.browse-folder(\"source\");",
        "field-label: root.ui-text.staged-output-directory;",
        "field-text <=> root.output-path;",
        "root.browse-folder(\"output\");",
    ] {
        assert!(
            editor_controls.contains(snippet),
            "EditorPage Source Engine settings rows should use one exported HubTextField/PillButton row component while preserving bindings and actions: {snippet}"
        );
    }
    assert!(
        !editor.contains("component EditorPathFieldRow"),
        "editor.slint should import EditorPathFieldRow instead of defining it inline"
    );

    let project_pages = read_ui_file("project_pages.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_create_controls = format!("{project_pages}\n{project_components}");
    assert!(
        project_pages.contains("field-height: HubTokens.input-field;"),
        "ProjectNewPage should derive form field height from HubTokens.input-field"
    );
    for snippet in [
        "component ProjectCreateField inherits Rectangle",
        "in property <string> field-label;",
        "in property <string> field-placeholder;",
        "in-out property <string> field-text;",
        "in property <bool> show-browse: false;",
        "callback browse-clicked();",
        "height: root.field-height;",
        "label: root.field-label;",
        "placeholder: root.field-placeholder;",
        "text <=> root.field-text;",
        "if root.show-browse: PillButton",
        "root.browse-clicked();",
        "ProjectCreateField {",
        "field-label: root.ui-text.project-name;",
        "field-text <=> root.project-name;",
        "field-label: root.ui-text.location;",
        "field-text <=> root.project-location;",
        "show-browse: true;",
        "root.browse-folder(\"new-project-location\");",
        "component ProjectCreateActionRow inherits Rectangle",
        "in property <string> action-label;",
        "in property <bool> action-enabled;",
        "callback action-clicked();",
        "PillButton {",
        "text: root.action-label;",
        "icon-image: @image-url(\"../assets/icons/ui/plus.svg\");",
        "enabled: root.action-enabled;",
        "clicked => { root.action-clicked(); }",
        "ProjectCreateActionRow {",
        "row-height: root.create-action-row-height;",
        "row-spacing: root.page-gap;",
        "action-label: root.ui-text.create;",
        "action-enabled: root.form-ready;",
        "action-clicked => { root.create-project(); }",
    ] {
        assert!(
            project_create_controls.contains(snippet),
            "ProjectNewPage create fields should use one local HubTextField/PillButton row wrapper while preserving bindings and browse behavior: {snippet}"
        );
    }
    for component_name in ["ProjectCreateField", "ProjectCreateActionRow"] {
        assert!(
            project_components.contains(&format!("export component {component_name}")),
            "project_page_components.slint should own {component_name} after Projects workflow component extraction"
        );
        assert!(
            !project_pages.contains(&format!("component {component_name} inherits")),
            "project_pages.slint should import {component_name} instead of declaring it locally"
        );
    }
    assert_eq!(
        project_pages.matches("ProjectCreateField {").count(),
        2,
        "ProjectNewPage should render project name and location through ProjectCreateField"
    );
    assert_eq!(
        project_pages.matches("ProjectCreateActionRow {").count(),
        1,
        "ProjectNewPage should render the create action through ProjectCreateActionRow"
    );
    assert!(
        project_pages.contains(
            "summary-row-height: max(HubTokens.control-sm, min(root.field-height, root.content-height / 18));"
        ),
        "ProjectNewPage create summary should stay compact enough to keep core create controls visible without depending on flow-height"
    );
    for snippet in [
        "create-action-row-height: root.field-height;",
        "height: root.create-action-row-height;",
        "form-panel-height: HubTokens.space-4 * 2 + HubTokens.list-row-sm + root.field-height * 2 + root.engine-section-height + root.create-action-row-height + root.page-gap * 4;",
        "summary-panel-padding: root.narrow-flow ? HubTokens.space-3 : HubTokens.space-4;",
        "summary-panel-height: root.summary-panel-padding * 2 + root.summary-section-height;",
        "project-settings-panel-height: root.narrow-flow ? root.form-panel-height : root.form-panel-height + root.summary-section-height + root.page-gap;",
        "template-panel-rows: root.template-count < 1 ? 1 : (root.template-count > 4 ? 4 : root.template-count);",
        "template-list-height: root.template-count == 0 ? HubTokens.list-row-lg : root.template-panel-rows * root.template-row-height + (root.template-panel-rows - 1) * root.page-gap;",
        "template-panel-height: HubTokens.space-4 * 2 + HubTokens.control-md + root.template-list-height + root.page-gap;",
        "template-scroll-y: 0px;",
        "page-gap: root.compact-page ? HubTokens.toolbar-gap : HubTokens.panel-gap;",
        "summary-header-height: root.narrow-flow ? HubTokens.control-md : HubTokens.list-row-sm;",
        "subtitle: root.narrow-flow ? \"\" : root.summary-subtitle;",
        "alignment: center;",
        "visible: root.narrow-flow;",
        "visible: !root.narrow-flow;",
    ] {
        assert!(
            project_pages.contains(snippet),
            "ProjectNewPage form rows and create action should align from shared Material control metrics instead of stretched offsets; missing {snippet}"
        );
    }
    assert!(
        project_pages.contains("section-label-height: MaterialTypography.body_small.font_size * 3 / 2;")
            && project_pages.contains(
                "engine-panel-rows: root.engine-count < 1 ? 1 : (root.engine-count > 3 ? 3 : root.engine-count);"
            )
            && project_pages.contains(
                "engine-list-height: root.engine-count == 0 ? root.choice-row-height : root.engine-panel-rows * root.choice-row-height + (root.engine-panel-rows - 1) * root.engine-row-gap;"
            )
            && project_pages.contains(
                "engine-section-height: root.section-label-height + MaterialStyleMetrics.spacing_8 + root.engine-list-height;"
            )
            && project_pages.contains("height: root.engine-section-height;")
            && project_pages.contains("ProjectEngineChoiceList {")
            && project_pages.contains("list-height: root.engine-list-height;")
            && project_pages.contains("list-scroll-y <=> root.new-engine-scroll-y;"),
        "ProjectNewPage source-engine selector should size from Material text and capped row metrics instead of stretching with every engine"
    );
    for snippet in [
        "private property <bool> project-name-ready: root.project-name != \"\";",
        "private property <bool> project-location-ready: root.project-location != \"\";",
        "private property <bool> form-ready: root.create-enabled && root.project-name-ready && root.project-location-ready;",
        "enabled: root.form-ready;",
        "value: root.form-ready ? root.ready-label : root.complete-label;",
        "badge-tone: root.form-ready ? \"accent\" : \"warning\";",
    ] {
        assert!(
            project_create_controls.contains(snippet),
            "ProjectNewPage create controls must validate name, location, template, and Source Engine before showing a ready state; missing {snippet}"
        );
    }
    for forbidden in [
        "value: root.selected-engine-label;",
        "value: root.selected-template-label;",
    ] {
        assert!(
            !project_pages.contains(forbidden),
            "ProjectNewPage should not duplicate engine/template selections in the compact create summary: {forbidden}"
        );
    }
}

#[test]
fn hub_search_box_uses_material_search_bar_wrapper() {
    let inputs = read_ui_file("inputs.slint");
    let search_box = inputs
        .split("export component SearchBox")
        .nth(1)
        .and_then(|source| source.split("export component HubTextField").next())
        .expect("inputs.slint must declare SearchBox before HubTextField");

    for snippet in [
        "in property <length> box-height: HubTokens.input-field;",
        "search-field := SearchBar",
        "placeholder_text: root.placeholder;",
        "leading_icon: @image-url(\"../assets/icons/ui/search.svg\");",
        "empty_text: \"\";",
        "text <=> root.text;",
        "height: root.box-height;",
        "edited(value) =>",
        "accepted(value) =>",
    ] {
        assert!(
            search_box.contains(snippet),
            "SearchBox must stay backed by the imported Material SearchBar wrapper; missing {snippet}"
        );
    }

    for forbidden in [
        "TextInput",
        "border-color: search-field.has-focus",
        "selection-background-color",
        "CenteredIcon",
        "search-field := TextField",
        "label: root.placeholder",
    ] {
        assert!(
            !search_box.contains(forbidden),
            "SearchBox should not return to a custom painted TextInput implementation: {forbidden}"
        );
    }

    for page in ["project_dashboard.slint", "project_pages.slint"] {
        let source = read_ui_file(page);
        assert!(
            source.contains("box-height: root.toolbar-height;"),
            "{page} must size SearchBox through the responsive toolbar height"
        );
    }
}

#[test]
fn hub_segment_button_uses_material_segmented_button() {
    let inputs = read_ui_file("inputs.slint");
    let segment = inputs
        .split("export component SegmentButton")
        .nth(1)
        .expect("inputs.slint must declare SegmentButton");
    for snippet in [
        "SegmentedButton",
        "export component SegmentButton",
        "material-segment := SegmentedButton",
        "current_index <=> root.selected-index;",
        "items: [{ text: root.text }];",
        "index_changed(index) =>",
    ] {
        assert!(
            inputs.contains(snippet),
            "SegmentButton must stay backed by the imported Material SegmentedButton; missing {snippet}"
        );
    }
    for forbidden in [
        "border-color: root.active",
        "background: root.active",
        "area := TouchArea",
    ] {
        assert!(
            !segment.contains(forbidden),
            "SegmentButton should not return to a custom painted toggle implementation: {forbidden}"
        );
    }
}

#[test]
fn hub_toolbar_select_uses_material_menu_primitives() {
    let inputs = read_ui_file("inputs.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let toolbar_select = inputs
        .split("export component ToolbarSelect")
        .nth(1)
        .and_then(|source| source.split("export component DropDownButton").next())
        .expect("inputs.slint must declare ToolbarSelect before DropDownButton");
    for snippet in [
        "MenuItem",
        "OutlineButton",
        "PopupMenu",
        "in property <length> select-height: HubTokens.control-md;",
        "in property <[MenuItem]> menu-items: [];",
        "private property <length> menu-width: max(root.select-width, HubTokens.input-width / 2);",
        "private property <length> menu-offset-x: min(0px, root.select-width - root.menu-width);",
        "private property <length> trailing-icon-size: max(MaterialStyleMetrics.icon_size_18, min(MaterialStyleMetrics.icon_size_24, root.select-height * 2 / 5));",
        "private property <length> trailing-icon-inset: max(MaterialStyleMetrics.padding_12, root.select-height / 4);",
        "clip: false;",
        "trigger := OutlineButton",
        "trailing-chevron := Icon",
        "x: parent.width - root.trailing-icon-inset - self.width;",
        "y: (parent.height - self.height) / 2;",
        "source: @image-url(\"../assets/icons/ui/chevron-down.svg\");",
        "colorize: MaterialPalette.on_surface_variant;",
        "menu := PopupMenu",
        "x: root.menu-offset-x;",
        "width: root.menu-width;",
        "items: root.menu-items;",
        "root.selected(root.options[index].id);",
    ] {
        assert!(
            inputs.contains(snippet),
            "ToolbarSelect must stay backed by imported Material menu/button primitives; missing {snippet}"
        );
    }
    for forbidden in [
        "SelectOptionRow",
        "popup := PopupWindow",
        "area := TouchArea",
        "callback clicked;",
        "root.clicked();",
    ] {
        assert!(
            !toolbar_select.contains(forbidden),
            "ToolbarSelect should stay menu-only instead of acting as a direct-toggle button: {forbidden}"
        );
    }
    for snippet in [
        "export component ProjectFilterSelect",
        "export component ProjectSortSelect",
        "ToolbarSelect {",
        "menu-items: [",
        "text: root.ui-text.last-modified-column",
        "text: root.ui-text.name-column",
        "selected(id) => { root.selected(id); }",
    ] {
        assert!(
            project_components.contains(snippet),
            "project_page_components.slint must own the shared Projects filter/sort menu shell; missing {snippet}"
        );
    }
    for page in ["project_dashboard.slint", "project_pages.slint"] {
        let source = read_ui_file(page);
        assert!(
            source.contains("ProjectFilterSelect {") && source.contains("ProjectSortSelect {"),
            "{page} must reuse the shared Projects filter/sort select wrappers"
        );
        assert!(
            source.contains("select-height: root.toolbar-height;"),
            "{page} must align project select wrappers to the same responsive toolbar height as SearchBox"
        );
        assert!(
            !source.contains("ToolbarSelect"),
            "{page} should not duplicate raw ToolbarSelect menu construction"
        );
        assert!(
            !source.contains("root.set-project-sort(\""),
            "{page} must not directly toggle project sort options from a button click"
        );
    }
}

#[test]
fn hub_dropdown_button_uses_material_button_primitives() {
    let inputs = read_ui_file("inputs.slint");
    let dropdown = inputs
        .split("export component DropDownButton")
        .nth(1)
        .and_then(|source| source.split("export component SegmentButton").next())
        .expect("inputs.slint must declare DropDownButton before SegmentButton");
    for snippet in [
        "OutlineButton",
        "TonalButton",
        "export component DropDownButton",
        "if root.active: TonalButton",
        "if !root.active: OutlineButton",
        "icon: root.icon-image;",
    ] {
        assert!(
            inputs.contains(snippet),
            "DropDownButton must stay backed by imported Material button primitives; missing {snippet}"
        );
    }
    for forbidden in [
        "border-color: root.active",
        "background: root.active",
        "area := TouchArea",
    ] {
        assert!(
            !dropdown.contains(forbidden),
            "DropDownButton should not return to a custom painted button implementation: {forbidden}"
        );
    }
}
