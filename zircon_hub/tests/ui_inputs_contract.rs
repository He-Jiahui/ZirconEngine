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
    let button_components = read_ui_file("button_components.slint");
    let components = read_ui_file("components.slint");
    for snippet in [
        "FilledButton,",
        "FilledIconButton,",
        "IconButton as MaterialIconButton,",
        "OutlineButton,",
        "OutlineIconButton,",
        "TonalButton,",
        "TonalIconButton,",
        "StateLayerArea,",
        "export component PillButton",
        "FilledButton {",
        "TonalButton {",
        "export component HubCommandButton",
        "export component IconButton",
        "FilledIconButton {",
        "OutlineIconButton {",
        "export component HubIconButton",
        "if root.focused: Rectangle",
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
    let command_start = shared
        .find("export component HubCommandButton")
        .expect("shared.slint must declare HubCommandButton before IconButton");
    let hub_icon_start = shared
        .find("export component HubIconButton")
        .expect("shared.slint must declare HubIconButton after IconButton");
    let nav_start = shared
        .find("export component NavButton")
        .expect("shared.slint must declare NavButton after HubIconButton");
    let pill_button = &shared[pill_start..command_start];
    let command_button = &shared[command_start..icon_start];
    let icon_button = &shared[icon_start..hub_icon_start];
    let hub_icon_button = &shared[hub_icon_start..nav_start];
    for snippet in [
        "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
        "height: MaterialStyleMetrics.size_40;",
        "min-width: root.height * 3;",
        "preferred-width: root.height * 4;",
        "in property <image> fallback-icon-image: @image-url(\"../assets/icons/ui/alert.svg\");",
        "private property <length> focus-radius: root.height / 2;",
        "icon: root.has-icon-image ? root.icon-image : root.fallback-icon-image;",
        "if (root.enabled) {",
        "clip: true;",
        "if root.focused: Rectangle",
        "border-radius: root.focus-radius;",
        "border-width: HubVisualSpec.focus-ring-width;",
        "border-color: HubVisualSpec.focus-ring-color;",
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
    for snippet in [
        "clip: true;",
        "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
        "in property <image> fallback-icon-image: @image-url(\"../assets/icons/ui/alert.svg\");",
        "private property <length> focus-radius: HubVisualSpec.compact-radius;",
        "if root.active: FilledIconButton",
        "if !root.active: OutlineIconButton",
        "icon: root.has-icon-image ? root.icon-image : root.fallback-icon-image;",
        "enabled: root.enabled;",
        "tooltip: root.icon;",
        "if (root.enabled) {",
        "if root.focused: Rectangle",
        "border-radius: root.focus-radius;",
        "border-width: HubVisualSpec.focus-ring-width;",
        "border-color: HubVisualSpec.focus-ring-color;",
    ] {
        assert!(
            icon_button.contains(snippet),
            "Hub IconButton must use the reference bordered square icon-button treatment; missing {snippet}"
        );
    }
    assert!(
        !pill_button.contains("TouchArea") && !icon_button.contains("TouchArea"),
        "PillButton and IconButton should delegate pointer behavior to Material buttons instead of hand-rolled TouchArea layers"
    );
    for snippet in [
        "export component HubCommandButton inherits Rectangle",
        "in property <length> button-width: root.primary ? HubTokens.control-lg * 4 + HubTokens.space-7 + HubTokens.space-1 : HubTokens.control-lg * 4 + HubTokens.space-1;",
        "in property <length> button-height: HubTokens.control-lg;",
        "border-width: root.focused ? HubVisualSpec.focus-ring-width : HubTokens.border-width;",
        "border-color: root.focused ? HubVisualSpec.focus-ring-color : (root.primary ? HubVisualSpec.accent-stroke : HubVisualSpec.outline-muted);",
        "background: root.primary ? HubVisualSpec.accent-fill : HubVisualSpec.panel-background;",
        "StateLayerArea {",
        "color: root.primary ? HubVisualSpec.accent-stroke : MaterialPalette.on_surface;",
        "HorizontalLayout {",
        "width: root.with-menu ? parent.width - root.height - MaterialStyleMetrics.size_2 : parent.width;",
        "padding-left: root.primary ? HubTokens.space-6 + MaterialStyleMetrics.size_4 : HubTokens.space-4;",
        "CenteredIcon {",
        "source-image: root.source-image;",
        "MaterialText {",
        "if root.with-menu: Rectangle",
        "background: HubVisualSpec.accent-stroke.with_alpha(0.22);",
        "if root.with-menu: Image",
        "source: @image-url(\"../assets/icons/ui/chevron-down.svg\");",
    ] {
        assert!(
            command_button.contains(snippet),
            "HubCommandButton must centralize reference Projects header command-button and split-button chrome; missing {snippet}"
        );
    }
    assert!(
        !command_button.contains("TouchArea"),
        "HubCommandButton should use Material StateLayerArea instead of direct TouchArea handling"
    );
    for snippet in [
        "export component HubIconButton inherits Rectangle",
        "in property <length> button-width: MaterialStyleMetrics.size_40;",
        "in property <length> button-height: root.button-width;",
        "in property <length> button-radius: HubVisualSpec.compact-radius;",
        "in property <length> button-border-width: HubTokens.border-width;",
        "in property <length> icon-size: HubTokens.icon-md;",
        "in property <color> active-background: HubVisualSpec.accent-fill;",
        "in property <color> idle-background: HubVisualSpec.panel-background;",
        "in property <color> active-border: HubVisualSpec.accent-stroke;",
        "in property <color> idle-border: HubVisualSpec.outline-muted;",
        "in property <color> active-foreground: HubVisualSpec.accent-stroke;",
        "in property <color> idle-foreground: MaterialPalette.on_surface;",
        "in property <color> state-layer-color: root.active ? root.active-foreground : MaterialPalette.on_surface;",
        "in property <float> disabled-opacity: HubVisualSpec.disabled-opacity;",
        "StateLayerArea {",
        "border_radius: root.button-radius;",
        "color: root.state-layer-color;",
        "root.clicked();",
        "colorize: root.active ? root.active-foreground : root.idle-foreground;",
        "border-width: HubVisualSpec.focus-ring-width;",
    ] {
        assert!(
            hub_icon_button.contains(snippet),
            "HubIconButton must centralize reference-tuned Hub icon-button chrome and state layers; missing {snippet}"
        );
    }
    assert!(
        !hub_icon_button.contains("TouchArea"),
        "HubIconButton should use Material StateLayerArea instead of direct TouchArea handling"
    );
    for snippet in [
        "export { HubFloatingIconButton } from \"button_components.slint\";",
        "export component HubFloatingIconButton inherits HubIconButton",
        "button-width: MaterialStyleMetrics.padding_28;",
        "button-height: MaterialStyleMetrics.size_32 - MaterialStyleMetrics.size_1;",
        "button-radius: HubVisualSpec.compact-radius;",
        "button-border-width: 0px;",
        "icon-size: HubTokens.icon-sm;",
        "idle-background: HubVisualSpec.chrome-background.with_alpha(0.86);",
        "idle-border: transparent;",
        "idle-foreground: MaterialPalette.on_surface;",
        "state-layer-color: MaterialPalette.on_surface;",
    ] {
        assert!(
            components.contains(snippet) || button_components.contains(snippet),
            "HubFloatingIconButton must centralize reference card-overlay icon button chrome; missing {snippet}"
        );
    }
    assert!(
        !button_components.contains("TouchArea") && !button_components.contains("StateLayerArea {"),
        "HubFloatingIconButton should inherit HubIconButton interaction instead of declaring another local pointer layer"
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
        "in property <image> fallback-icon-image: @image-url(\"../assets/icons/ui/close.svg\");",
        "private property <length> focus-radius: HubVisualSpec.compact-radius;",
        "icon: root.has-icon-image ? root.icon-image : root.fallback-icon-image;",
        "inline: true;",
        "has_error: root.danger;",
        "enabled: root.enabled;",
        "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
        "if (root.enabled) {",
        "if root.focused: Rectangle",
        "border-radius: root.focus-radius;",
        "border-color: HubVisualSpec.focus-ring-color;",
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
        "out property <bool> focused: material-field.has-focus;",
        "private property <color> focus-border:",
        "private property <float> state-opacity:",
        "border-width: root.focused ? HubVisualSpec.focus-ring-width : 0px;",
        "border-color: root.focus-border;",
        "opacity: root.state-opacity;",
        "clip: false;",
        "placeholder_text:",
        "text <=> root.text;",
        "height: HubTokens.input-field;",
        "preferred-width: HubTokens.input-width;",
        "enabled: root.enabled;",
        "edited(value) =>",
        "accepted(value) =>",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep HubTextField backed by Material TextField with explicit Hub enabled/focused state; missing {snippet}"
        );
    }

    let settings = read_ui_file("settings.slint");
    let settings_components = read_ui_file("settings_page_components.slint");
    let settings_surface = format!("{settings}\n{settings_components}");
    for (page, source) in [
        ("settings surface", settings_surface.clone()),
        (
            "editor surface",
            format!(
                "{}\n{}",
                read_ui_file("editor.slint"),
                read_ui_file("editor_page_components.slint")
            ),
        ),
        (
            "project_page_components.slint",
            read_ui_file("project_page_components.slint"),
        ),
    ] {
        assert!(
            source.contains("HubTextField"),
            "{page} form fields must use the HubTextField wrapper"
        );
        assert!(
            !source.contains("LineEdit"),
            "{page} should not reintroduce std-widgets LineEdit now that HubTextField owns Material input behavior"
        );
    }

    for snippet in [
        "export component SettingsToolchainField inherits HubTextField",
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
            settings_surface.contains(snippet),
            "SettingsPage toolchain fields should use one local HubTextField wrapper while preserving bindings: {snippet}"
        );
    }
    assert!(
        settings_components.contains("export component SettingsToolchainField inherits HubTextField"),
        "settings_page_components.slint should own SettingsToolchainField after Settings component extraction"
    );
    assert!(
        !settings.contains("component SettingsToolchainField inherits"),
        "settings.slint should import SettingsToolchainField instead of defining it inline"
    );
    assert_eq!(
        settings_surface.matches("SettingsToolchainField {").count(),
        3,
        "SettingsPage should render python/cargo/rustup paths through SettingsToolchainField"
    );
    assert!(
        !settings.contains("SettingsToolchainField {"),
        "settings.slint should compose SettingsToolchainPanel instead of repeating toolchain field rows"
    );
    for snippet in [
        "export component SettingsSaveActionRow inherits Rectangle",
        "in property <length> button-width;",
        "in property <string> action-label;",
        "callback action-clicked();",
        "PillButton {",
        "width: root.button-width;",
        "text: root.action-label;",
        "primary: true;",
        "root.action-clicked();",
        "SettingsSaveActionRow {",
        "button-width: root.save-button-width;",
        "action-label: root.ui-text.save-settings;",
        "root.save-settings();",
    ] {
        assert!(
            settings_surface.contains(snippet),
            "SettingsPage save action should use the exported SettingsSaveActionRow wrapper while preserving button bindings: {snippet}"
        );
    }
    assert!(
        settings_components.contains("export component SettingsSaveActionRow inherits Rectangle")
            && !settings.contains("PillButton {"),
        "settings.slint should import SettingsSaveActionRow instead of constructing the footer PillButton inline"
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
        editor.contains("EditorSourceSettingsPanel {")
            && editor_components
                .contains("export component EditorSourceSettingsPanel inherits PanelSlot"),
        "editor.slint should compose EditorSourceSettingsPanel while editor_page_components.slint owns the source settings fields"
    );
    assert!(
        !editor.contains("component EditorPathFieldRow") && !editor.contains("EditorPathFieldRow {"),
        "editor.slint should keep EditorPathFieldRow definition and call sites inside editor_page_components.slint"
    );

    let project_pages = read_ui_file("project_pages.slint");
    let project_new_page = read_ui_file("project_new_page.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_create_controls = format!("{project_new_page}\n{project_components}");
    assert!(
        project_new_page.contains("field-height: HubTokens.input-field;"),
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
        "export component ProjectCreateSettingsPanel inherits PanelSlot",
        "export component ProjectCreateCompactSummaryPanel inherits PanelSlot",
        "ProjectCreateSettingsPanel {",
        "ProjectCreateCompactSummaryPanel {",
        "project-name <=> root.project-name;",
        "project-location <=> root.project-location;",
        "engine-scroll-y <=> root.new-engine-scroll-y;",
        "show-summary: !root.narrow-flow;",
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
        "root.create-project();",
        "panel-padding: root.summary-panel-padding;",
        "body-spacing: 0px;",
        "summary-height: root.narrow-flow ? root.summary-section-height : 0px;",
    ] {
        assert!(
            project_create_controls.contains(snippet),
            "ProjectNewPage create fields should use typed ProjectCreateSettingsPanel wrappers while preserving bindings and browse behavior: {snippet}"
        );
    }
    for component_name in [
        "ProjectCreateField",
        "ProjectCreateActionRow",
        "ProjectCreateSettingsPanel",
        "ProjectCreateCompactSummaryPanel",
    ] {
        assert!(
            project_components.contains(&format!("export component {component_name}")),
            "project_page_components.slint should own {component_name} after Projects workflow component extraction"
        );
        assert!(
            !project_pages.contains(&format!("component {component_name} inherits"))
                && !project_new_page.contains(&format!("component {component_name} inherits")),
            "ProjectNewPage should import {component_name} instead of declaring it locally"
        );
    }
    assert_eq!(
        project_new_page.matches("ProjectCreateField {").count(),
        0,
        "ProjectNewPage should leave project name and location rows inside ProjectCreateSettingsPanel"
    );
    assert_eq!(
        project_components.matches("ProjectCreateField {").count(),
        2,
        "ProjectCreateSettingsPanel should render project name and location through ProjectCreateField"
    );
    assert_eq!(
        project_new_page.matches("ProjectCreateActionRow {").count(),
        0,
        "ProjectNewPage should leave the create action inside ProjectCreateSettingsPanel"
    );
    assert_eq!(
        project_components
            .matches("ProjectCreateActionRow {")
            .count(),
        1,
        "ProjectCreateSettingsPanel should render the create action through ProjectCreateActionRow"
    );
    assert!(
        project_new_page.contains(
            "summary-row-height: max(HubTokens.control-sm, min(root.field-height, root.content-height / 18));"
        ),
        "ProjectNewPage create summary should stay compact enough to keep core create controls visible without depending on flow-height"
    );
    for snippet in [
        "create-action-row-height: root.field-height;",
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
        "visible: root.narrow-flow;",
        "show-summary: !root.narrow-flow;",
    ] {
        assert!(
            project_new_page.contains(snippet),
            "ProjectNewPage form rows and create action should keep page-level sizing/state metrics instead of stretched offsets; missing {snippet}"
        );
    }
    for snippet in [
        "height: root.create-action-row-height;",
        "alignment: center;",
    ] {
        assert!(
            project_create_controls.contains(snippet),
            "ProjectCreateSettingsPanel internals should align from shared Material control metrics instead of stretched offsets; missing {snippet}"
        );
    }
    assert!(
        project_new_page.contains("section-label-height: MaterialTypography.body_small.font_size * 3 / 2;")
            && project_new_page.contains(
                "engine-panel-rows: root.engine-count < 1 ? 1 : (root.engine-count > 3 ? 3 : root.engine-count);"
            )
            && project_new_page.contains(
                "engine-list-height: root.engine-count == 0 ? root.choice-row-height : root.engine-panel-rows * root.choice-row-height + (root.engine-panel-rows - 1) * root.engine-row-gap;"
            )
            && project_new_page.contains(
                "engine-section-height: root.section-label-height + MaterialStyleMetrics.spacing_8 + root.engine-list-height;"
            )
            && project_components.contains("height: root.engine-section-height;")
            && project_components.contains("ProjectEngineChoiceList {")
            && project_components.contains("list-height: root.engine-list-height;")
            && project_components.contains("list-scroll-y <=> root.engine-scroll-y;")
            && project_new_page.contains("engine-scroll-y <=> root.new-engine-scroll-y;"),
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
            !project_new_page.contains(forbidden),
            "ProjectNewPage should not duplicate engine/template selections in the compact create summary: {forbidden}"
        );
    }
    assert!(
        project_pages.contains("export { ProjectNewPage } from \"project_new_page.slint\";")
            && !project_pages.contains("ProjectCreateField {")
            && !project_pages.contains("ProjectCreateActionRow {")
            && !project_pages.contains("ProjectCreateSettingsPanel {")
            && !project_pages.contains("ProjectCreateCompactSummaryPanel {"),
        "project_pages.slint should route New Project form controls through the dedicated ProjectNewPage module"
    );
}

