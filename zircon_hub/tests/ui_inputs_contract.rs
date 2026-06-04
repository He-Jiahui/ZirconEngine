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
    let icon_button_components = read_ui_file("icon_button_components.slint");
    let button_state_samples = read_ui_file("button_state_sample_components.slint");
    let components = read_ui_file("components.slint");
    let project_dashboard = read_ui_file("project_dashboard_components.slint");
    for snippet in [
        "FilledButton,",
        "IconButton as MaterialIconButton,",
        "OutlineButton,",
        "TonalButton,",
        "StateLayerArea,",
        "export component PillButton",
        "FilledButton {",
        "TonalButton {",
        "export component HubCommandButton",
        "export component HubHeaderCommandGroup",
        "export component HubPanelNavigationCommand",
        "export component HubActionCommandButton",
        "export component HubActionStack",
        "export component HubFormActionRow",
        "export component HubDisclosureButton",
        "export component HubPanelHeaderActionButton",
        "export component HubUserMenuTriggerButton",
        "export component HubSidebarCollapseButton",
        "if root.focused: Rectangle",
        "export component WindowButton",
        "MaterialIconButton {",
    ] {
        assert!(
            button_components.contains(snippet),
            "button_components.slint must keep Hub button APIs backed by Material button primitives; missing {snippet}"
        );
    }
    for snippet in [
        "FilledIconButton,",
        "OutlineIconButton,",
        "StateLayerArea,",
        "export component IconButton",
        "FilledIconButton {",
        "OutlineIconButton {",
        "export component HubIconButton",
        "export component HubTopbarIconButton",
        "export component HubBackButton",
        "export component HubFlowNextButton",
        "export component HubRowActionButton",
        "export component HubViewToggleButton",
        "export component HubViewToggleGroup",
        "export component HubFloatingIconButton",
        "export component HubMoreMenuButton",
    ] {
        assert!(
            icon_button_components.contains(snippet),
            "icon_button_components.slint must own Hub icon-button APIs backed by Material icon-button primitives; missing {snippet}"
        );
    }
    for snippet in [
        "import { HubIconButton } from \"icon_button_components.slint\";",
        "export component HubButtonStateTextSample",
        "export component HubButtonStateIconSample",
        "HubButtonStateTextSampleLabel",
    ] {
        assert!(
            button_state_samples.contains(snippet),
            "button_state_sample_components.slint must own reference button-state samples after the button module split; missing {snippet}"
        );
    }
    for removed_snippet in [
        "export component PillButton",
        "export component HubCommandButton",
        "export component HubHeaderCommandGroup",
        "export component HubPanelNavigationCommand",
        "export component HubActionCommandButton",
        "export component HubActionStack",
        "export component HubFormActionRow",
        "export component HubDisclosureButton",
        "export component IconButton",
        "export component HubIconButton",
        "export component HubTopbarIconButton",
        "export component HubBackButton",
        "export component HubFlowNextButton",
        "export component HubRowActionButton",
        "export component HubPanelHeaderActionButton",
        "export component HubUserMenuTriggerButton",
        "export component HubSidebarCollapseButton",
        "export component HubViewToggleButton",
        "export component HubViewToggleGroup",
        "export component HubButtonStateTextSample",
        "export component HubButtonStateIconSample",
        "export component WindowButton",
        "export component HubMoreMenuButton",
    ] {
        assert!(
            !shared.contains(removed_snippet),
            "shared.slint should not retain button-family component ownership after extraction to focused button modules: {removed_snippet}"
        );
    }
    for removed_snippet in [
        "component HubButtonStateTextSampleLabel",
        "export component HubButtonStateTextSample",
        "export component HubButtonStateIconSample",
    ] {
        assert!(
            !button_components.contains(removed_snippet),
            "button_components.slint should not retain reference button-state sample ownership after extraction to button_state_sample_components.slint: {removed_snippet}"
        );
    }
    for removed_snippet in [
        "export component IconButton",
        "export component HubIconButton",
        "export component HubTopbarIconButton",
        "export component HubBackButton",
        "export component HubFlowNextButton",
        "export component HubRowActionButton",
        "export component HubViewToggleButton",
        "export component HubViewToggleGroup",
        "export component HubFloatingIconButton",
        "export component HubMoreMenuButton",
    ] {
        assert!(
            !button_components.contains(removed_snippet),
            "button_components.slint should not retain icon-button ownership after extraction to icon_button_components.slint: {removed_snippet}"
        );
    }

    let pill_start = button_components
        .find("export component PillButton")
        .expect("button_components.slint must declare PillButton");
    let icon_start = icon_button_components
        .find("export component IconButton")
        .expect("icon_button_components.slint must declare IconButton");
    let command_start = button_components
        .find("export component HubCommandButton")
        .expect("button_components.slint must declare HubCommandButton");
    let command_label_start = button_components
        .find("component HubCommandButtonLabel")
        .expect(
            "button_components.slint must declare HubCommandButtonLabel before HubCommandButton",
        );
    let action_start = button_components
        .find("export component HubActionCommandButton")
        .expect("button_components.slint must declare HubActionCommandButton before IconButton");
    let header_group_start = button_components
        .find("export component HubHeaderCommandGroup")
        .expect("button_components.slint must declare HubHeaderCommandGroup before HubActionCommandButton");
    let panel_navigation_start = button_components
        .find("export component HubPanelNavigationCommand")
        .expect("button_components.slint must declare HubPanelNavigationCommand before HubActionCommandButton");
    let stack_start = button_components
        .find("export component HubActionStack")
        .expect("button_components.slint must declare HubActionStack before IconButton");
    let form_action_start = button_components
        .find("export component HubFormActionRow")
        .expect("button_components.slint must declare HubFormActionRow before IconButton");
    let disclosure_start = button_components
        .find("export component HubDisclosureButton")
        .expect("button_components.slint must declare HubDisclosureButton");
    let hub_icon_start = icon_button_components
        .find("export component HubIconButton")
        .expect("icon_button_components.slint must declare HubIconButton after IconButton");
    let topbar_icon_start = icon_button_components
        .find("export component HubTopbarIconButton")
        .expect(
            "icon_button_components.slint must declare HubTopbarIconButton after HubIconButton",
        );
    let view_toggle_start = icon_button_components
        .find("export component HubViewToggleButton")
        .expect(
            "icon_button_components.slint must declare HubViewToggleButton after HubIconButton",
        );
    let back_button_start = icon_button_components
        .find("export component HubBackButton")
        .expect(
            "icon_button_components.slint must declare HubBackButton before HubViewToggleButton",
        );
    let flow_next_start = icon_button_components
        .find("export component HubFlowNextButton")
        .expect(
            "icon_button_components.slint must declare HubFlowNextButton before HubViewToggleButton",
        );
    let row_action_start = icon_button_components
        .find("export component HubRowActionButton")
        .expect(
            "icon_button_components.slint must declare HubRowActionButton before HubViewToggleButton",
        );
    let sidebar_collapse_start = button_components
        .find("export component HubSidebarCollapseButton")
        .expect(
            "button_components.slint must declare HubSidebarCollapseButton before HubViewToggleButton",
        );
    let sidebar_collapse_label_start = button_components
        .find("component HubSidebarCollapseButtonLabel")
        .expect(
            "button_components.slint must declare HubSidebarCollapseButtonLabel before HubSidebarCollapseButton",
        );
    let panel_header_action_start = button_components
        .find("export component HubPanelHeaderActionButton")
        .expect(
            "button_components.slint must declare HubPanelHeaderActionButton before HubSidebarCollapseButton",
        );
    let user_menu_trigger_start = button_components
        .find("export component HubUserMenuTriggerButton")
        .expect(
            "button_components.slint must declare HubUserMenuTriggerButton before HubSidebarCollapseButton",
        );
    let user_menu_avatar_start = button_components
        .find("component HubUserMenuAvatarMark")
        .expect(
            "button_components.slint must declare HubUserMenuAvatarMark before HubUserMenuTriggerButton",
        );
    let user_menu_name_start = button_components
        .find("component HubUserMenuNameText")
        .expect(
        "button_components.slint must declare HubUserMenuNameText before HubUserMenuTriggerButton",
    );
    let view_toggle_group_start = icon_button_components
        .find("export component HubViewToggleGroup")
        .expect(
            "icon_button_components.slint must declare HubViewToggleGroup after HubViewToggleButton",
        );
    let floating_icon_start = icon_button_components
        .find("export component HubFloatingIconButton")
        .expect(
            "icon_button_components.slint must declare HubFloatingIconButton after HubViewToggleGroup",
        );
    let more_menu_start = icon_button_components
        .find("export component HubMoreMenuButton")
        .expect(
            "icon_button_components.slint must declare HubMoreMenuButton after HubFloatingIconButton",
        );
    let button_state_text_start = button_state_samples
        .find("export component HubButtonStateTextSample")
        .expect("button_state_sample_components.slint must declare HubButtonStateTextSample");
    let button_state_text_label_start = button_state_samples
        .find("component HubButtonStateTextSampleLabel")
        .expect(
            "button_state_sample_components.slint must declare HubButtonStateTextSampleLabel before HubButtonStateTextSample",
        );
    let button_state_icon_start = button_state_samples
        .find("export component HubButtonStateIconSample")
        .expect("button_state_sample_components.slint must declare HubButtonStateIconSample after HubButtonStateTextSample");
    let window_start = button_components
        .find("export component WindowButton")
        .expect("button_components.slint must declare WindowButton");
    let pill_button = &button_components[pill_start..command_label_start];
    let command_button_label = &button_components[command_label_start..command_start];
    let command_button = &button_components[command_start..header_group_start];
    let header_command_group = &button_components[header_group_start..panel_navigation_start];
    let panel_navigation_command = &button_components[panel_navigation_start..action_start];
    let action_button = &button_components[action_start..stack_start];
    let action_stack = &button_components[stack_start..form_action_start];
    let form_action_row = &button_components[form_action_start..disclosure_start];
    let disclosure_button = &button_components[disclosure_start..panel_header_action_start];
    let icon_button = &icon_button_components[icon_start..hub_icon_start];
    let hub_icon_button = &icon_button_components[hub_icon_start..topbar_icon_start];
    let topbar_icon_button = &icon_button_components[topbar_icon_start..back_button_start];
    let back_button = &icon_button_components[back_button_start..flow_next_start];
    let flow_next_button = &icon_button_components[flow_next_start..row_action_start];
    let row_action_button = &icon_button_components[row_action_start..view_toggle_start];
    let panel_header_action_button =
        &button_components[panel_header_action_start..user_menu_avatar_start];
    let user_menu_avatar_mark = &button_components[user_menu_avatar_start..user_menu_name_start];
    let user_menu_name_text = &button_components[user_menu_name_start..user_menu_trigger_start];
    let user_menu_trigger_button =
        &button_components[user_menu_trigger_start..sidebar_collapse_label_start];
    let sidebar_collapse_label =
        &button_components[sidebar_collapse_label_start..sidebar_collapse_start];
    let sidebar_collapse_button = &button_components[sidebar_collapse_start..window_start];
    let view_toggle_button = &icon_button_components[view_toggle_start..view_toggle_group_start];
    let view_toggle_group = &icon_button_components[view_toggle_group_start..floating_icon_start];
    let button_state_text_label =
        &button_state_samples[button_state_text_label_start..button_state_text_start];
    let button_state_text = &button_state_samples[button_state_text_start..button_state_icon_start];
    let button_state_icon = &button_state_samples[button_state_icon_start..];
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
        "component HubCommandButtonLabel inherits MaterialText",
        "in property <string> value;",
        "in property <bool> primary: false;",
        "horizontal-stretch: root.primary ? 0 : 1;",
        "text: root.value;",
        "color: root.primary ? MaterialPalette.on_primary_container : MaterialPalette.on_surface;",
        "style: MaterialTypography.label_medium;",
        "overflow: elide;",
        "vertical_alignment: center;",
    ] {
        assert!(
            command_button_label.contains(snippet),
            "HubCommandButtonLabel must own command-button label typography; missing {snippet}"
        );
    }
    for snippet in [
        "export component HubCommandButton inherits Rectangle",
        "in property <length> button-width: root.primary ? HubTokens.control-lg * 4 + HubTokens.space-7 + HubTokens.space-1 : HubTokens.control-lg * 4 + HubTokens.space-1;",
        "in property <length> button-height: HubTokens.control-lg;",
        "in property <image> trailing-icon-image: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "in property <bool> show-trailing-icon: false;",
        "private property <color> primary-command-fill: HubVisualSpec.command-primary-fill;",
        "private property <color> primary-command-stroke: HubVisualSpec.command-primary-stroke;",
        "private property <bool> reserve-trailing-lane: root.with-menu || root.show-trailing-icon;",
        "border-width: root.focused ? HubVisualSpec.focus-ring-width : HubTokens.border-width;",
        "border-color: root.focused ? HubVisualSpec.focus-ring-color : (root.primary ? root.primary-command-stroke : HubVisualSpec.outline-muted);",
        "background: root.primary ? root.primary-command-fill : HubVisualSpec.panel-background;",
        "StateLayerArea {",
        "color: root.primary ? root.primary-command-stroke : MaterialPalette.on_surface;",
        "HorizontalLayout {",
        "width: root.reserve-trailing-lane ? parent.width - root.height - MaterialStyleMetrics.size_2 : parent.width;",
        "padding-left: root.primary ? HubTokens.space-6 + MaterialStyleMetrics.size_4 : HubTokens.space-4;",
        "CenteredIcon {",
        "source-image: root.source-image;",
        "HubCommandButtonLabel {",
        "value: root.text;",
        "primary: root.primary;",
        "if root.with-menu: Rectangle",
        "background: root.primary-command-stroke.with_alpha(0.22);",
        "if root.with-menu: Image",
        "source: @image-url(\"../assets/icons/ui/chevron-down.svg\");",
        "if root.show-trailing-icon && !root.with-menu: Image",
        "source: root.trailing-icon-image;",
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
    for forbidden in [
        "MaterialText {",
        "text: root.text;",
        "style: MaterialTypography.label_medium;",
        "color: root.primary ? MaterialPalette.on_primary_container : MaterialPalette.on_surface;",
    ] {
        assert!(
            !command_button.contains(forbidden),
            "HubCommandButton should not own direct visible label text after helper extraction: {forbidden}"
        );
    }
    for snippet in [
        "export component HubHeaderCommandGroup inherits HorizontalLayout",
        "in property <string> secondary-text;",
        "in property <image> secondary-image: @image-url(\"../assets/icons/ui/plus.svg\");",
        "in property <string> primary-text;",
        "in property <image> primary-image: @image-url(\"../assets/icons/ui/plus.svg\");",
        "in property <bool> primary-with-menu: false;",
        "in property <length> action-height: HubTokens.control-lg;",
        "in property <length> action-gap: HubTokens.toolbar-gap;",
        "callback secondary-clicked();",
        "callback primary-clicked();",
        "height: root.action-height;",
        "spacing: root.action-gap;",
        "alignment: center;",
        "HubCommandButton {",
        "height: root.action-height;",
        "button-height: root.action-height;",
        "text: root.secondary-text;",
        "source-image: root.secondary-image;",
        "clicked => { root.secondary-clicked(); }",
        "text: root.primary-text;",
        "source-image: root.primary-image;",
        "primary: true;",
        "with-menu: root.primary-with-menu;",
        "clicked => { root.primary-clicked(); }",
    ] {
        assert!(
            header_command_group.contains(snippet),
            "HubHeaderCommandGroup must centralize two-button page-header command composition over HubCommandButton; missing {snippet}"
        );
    }
    assert!(
        !header_command_group.contains("TouchArea")
            && !header_command_group.contains("StateLayerArea {"),
        "HubHeaderCommandGroup should delegate pointer behavior through HubCommandButton"
    );
    for snippet in [
        "export component HubPanelNavigationCommand inherits Rectangle",
        "in property <string> text;",
        "in property <image> source-image: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "in property <length> button-width: HubTokens.control-md * 5;",
        "in property <length> button-height: HubTokens.control-md;",
        "in property <bool> enabled: true;",
        "callback clicked();",
        "height: root.button-height;",
        "vertical-stretch: 0;",
        "HubCommandButton {",
        "button-width: root.button-width;",
        "button-height: root.button-height;",
        "text: root.text;",
        "source-image: root.source-image;",
        "has-source-image: true;",
        "show-trailing-icon: true;",
        "enabled: root.enabled;",
        "clicked => { root.clicked(); }",
    ] {
        assert!(
            panel_navigation_command.contains(snippet),
            "HubPanelNavigationCommand must centralize compact panel navigation command buttons over HubCommandButton; missing {snippet}"
        );
    }
    assert!(
        !panel_navigation_command.contains("TouchArea")
            && !panel_navigation_command.contains("StateLayerArea {"),
        "HubPanelNavigationCommand should delegate pointer behavior through HubCommandButton"
    );
    for snippet in [
        "export component HubActionCommandButton inherits Rectangle",
        "in property <length> action-height: MaterialStyleMetrics.size_40;",
        "horizontal-stretch: 1;",
        "min-width: 1px;",
        "preferred-width: 0px;",
        "height: root.action-height;",
        "HubCommandButton {",
        "button-width: parent.width;",
        "button-height: parent.height;",
        "source-image: root.source-image;",
        "has-source-image: root.has-source-image;",
        "clicked => { root.clicked(); }",
    ] {
        assert!(
            action_button.contains(snippet),
            "HubActionCommandButton must centralize full-width action command rows over HubCommandButton; missing {snippet}"
        );
    }
    assert!(
        !action_button.contains("TouchArea"),
        "HubActionCommandButton should delegate pointer behavior through HubCommandButton"
    );
    for snippet in [
        "export component HubActionStack inherits Rectangle",
        "in property <length> stack-height: HubTokens.control-md;",
        "in property <length> stack-spacing: HubTokens.panel-gap;",
        "horizontal-stretch: 1;",
        "preferred-width: 0px;",
        "height: root.stack-height;",
        "VerticalLayout {",
        "spacing: root.stack-spacing;",
        "@children",
    ] {
        assert!(
            action_stack.contains(snippet),
            "HubActionStack must centralize vertical command/action stack spacing over child action rows; missing {snippet}"
        );
    }
    assert!(
        !action_stack.contains("TouchArea") && !action_stack.contains("HubCommandButton {"),
        "HubActionStack should own only stack layout and leave row interaction to child actions"
    );
    for snippet in [
        "export component HubFormActionRow inherits Rectangle",
        "in property <length> row-height: HubTokens.control-lg;",
        "in property <length> row-spacing: HubTokens.toolbar-gap;",
        "in property <length> action-width: MaterialStyleMetrics.size_40 * 4;",
        "in property <length> action-height: MaterialStyleMetrics.size_40;",
        "in property <string> action-label;",
        "in property <image> action-icon: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "in property <bool> action-enabled: true;",
        "callback action-clicked();",
        "horizontal-stretch: 1;",
        "preferred-width: 0px;",
        "height: root.row-height;",
        "HorizontalLayout {",
        "alignment: center;",
        "spacing: root.row-spacing;",
        "Rectangle { horizontal-stretch: 1; }",
        "PillButton {",
        "width: root.action-width;",
        "height: root.action-height;",
        "text: root.action-label;",
        "icon-image: root.action-icon;",
        "enabled: root.action-enabled;",
        "clicked => { root.action-clicked(); }",
    ] {
        assert!(
            form_action_row.contains(snippet),
            "HubFormActionRow must centralize right-aligned primary form actions over PillButton; missing {snippet}"
        );
    }
    assert!(
        !form_action_row.contains("TouchArea") && !form_action_row.contains("StateLayerArea {"),
        "HubFormActionRow should delegate pointer behavior through PillButton"
    );
    for snippet in [
        "export component HubDisclosureButton inherits Rectangle",
        "in property <bool> expanded: false;",
        "in property <string> expanded-label;",
        "in property <string> collapsed-label;",
        "in property <length> button-height: HubTokens.control-md;",
        "in property <bool> enabled: true;",
        "callback toggled(bool);",
        "horizontal-stretch: 1;",
        "preferred-width: 0px;",
        "height: root.button-height;",
        "PillButton {",
        "width: parent.width;",
        "height: parent.height;",
        "text: root.expanded ? root.expanded-label : root.collapsed-label;",
        "icon-image: root.expanded ? @image-url(\"../assets/icons/ui/chevron-down.svg\") : @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "enabled: root.enabled;",
        "clicked => { root.toggled(!root.expanded); }",
    ] {
        assert!(
            disclosure_button.contains(snippet),
            "HubDisclosureButton must centralize Material-backed expand/collapse text button behavior over PillButton; missing {snippet}"
        );
    }
    assert!(
        !disclosure_button.contains("TouchArea") && !disclosure_button.contains("StateLayerArea {"),
        "HubDisclosureButton should delegate pointer behavior through PillButton"
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
        "export component HubTopbarIconButton inherits HubIconButton",
        "in property <length> button-size: HubTokens.control-md;",
        "button-width: root.button-size;",
        "button-height: root.button-size;",
        "button-border-width: 0px;",
        "idle-background: transparent;",
        "idle-border: transparent;",
        "idle-foreground: HubVisualSpec.topbar-icon-foreground;",
        "state-layer-color: HubVisualSpec.topbar-icon-foreground;",
        "disabled-opacity: 0.58;",
        "icon-size: HubTokens.icon-md;",
        "has-icon-image: true;",
    ] {
        assert!(
            topbar_icon_button.contains(snippet),
            "HubTopbarIconButton must centralize transparent topbar icon-button chrome over HubIconButton; missing {snippet}"
        );
    }
    assert!(
        !topbar_icon_button.contains("TouchArea") && !topbar_icon_button.contains("StateLayerArea {"),
        "HubTopbarIconButton should inherit HubIconButton interaction instead of declaring local pointer layers"
    );
    for snippet in [
        "export component HubBackButton inherits HubIconButton",
        "in property <length> button-size: MaterialStyleMetrics.size_40;",
        "button-width: root.button-size;",
        "button-height: root.button-size;",
        "icon-image: @image-url(\"../assets/icons/ui/chevron-left.svg\");",
        "has-icon-image: true;",
    ] {
        assert!(
            back_button.contains(snippet),
            "HubBackButton must centralize secondary-page back icon-button chrome over HubIconButton; missing {snippet}"
        );
    }
    assert!(
        !back_button.contains("TouchArea") && !back_button.contains("StateLayerArea {"),
        "HubBackButton should inherit HubIconButton interaction instead of declaring local pointer layers"
    );
    for snippet in [
        "export component HubFlowNextButton inherits HubIconButton",
        "in property <length> button-size: MaterialStyleMetrics.size_48;",
        "button-width: root.button-size;",
        "button-height: root.button-size;",
        "button-radius: root.button-size / 2;",
        "icon-size: HubTokens.icon-sm;",
        "icon-image: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "has-icon-image: true;",
        "active: false;",
        "idle-background: HubVisualSpec.panel-background;",
        "idle-border: HubVisualSpec.outline-muted;",
        "idle-foreground: MaterialPalette.on_surface;",
        "state-layer-color: MaterialPalette.on_surface;",
    ] {
        assert!(
            flow_next_button.contains(snippet),
            "HubFlowNextButton must centralize collapsed project-flow next icon-button chrome over HubIconButton; missing {snippet}"
        );
    }
    assert!(
        !flow_next_button.contains("TouchArea") && !flow_next_button.contains("StateLayerArea {"),
        "HubFlowNextButton should inherit HubIconButton interaction instead of declaring local pointer layers"
    );
    for snippet in [
        "export component HubRowActionButton inherits HubIconButton",
        "in property <length> button-size: HubTokens.control-md;",
        "in property <bool> framed: true;",
        "button-width: root.button-size;",
        "button-height: root.button-size;",
        "button-radius: root.button-size / 2;",
        "button-border-width: root.framed ? HubTokens.border-width : 0px;",
        "icon-size: HubTokens.icon-md;",
        "icon-image: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "has-icon-image: true;",
        "active: false;",
        "idle-background: root.framed ? HubVisualSpec.panel-background : transparent;",
        "idle-border: root.framed ? HubVisualSpec.outline-muted : transparent;",
        "idle-foreground: MaterialPalette.on_surface_variant;",
        "state-layer-color: root.idle-foreground;",
    ] {
        assert!(
            row_action_button.contains(snippet),
            "HubRowActionButton must centralize list/table row trailing action chrome over HubIconButton; missing {snippet}"
        );
    }
    assert!(
        !row_action_button.contains("TouchArea")
            && !row_action_button.contains("StateLayerArea {"),
        "HubRowActionButton should inherit HubIconButton interaction instead of declaring local pointer layers"
    );
    for snippet in [
        "export component HubPanelHeaderActionButton inherits Rectangle",
        "in property <string> text;",
        "in property <bool> enabled: true;",
        "in property <length> button-width: HubTokens.panel-min-sm / 2;",
        "in property <length> button-height: HubVisualSpec.toolbar-density-height;",
        "in property <image> icon-image: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
        "OutlineButton {",
        "text: root.text;",
        "icon: root.icon-image;",
        "enabled: root.enabled;",
        "if (root.enabled) {",
    ] {
        assert!(
            panel_header_action_button.contains(snippet),
            "HubPanelHeaderActionButton must centralize panel-header right-action chrome over Material OutlineButton; missing {snippet}"
        );
    }
    assert!(
        !panel_header_action_button.contains("TouchArea")
            && !panel_header_action_button.contains("StateLayerArea {"),
        "HubPanelHeaderActionButton should delegate pointer behavior to Material OutlineButton instead of declaring local pointer layers"
    );
    for snippet in [
        "component HubUserMenuAvatarMark inherits Rectangle",
        "in property <string> value;",
        "width: HubTokens.icon-xl;",
        "height: HubTokens.icon-xl;",
        "border-radius: HubTokens.icon-xl / 2;",
        "background: HubVisualSpec.user-avatar-background;",
        "MaterialText {",
        "text: root.value;",
        "color: HubVisualSpec.user-avatar-foreground;",
        "style: MaterialTypography.label_medium_prominent;",
        "horizontal_alignment: center;",
        "vertical_alignment: center;",
    ] {
        assert!(
            user_menu_avatar_mark.contains(snippet),
            "HubUserMenuAvatarMark must own user-menu avatar text typography; missing {snippet}"
        );
    }
    for snippet in [
        "component HubUserMenuNameText inherits MaterialText",
        "in property <string> value;",
        "text: root.value;",
        "color: HubVisualSpec.user-name-foreground;",
        "style: MaterialTypography.label_medium;",
        "vertical_alignment: center;",
        "horizontal-stretch: 1;",
        "overflow: elide;",
    ] {
        assert!(
            user_menu_name_text.contains(snippet),
            "HubUserMenuNameText must own user-menu name typography; missing {snippet}"
        );
    }
    for snippet in [
        "export component HubUserMenuTriggerButton inherits Rectangle",
        "in property <string> avatar-text;",
        "in property <string> user-name;",
        "in property <bool> tight: false;",
        "in property <length> button-width: HubTokens.user-menu-min-width;",
        "in property <length> button-height: HubTokens.control-lg;",
        "in property <color> state-layer-color: HubVisualSpec.accent-stroke;",
        "StateLayerArea {",
        "border_radius: HubVisualSpec.compact-radius;",
        "color: root.state-layer-color;",
        "root.clicked();",
        "HubUserMenuAvatarMark {",
        "value: root.avatar-text;",
        "if !root.tight: HubUserMenuNameText",
        "value: root.user-name;",
        "source: @image-url(\"../assets/icons/ui/chevron-down.svg\");",
        "colorize: MaterialPalette.on_surface_variant;",
    ] {
        assert!(
            user_menu_trigger_button.contains(snippet),
            "HubUserMenuTriggerButton must centralize the topbar user-menu trigger chrome and state layer in the button family; missing {snippet}"
        );
    }
    assert!(
        !user_menu_trigger_button.contains("TouchArea"),
        "HubUserMenuTriggerButton should use Material StateLayerArea instead of direct TouchArea handling"
    );
    for forbidden in [
        "MaterialText {",
        "text: root.avatar-text;",
        "text: root.user-name;",
        "style: MaterialTypography.label_medium_prominent;",
        "color: HubVisualSpec.user-name-foreground;",
    ] {
        assert!(
            !user_menu_trigger_button.contains(forbidden),
            "HubUserMenuTriggerButton should not own direct user identity text after helper extraction: {forbidden}"
        );
    }
    for snippet in [
        "component HubSidebarCollapseButtonLabel inherits MaterialText",
        "in property <string> value;",
        "in property <color> foreground: MaterialPalette.on_surface_variant;",
        "text: root.value;",
        "color: root.foreground;",
        "style: MaterialTypography.body_small;",
        "vertical_alignment: center;",
        "overflow: elide;",
    ] {
        assert!(
            sidebar_collapse_label.contains(snippet),
            "HubSidebarCollapseButtonLabel must own sidebar collapse label typography; missing {snippet}"
        );
    }
    for snippet in [
        "export component HubSidebarCollapseButton inherits Rectangle",
        "in property <bool> collapsed: false;",
        "in property <string> text: \"\";",
        "in property <length> button-height: HubTokens.control-md;",
        "in property <length> button-radius: HubVisualSpec.panel-radius;",
        "in property <color> foreground: MaterialPalette.on_surface_variant;",
        "in property <color> state-layer-color: MaterialPalette.on_surface;",
        "height: root.button-height;",
        "border-radius: root.button-radius;",
        "StateLayerArea {",
        "border_radius: root.button-radius;",
        "color: root.state-layer-color;",
        "root.clicked();",
        "padding-left: root.collapsed ? 0px : HubTokens.space-2;",
        "source-image: root.collapsed ? @image-url(\"../assets/icons/ui/chevron-right.svg\") : @image-url(\"../assets/icons/ui/collapse.svg\");",
        "if !root.collapsed: HubSidebarCollapseButtonLabel",
        "value: root.text;",
        "foreground: root.foreground;",
    ] {
        assert!(
            sidebar_collapse_button.contains(snippet),
            "HubSidebarCollapseButton must centralize sidebar collapse chrome and state-layer behavior in the button family; missing {snippet}"
        );
    }
    assert!(
        !sidebar_collapse_button.contains("TouchArea"),
        "HubSidebarCollapseButton should use Material StateLayerArea instead of direct TouchArea handling"
    );
    for forbidden in [
        "MaterialText {",
        "text: root.text;",
        "color: root.foreground;",
        "style: MaterialTypography.body_small;",
    ] {
        assert!(
            !sidebar_collapse_button.contains(forbidden),
            "HubSidebarCollapseButton should not own direct collapse label text after helper extraction: {forbidden}"
        );
    }
    for snippet in [
        "export component HubViewToggleButton inherits HubIconButton",
        "button-width: MaterialStyleMetrics.size_48;",
        "button-height: HubTokens.control-lg;",
        "button-radius: HubVisualSpec.compact-radius;",
        "icon-size: HubTokens.icon-sm;",
        "active-background: HubVisualSpec.view-toggle-active-fill;",
        "idle-background: HubVisualSpec.panel-background;",
        "active-border: HubVisualSpec.view-toggle-active-stroke;",
        "idle-border: HubVisualSpec.outline-muted;",
        "active-foreground: HubVisualSpec.view-toggle-active-foreground;",
        "idle-foreground: HubVisualSpec.view-toggle-idle-foreground;",
        "state-layer-color: root.active ? HubVisualSpec.view-toggle-active-stroke : MaterialPalette.on_surface;",
    ] {
        assert!(
            view_toggle_button.contains(snippet),
            "HubViewToggleButton must centralize compact grid/list view-toggle icon-button chrome; missing {snippet}"
        );
    }
    for snippet in [
        "export component HubViewToggleGroup inherits HorizontalLayout",
        "in property <string> selected-mode;",
        "in property <length> group-height: HubTokens.control-lg;",
        "in property <length> button-width: root.group-height + MaterialStyleMetrics.size_6;",
        "in property <length> group-spacing: HubTokens.panel-gap;",
        "callback selected(string);",
        "width: root.button-width * 2 + root.group-spacing;",
        "height: root.group-height;",
        "spacing: root.group-spacing;",
        "HubViewToggleButton {",
        "button-width: root.button-width;",
        "button-height: root.group-height;",
        "icon-image: @image-url(\"../assets/icons/ui/grid.svg\");",
        "active: root.selected-mode == \"grid\";",
        "root.selected(\"grid\");",
        "icon-image: @image-url(\"../assets/icons/ui/list.svg\");",
        "active: root.selected-mode == \"list\";",
        "root.selected(\"list\");",
    ] {
        assert!(
            view_toggle_group.contains(snippet),
            "HubViewToggleGroup must own the compact grid/list toggle pair over shared HubViewToggleButton chrome; missing {snippet}"
        );
    }
    assert!(
        !view_toggle_button.contains("TouchArea") && !view_toggle_group.contains("TouchArea"),
        "Hub view toggles should inherit HubIconButton interaction instead of declaring local pointer layers"
    );
    for snippet in [
        "component HubButtonStateTextSampleLabel inherits MaterialText",
        "in property <string> value;",
        "in property <bool> primary: false;",
        "in property <bool> tertiary: false;",
        "in property <bool> active: false;",
        "in property <bool> enabled: true;",
        "text: root.value;",
        "overflow: elide;",
        "horizontal_alignment: center;",
        "vertical_alignment: center;",
        "style: MaterialTypography.label_medium;",
        "color: root.tertiary ?",
        "root.enabled ? (root.active ? HubVisualSpec.accent-stroke : MaterialPalette.primary) : MaterialPalette.on_surface_variant",
        "root.primary ? MaterialPalette.on_primary_container : MaterialPalette.on_surface",
    ] {
        assert!(
            button_state_text_label.contains(snippet),
            "HubButtonStateTextSampleLabel must own reference button-state sample label typography; missing {snippet}"
        );
    }
    for snippet in [
        "export component HubButtonStateTextSample inherits Rectangle",
        "in property <string> text;",
        "in property <string> variant: \"secondary\";",
        "private property <bool> primary: root.variant == \"primary\";",
        "private property <bool> tertiary: root.variant == \"tertiary\";",
        "width: root.tertiary ? HubTokens.control-lg * 3 / 2 :",
        "border-width: root.tertiary || root.primary ? 0px : HubTokens.border-width;",
        "background: root.tertiary ? transparent : (root.primary ?",
        "HubVisualSpec.button-state-primary-default-background",
        "HubVisualSpec.button-state-secondary-default-background",
        "opacity: root.enabled ? 1.0 : (root.tertiary ? 0.45 : 0.54);",
        "HubButtonStateTextSampleLabel {",
        "width: parent.width;",
        "height: parent.height;",
        "value: root.text;",
        "primary: root.primary;",
        "tertiary: root.tertiary;",
        "active: root.active;",
        "enabled: root.enabled;",
    ] {
        assert!(
            button_state_text.contains(snippet),
            "HubButtonStateTextSample must centralize reference button-state text examples in the button family; missing {snippet}"
        );
    }
    for snippet in [
        "export component HubButtonStateIconSample inherits HubIconButton",
        "in property <bool> primary: false;",
        "button-width: HubTokens.control-lg + HubTokens.border-width * 2;",
        "button-height: HubTokens.control-lg;",
        "button-border-width: root.primary ? 0px : HubTokens.border-width;",
        "icon-image: @image-url(\"../assets/icons/ui/plus.svg\");",
        "has-icon-image: true;",
        "active-background: !root.enabled ? HubVisualSpec.button-state-icon-disabled-background : (root.primary && root.active ? HubVisualSpec.button-state-icon-primary-hover-background : HubVisualSpec.button-state-icon-hover-background);",
        "idle-background: !root.enabled ? HubVisualSpec.button-state-icon-disabled-background : (root.primary ? HubVisualSpec.button-state-icon-primary-background : HubVisualSpec.panel-background);",
        "idle-foreground: root.primary ? MaterialPalette.on_primary_container : MaterialPalette.on_surface;",
        "disabled-opacity: 0.54;",
    ] {
        assert!(
            button_state_icon.contains(snippet),
            "HubButtonStateIconSample must centralize reference button-state icon examples over HubIconButton; missing {snippet}"
        );
    }
    assert!(
        !button_state_text.contains("TouchArea")
            && !button_state_icon.contains("TouchArea")
            && !button_state_icon.contains("StateLayerArea {"),
        "Hub button-state samples should avoid new local pointer layers and let HubIconButton own icon interaction"
    );
    for forbidden in [
        "MaterialText {",
        "text: root.text;",
        "style: MaterialTypography.label_medium;",
        "color: root.tertiary ?",
    ] {
        assert!(
            !button_state_text.contains(forbidden),
            "HubButtonStateTextSample should not own direct sample label text after helper extraction: {forbidden}"
        );
    }
    for snippet in [
        "export { PillButton, HubCommandButton, HubHeaderCommandGroup, HubPanelNavigationCommand, HubActionCommandButton, HubActionStack, HubFormActionRow, HubDisclosureButton, HubPanelHeaderActionButton, HubUserMenuTriggerButton, HubSidebarCollapseButton, WindowButton } from \"button_components.slint\";",
        "export { IconButton, HubIconButton, HubTopbarIconButton, HubBackButton, HubFlowNextButton, HubRowActionButton, HubViewToggleButton, HubViewToggleGroup, HubFloatingIconButton, HubMoreMenuButton } from \"icon_button_components.slint\";",
        "export { HubButtonStateTextSample, HubButtonStateIconSample } from \"button_state_sample_components.slint\";",
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
            components.contains(snippet) || icon_button_components.contains(snippet),
            "HubFloatingIconButton must centralize reference card-overlay icon button chrome; missing {snippet}"
        );
    }
    let floating_button = &icon_button_components[floating_icon_start..more_menu_start];
    assert!(
        !floating_button.contains("TouchArea") && !floating_button.contains("StateLayerArea {"),
        "HubFloatingIconButton should inherit HubIconButton interaction instead of declaring another local pointer layer"
    );
    let more_menu_button = &icon_button_components[more_menu_start..];
    assert!(
        icon_button_components
            .contains("export component HubMoreMenuButton inherits HubFloatingIconButton"),
        "HubMoreMenuButton must inherit HubFloatingIconButton"
    );
    for snippet in [
        "icon-image: @image-url(\"../assets/icons/ui/more-vertical.svg\");",
        "has-icon-image: true;",
    ] {
        assert!(
            more_menu_button.contains(snippet),
            "HubMoreMenuButton must centralize the repeated more-menu icon binding over HubFloatingIconButton; missing {snippet}"
        );
    }
    assert!(
        !more_menu_button.contains("TouchArea") && !more_menu_button.contains("StateLayerArea {"),
        "HubMoreMenuButton should inherit HubFloatingIconButton interaction instead of declaring local pointer layers"
    );

    let window_button = &button_components[window_start..];
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
    assert!(
        project_dashboard.contains(
            "HubButtonStateIconSample,\n    HubButtonStateTextSample,\n} from \"button_state_sample_components.slint\";"
        ),
        "project_dashboard_components.slint must import reference button-state samples from the focused sample module"
    );
    assert!(
        !project_dashboard.contains("HubButtonStateTextSample,\n    HubPanelNavigationCommand"),
        "project_dashboard_components.slint should not continue importing reference button-state samples from button_components.slint"
    );
}