#[test]
fn input_primitives_expose_shared_enabled_and_focus_state_api() {
    let inputs = read_ui_file("inputs.slint");

    let search_box = inputs
        .split("export component SearchBox")
        .nth(1)
        .and_then(|source| source.split("export component HubTextField").next())
        .expect("inputs.slint must declare SearchBox before HubTextField");
    for snippet in [
        "in property <bool> enabled: true;",
        "out property <bool> focused: search-field.has-focus;",
        "private property <color> state-background:",
        "private property <bool> highlighted: root.focused || root.prominent;",
        "private property <color> state-border:",
        "private property <color> placeholder-color:",
        "border-width: root.highlighted ? HubVisualSpec.focus-ring-width : HubTokens.border-width;",
        "border-color: root.state-border;",
        "background: root.state-background;",
        "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
        "enabled: root.enabled;",
    ] {
        assert!(
            search_box.contains(snippet),
            "SearchBox must expose and consume shared enabled/focus foundation state; missing {snippet}"
        );
    }

    for (component, next_component, required) in [
        (
            "HubTextField",
            "ToolbarSelect",
            &[
                "in property <bool> enabled: true;",
                "out property <bool> focused: material-field.has-focus;",
                "private property <color> focus-border:",
                "private property <float> state-opacity:",
                "border-radius: HubVisualSpec.compact-radius;",
                "border-width: root.focused ? HubVisualSpec.focus-ring-width : 0px;",
                "border-color: root.focus-border;",
                "opacity: root.state-opacity;",
                "enabled: root.enabled;",
            ][..],
        ),
        (
            "ToolbarSelect",
            "DropDownButton",
            &[
                "in property <bool> enabled: true;",
                "in property <bool> focused: false;",
                "private property <bool> menu-ready: root.enabled && root.menu-items.length > 0;",
                "private property <color> select-background:",
                "private property <color> select-border:",
                "private property <color> select-foreground:",
                "border-width: root.focused ? HubVisualSpec.focus-ring-width : HubTokens.border-width;",
                "border-color: root.select-border;",
                "background: root.select-background;",
                "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
            ][..],
        ),
        (
            "DropDownButton",
            "SegmentButton",
            &[
                "in property <bool> enabled: true;",
                "in property <bool> focused: false;",
                "in property <length> button-height: HubTokens.control-md;",
                "private property <length> focus-radius: root.button-height / 2;",
                "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
                "if root.focused: Rectangle",
                "border-radius: root.focus-radius;",
                "border-width: HubVisualSpec.focus-ring-width;",
                "border-color: HubVisualSpec.focus-ring-color;",
                "if (root.enabled) {",
            ][..],
        ),
        (
            "SegmentButton",
            "",
            &[
                "in property <bool> enabled: true;",
                "in property <bool> focused: false;",
                "in property <length> button-height: HubTokens.control-md;",
                "private property <length> focus-radius: root.button-height / 2;",
                "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
                "if root.focused: Rectangle",
                "border-radius: root.focus-radius;",
                "border-width: HubVisualSpec.focus-ring-width;",
                "border-color: HubVisualSpec.focus-ring-color;",
                "current_index <=> root.selected-index;",
                "if (root.enabled) {",
                "} else {",
                "root.selected-index = root.active ? 0 : -1;",
            ][..],
        ),
    ] {
        let component_source = inputs
            .split(&format!("export component {component}"))
            .nth(1)
            .and_then(|source| {
                if next_component.is_empty() {
                    Some(source)
                } else {
                    source.split(&format!("export component {next_component}")).next()
                }
            })
            .unwrap_or_else(|| panic!("inputs.slint must declare {component}"));
        for snippet in required {
            assert!(
                component_source.contains(snippet),
                "{component} must expose shared enabled/focus primitive state; missing {snippet}"
            );
        }
    }
}