#[test]
fn hub_form_text_inputs_use_material_text_field_wrapper() {
    let components = read_ui_file("components.slint");
    assert!(
        components.contains("HubTextField") && components.contains("HubPathFieldRow"),
        "components.slint must re-export the Hub Material-backed text field and path-field row wrappers"
    );

    let text_inputs = read_ui_file("text_input_components.slint");
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
            text_inputs.contains(snippet),
            "text_input_components.slint must keep HubTextField backed by Material TextField with explicit Hub enabled/focused state; missing {snippet}"
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
            source.contains("HubTextField") || source.contains("HubPathFieldRow"),
            "{page} form fields must use Hub input wrappers instead of raw text controls"
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
        "export component SettingsSaveActionRow inherits HubFormActionRow",
        "in property <length> button-width;",
        "action-width: root.button-width;",
        "action-height: HubTokens.control-md;",
        "action-icon: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
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
        settings_components.contains("export component SettingsSaveActionRow inherits HubFormActionRow")
            && !settings.contains("PillButton {"),
        "settings.slint should import SettingsSaveActionRow and the wrapper should inherit HubFormActionRow instead of constructing the footer PillButton inline"
    );
    let settings_save_action = settings_components
        .split("export component SettingsSaveActionRow")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .expect("settings_page_components.slint must declare SettingsSaveActionRow");
    for forbidden in ["HorizontalLayout {", "PillButton {"] {
        assert!(
            !settings_save_action.contains(forbidden),
            "SettingsSaveActionRow should inherit HubFormActionRow instead of retaining local footer button layout: {forbidden}"
        );
    }

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
            "EditorPage Source Engine settings rows should use the exported HubPathFieldRow wrapper while preserving bindings and actions: {snippet}"
        );
    }
    for snippet in [
        "HubPathFieldRow {",
        "label: root.field-label;",
        "text <=> root.field-text;",
        "action-label: root.button-text;",
        "row-padding: HubTokens.space-2;",
        "framed: true;",
    ] {
        assert!(
            editor_components.contains(snippet),
            "EditorPathFieldRow should delegate field/action geometry to HubPathFieldRow: {snippet}"
        );
    }
    assert!(
        editor.contains("EditorSourceSettingsPanel {")
            && editor_components
                .contains("export component EditorSourceSettingsPanel inherits HubFormPanelSlot"),
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
        "HubPathFieldRow {",
        "label: root.field-label;",
        "placeholder: root.field-placeholder;",
        "text <=> root.field-text;",
        "show-action: root.show-browse;",
        "action-label: root.browse-label;",
        "root.browse-clicked();",
        "export component ProjectCreateSettingsPanel inherits HubContentPanelSlot",
        "export component ProjectCreateCompactSummaryPanel inherits HubContentPanelSlot",
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
        "export component ProjectCreateActionRow inherits HubFormActionRow",
        "action-icon: @image-url(\"../assets/icons/ui/plus.svg\");",
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
    let project_create_field = project_components
        .split("export component ProjectCreateField")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .expect("project_page_components.slint must declare ProjectCreateField");
    for forbidden in ["HubTextField {", "if root.show-browse: PillButton"] {
        assert!(
            !project_create_field.contains(forbidden),
            "ProjectCreateField should delegate path/name row structure to HubPathFieldRow: {forbidden}"
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
    let project_create_action = project_components
        .split("export component ProjectCreateActionRow")
        .nth(1)
        .and_then(|source| source.split("export component ").next())
        .expect("project_page_components.slint must declare ProjectCreateActionRow");
    for forbidden in ["HorizontalLayout {", "PillButton {"] {
        assert!(
            !project_create_action.contains(forbidden),
            "ProjectCreateActionRow should inherit HubFormActionRow instead of retaining local footer button layout: {forbidden}"
        );
    }
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
    let text_inputs = read_ui_file("text_input_components.slint");

    let search_box = text_inputs
        .split("export component SearchBox")
        .nth(1)
        .and_then(|source| source.split("export component HubTextField").next())
        .expect("text_input_components.slint must declare SearchBox before HubTextField");
    for snippet in [
        "in property <bool> enabled: true;",
        "out property <bool> focused: search-field.has-focus;",
        "private property <color> state-background:",
        "private property <bool> highlighted: root.focused || root.prominent;",
        "private property <color> state-border:",
        "private property <color> placeholder-color:",
        "border-width: root.focused ? HubVisualSpec.focus-ring-width : HubTokens.border-width;",
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
            "HubPathFieldRow",
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
            "HubPathFieldRow",
            "",
            &[
                "in property <bool> enabled: true;",
                "in property <bool> show-action: true;",
                "in property <bool> action-enabled: true;",
                "in property <length> field-height: HubTokens.input-field;",
                "in property <length> action-width: HubTokens.control-md * 3;",
                "in property <length> row-spacing: HubTokens.toolbar-gap;",
                "in property <bool> framed: false;",
                "HubTextField {",
                "HubCommandButton {",
                "enabled: root.action-enabled && root.enabled;",
            ][..],
        ),
    ] {
        let component_source = text_inputs
            .split(&format!("export component {component}"))
            .nth(1)
            .and_then(|source| {
                if next_component.is_empty() {
                    Some(source)
                } else {
                    source
                        .split(&format!("export component {next_component}"))
                        .next()
                }
            })
            .unwrap_or_else(|| panic!("text_input_components.slint must declare {component}"));
        for snippet in required {
            assert!(
                component_source.contains(snippet),
                "{component} must expose shared enabled/focus primitive state; missing {snippet}"
            );
        }
    }

    for (component, next_component, required) in [
        (
            "HubSelectTrigger",
            "DropDownButton",
            &[
                "in property <bool> enabled: true;",
                "in property <bool> focused: false;",
                "in property <bool> menu-ready: true;",
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
            "ToolbarSelect",
            "DropDownButton",
            &[
                "in property <bool> enabled: true;",
                "in property <bool> focused: false;",
                "private property <bool> menu-ready: root.enabled && root.menu-items.length > 0;",
                "HubSelectTrigger {",
                "enabled: root.enabled;",
                "focused: root.focused;",
                "menu-ready: root.menu-ready;",
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
    let text_inputs = read_ui_file("text_input_components.slint");
    let search_box = text_inputs
        .split("export component SearchBox")
        .nth(1)
        .and_then(|source| source.split("export component HubTextField").next())
        .expect("text_input_components.slint must declare SearchBox before HubTextField");

    for snippet in [
        "in property <length> box-height: HubVisualSpec.toolbar-density-height;",
        "border-radius: HubVisualSpec.compact-radius;",
        "border-color: root.state-border;",
        "background: root.state-background;",
        "SearchBoxPlaceholderText {",
        "foreground: root.placeholder-color;",
        "root.focused ? HubVisualSpec.focus-ring-color : (root.prominent ? HubVisualSpec.search-prominent-stroke",
        "root.focused ? MaterialPalette.on_surface_variant : (root.prominent ? HubVisualSpec.search-prominent-placeholder",
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
    let select_trigger = inputs
        .split("export component HubSelectTrigger")
        .nth(1)
        .and_then(|source| source.split("export component ToolbarSelect").next())
        .expect("inputs.slint must declare HubSelectTrigger before ToolbarSelect");
    for snippet in [
        "MenuItem",
        "OutlineButton",
        "HubSelectMenu",
        "import { HubSelectMenu } from \"dropdown_components.slint\";",
        "export component HubSelectTrigger",
        "in property <length> trigger-height: HubVisualSpec.toolbar-density-height;",
        "in property <length> chevron-size: max(MaterialStyleMetrics.icon_size_18, min(MaterialStyleMetrics.icon_size_24, root.trigger-height * 2 / 5));",
        "in property <length> content-inset: max(MaterialStyleMetrics.padding_12, root.trigger-height / 4);",
        "private property <color> select-background:",
        "private property <color> select-border: root.focused ? HubVisualSpec.focus-ring-color : HubVisualSpec.outline-muted;",
        "private property <color> select-foreground:",
        "trigger := OutlineButton",
        "opacity: 0%;",
        "select-visual := Rectangle",
        "border-radius: HubVisualSpec.compact-radius;",
        "border-color: root.select-border;",
        "background: root.select-background;",
        "StateLayerArea {",
        "if (root.menu-ready) {",
        "trailing-chevron := Icon",
        "x: parent.width - root.content-inset - self.width;",
        "y: (parent.height - self.height) / 2;",
        "source: @image-url(\"../assets/icons/ui/chevron-down.svg\");",
        "colorize: root.select-foreground;",
        "in property <length> select-height: HubVisualSpec.toolbar-density-height;",
        "in property <[MenuItem]> menu-items: [];",
        "private property <bool> menu-ready: root.enabled && root.menu-items.length > 0;",
        "clip: false;",
        "HubSelectTrigger {",
        "trigger-width: parent.width;",
        "trigger-height: parent.height;",
        "menu-ready: root.menu-ready;",
        "activated =>",
        "menu := HubSelectMenu",
        "anchor-width: root.select-width;",
        "anchor-height: root.height;",
        "menu-min-width: HubTokens.input-width / 2;",
        "select-items: root.menu-items;",
        "root.selected(root.options[index].id);",
    ] {
        assert!(
            inputs.contains(snippet),
            "ToolbarSelect must stay backed by imported Material menu/button primitives; missing {snippet}"
        );
    }
    for forbidden in [
        "trigger := OutlineButton",
        "select-visual := Rectangle",
        "trailing-chevron := Icon",
        "StateLayerArea {",
    ] {
        assert!(
            !toolbar_select.contains(forbidden),
            "ToolbarSelect should delegate trigger painting to HubSelectTrigger: {forbidden}"
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
    assert!(
        select_trigger.contains("callback activated();")
            && select_trigger.contains("root.activated();"),
        "HubSelectTrigger must expose one activation callback for menu anchors"
    );
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