#[test]
fn hub_search_box_uses_reference_outlined_text_input() {
    let inputs = read_ui_file("inputs.slint");
    let search_box = inputs
        .split("export component SearchBox")
        .nth(1)
        .and_then(|source| source.split("export component HubTextField").next())
        .expect("inputs.slint must declare SearchBox before HubTextField");

    for snippet in [
        "in property <length> box-height: HubVisualSpec.toolbar-density-height;",
        "border-radius: HubVisualSpec.compact-radius;",
        "border-color: root.state-border;",
        "background: root.state-background;",
        "color: root.placeholder-color;",
        "source: @image-url(\"../assets/icons/ui/search.svg\");",
        "search-field := TextInput",
        "single-line: true;",
        "text <=> root.text;",
        "height: root.box-height;",
        "root.edited(root.text);",
        "root.accepted(root.text);",
    ] {
        assert!(
            search_box.contains(snippet),
            "SearchBox must keep the reference Hub outlined search field behavior; missing {snippet}"
        );
    }

    for forbidden in [
        "selection-background-color",
        "CenteredIcon",
        "search-field := TextField",
        "search-field := SearchBar",
        "placeholder_text: root.placeholder",
        "label: root.placeholder",
    ] {
        assert!(
            !search_box.contains(forbidden),
            "SearchBox should not return to the earlier Material capsule/search-field branch: {forbidden}"
        );
    }

    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let project_browser_page = read_ui_file("project_browser_page.slint");
    let dashboard_surface = format!("{dashboard}\n{dashboard_components}");
    for (page, source) in [
        ("project_dashboard.slint", &dashboard_surface),
        ("project_browser_page.slint", &project_browser_page),
    ] {
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
        "in property <length> button-height: HubTokens.control-md;",
        "private property <length> focus-radius: root.button-height / 2;",
        "height: root.button-height;",
        "material-segment := SegmentedButton",
        "current_index <=> root.selected-index;",
        "items: [{ text: root.text }];",
        "index_changed(index) =>",
        "if (root.enabled) {",
        "} else {",
        "root.selected-index = root.active ? 0 : -1;",
        "changed active =>",
    ] {
        assert!(
            inputs.contains(snippet),
            "SegmentButton must stay backed by the imported Material SegmentedButton; missing {snippet}"
        );
    }
    assert!(
        segment.find("} else {").expect("SegmentButton disabled branch must exist")
            < segment
                .find("if root.focused: Rectangle")
                .expect("SegmentButton focus overlay must stay after Material control"),
        "SegmentButton must reset disabled Material selection changes inside the index_changed handler"
    );
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
    let project_browser_components = read_ui_file("project_browser_components.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let toolbar_select = inputs
        .split("export component ToolbarSelect")
        .nth(1)
        .and_then(|source| source.split("export component DropDownButton").next())
        .expect("inputs.slint must declare ToolbarSelect before DropDownButton");
    for snippet in [
        "MenuItem",
        "OutlineButton",
        "HubPopupMenu",
        "import { HubDropDownSurface, HubPopupMenu } from \"overlays.slint\";",
        "in property <length> select-height: HubVisualSpec.toolbar-density-height;",
        "in property <[MenuItem]> menu-items: [];",
        "private property <length> menu-width: max(root.select-width, HubTokens.input-width / 2);",
        "private property <length> menu-offset-x: min(0px, root.select-width - root.menu-width);",
        "private property <length> trailing-icon-size: max(MaterialStyleMetrics.icon_size_18, min(MaterialStyleMetrics.icon_size_24, root.select-height * 2 / 5));",
        "private property <length> trailing-icon-inset: max(MaterialStyleMetrics.padding_12, root.select-height / 4);",
        "private property <bool> menu-ready: root.enabled && root.menu-items.length > 0;",
        "private property <color> select-background:",
        "private property <color> select-border:",
        "private property <color> select-foreground:",
        "clip: false;",
        "trigger := OutlineButton",
        "opacity: 0%;",
        "select-visual := Rectangle",
        "border-radius: HubVisualSpec.compact-radius;",
        "border-color: root.select-border;",
        "background: root.select-background;",
        "color: root.select-foreground;",
        "StateLayerArea {",
        "if (root.menu-ready) {",
        "trailing-chevron := Icon",
        "x: parent.width - root.trailing-icon-inset - self.width;",
        "y: (parent.height - self.height) / 2;",
        "source: @image-url(\"../assets/icons/ui/chevron-down.svg\");",
        "colorize: root.select-foreground;",
        "menu := HubPopupMenu",
        "x: root.menu-offset-x;",
        "menu-width: root.menu-width;",
        "menu-items: root.menu-items;",
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
        "ProjectFilterSelect,",
        "ProjectSortSelect,",
        "} from \"project_browser_components.slint\";",
    ] {
        assert!(
            project_components.contains(snippet),
            "project_page_components.slint should re-export the shared Projects filter/sort menu shell; missing {snippet}"
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
            project_browser_components.contains(snippet),
            "project_browser_components.slint must own the shared Projects filter/sort menu shell; missing {snippet}"
        );
    }
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let project_browser_page = read_ui_file("project_browser_page.slint");
    let dashboard_surface = format!("{dashboard}\n{dashboard_components}");
    for (page, source) in [
        ("project_dashboard.slint", &dashboard_surface),
        ("project_browser_page.slint", &project_browser_page),
    ] {
        assert!(
            source.contains("ProjectFilterSelect {") && source.contains("ProjectSortSelect {"),
            "{page} must reuse the shared Projects filter/sort select wrappers"
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
    assert!(
        dashboard_surface.contains("select-height: root.compact-control-height;"),
        "project_dashboard.slint must keep project select wrappers on the compact reference toolbar height"
    );
    assert!(
        project_browser_page.contains("select-height: root.toolbar-height;"),
        "project_browser_page.slint must align project select wrappers to the browser toolbar height"
    );
    assert!(
        dashboard.contains("DashboardToolbar {")
            && !dashboard.contains("ProjectFilterSelect {")
            && !dashboard.contains("ProjectSortSelect {")
            && !dashboard.contains("SearchBox {"),
        "project_dashboard.slint should compose DashboardToolbar while toolbar internals live in project_dashboard_components.slint"
    );
    assert!(
        project_pages.contains("export { ProjectBrowserPage } from \"project_browser_page.slint\";")
            && !project_pages.contains("ProjectFilterSelect {")
            && !project_pages.contains("ProjectSortSelect {"),
        "project_pages.slint should route Browser filter/sort controls through the dedicated ProjectBrowserPage module"
    );
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
        "in property <length> button-height: HubTokens.control-md;",
        "private property <length> focus-radius: root.button-height / 2;",
        "height: root.button-height;",
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
