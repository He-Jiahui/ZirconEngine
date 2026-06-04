//! Static contracts for Zircon Hub data-display primitives.

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
fn data_display_lists_use_material_scroll_view() {
    let data_display = read_ui_file("data_display.slint");
    let list_container = read_ui_file("list_container_components.slint");
    let table_view = read_ui_file("table_view_components.slint");
    let catalog_components = read_ui_file("catalog_page_components.slint");
    let data_surface = format!("{list_container}\n{table_view}\n{catalog_components}");

    for snippet in [
        "import { ScrollView } from \"material_bridge.slint\";",
        "table-scroll := HubTableBody {",
        "export component PanelListViewport inherits ScrollView",
        "export component HubMenuListViewport inherits PanelListViewport",
        "row-height: HubTokens.list-row-md;",
        "empty-height: HubTokens.list-row-lg;",
        "export component HubTableBody inherits PanelListViewport",
        "export component CatalogPage inherits PageScrollSurface",
        "min-height: root.row-height * 4;",
        "row-slot-height: root.row-height + HubTokens.space-2;",
        "panel-chrome-height: HubTokens.space-4 * 2 + HubTokens.control-md + HubTokens.toolbar-gap;",
        "fit-row-count: Math.floor(root.fit-list-height / root.row-slot-height);",
        "panel-height: root.row-count > root.visible-row-count ? root.fitted-panel-height : max(root.row-slot-height + root.panel-chrome-height, root.content-height - root.tabs-block-height);",
        "empty-height: root.row-height + HubTokens.space-4;",
        "list-content-height: root.row-count == 0 ? root.empty-height : root.row-count * root.row-height + (root.row-count - 1) * root.row-spacing + root.vertical-padding * 2;",
        "row-spacing: root.row-gap;",
        "vertical-padding: root.row-gap * 2;",
        "empty-height: root.row-height + root.row-gap * 4;",
        "height: max(root.row-height + HubTokens.space-2, parent.height);",
        "vertical-stretch: 1;",
        "viewport_y <=> root.scroll-y;",
        "scroll-y <=> root.scroll-y;",
        "row-count: root.row-count;",
        "viewport_width: root.visible_width;",
        "viewport_height: max(root.visible_height, root.list-content-height);",
        "vertical_scrollbar_policy: ScrollBarPolicy.as-needed;",
        "horizontal_scrollbar_policy: ScrollBarPolicy.always-off;",
    ] {
        assert!(
            data_surface.contains(snippet),
            "Data-display list and table surfaces must use the Material ScrollView API; missing {snippet}"
        );
    }

    for forbidden in [
        "std-widgets.slint",
        "mouse-drag-pan-enabled",
        "viewport-y <=>",
        "visible-width",
        "min-height: HubTokens.list-row-lg * 4",
        "panel-height: max(root.row-height * 4, root.content-height)",
        "panel-height: max(HubTokens.list-row-lg * 4, root.content-height)",
        "panel-height: max(HubTokens.list-row-lg * 4, root.height - HubTokens.page-padding * 2)",
        "table-scroll := ScrollView {",
        "catalog-scroll := ScrollView {",
        "table-content-height:",
        "catalog-content-height:",
        "viewport_width: table-scroll.visible_width;",
        "viewport_width: catalog-scroll.visible_width;",
        "viewport_height: max(table-scroll.visible_height, root.table-content-height);",
        "viewport_height: max(catalog-scroll.visible_height, root.catalog-content-height);",
        "root.row-count * (root.row-height + root.row-gap) + root.row-gap * 4",
        "root.row-count * (root.row-height + HubTokens.space-2) + HubTokens.space-2",
        "root.row-count * (root.row-height + root.row-spacing) + root.vertical-padding * 2",
        "root.height - HubTokens.page-padding * 2",
        "page-surface := PageScrollSurface",
        "width: root.width;",
        "height: root.height;",
    ] {
        assert!(
            !data_surface.contains(forbidden),
            "Data-display list/table scrolling should not return to the std-widgets ScrollView surface: {forbidden}"
        );
    }

    for table_component in [
        "TableColumnHeader",
        "TableCellText",
        "ProjectTableRow",
        "HubTableBody",
        "DataTable",
        "HubTableView",
    ] {
        assert!(
            !data_display.contains(&format!("export component {table_component}")),
            "data_display.slint should not regain table-view ownership after the table module split: {table_component}"
        );
    }
    for catalog_component in ["CatalogListPanel", "CatalogPage"] {
        assert!(
            !data_display.contains(&format!("export component {catalog_component}")),
            "data_display.slint should not regain catalog-page ownership after the catalog module split: {catalog_component}"
        );
    }
    for list_component in [
        "PanelListViewport",
        "HubMenuListViewport",
        "HubListView",
        "HubListPanelSlot",
        "HubTabbedListPanelSlot",
    ] {
        assert!(
            !data_display.contains(&format!("export component {list_component}")),
            "data_display.slint should not retain list-container ownership after the focused list module split: {list_component}"
        );
    }
}

#[test]
fn row_trailing_slot_delegates_action_chrome_to_button_family() {
    let icon_button_components = read_ui_file("icon_button_components.slint");
    let components = read_ui_file("components.slint");
    let row_slots = read_ui_file("row_slot_components.slint");
    let trailing_slot = row_slots
        .split("export component HubRowTrailingSlot")
        .nth(1)
        .expect("row_slot_components.slint must declare HubRowTrailingSlot");

    assert!(
        icon_button_components.contains("export component HubRowActionButton inherits HubIconButton"),
        "icon_button_components.slint must own row trailing action chrome through HubRowActionButton"
    );
    assert!(
        components.contains("HubRowActionButton,"),
        "components.slint must re-export HubRowActionButton for shared row action consumers"
    );

    for snippet in [
        "import { HubRowActionButton } from \"icon_button_components.slint\";",
        "HubRowActionButton {",
        "button-size: root.action-size;",
        "button-radius: root.action-radius;",
        "icon-image: root.action-icon-image;",
        "icon-size: root.action-icon-size;",
        "idle-foreground: root.action-foreground;",
        "state-layer-color: root.action-state-layer-color;",
        "framed: root.action-framed;",
        "clicked => { root.clicked(); }",
    ] {
        assert!(
            row_slots.contains(snippet) || trailing_slot.contains(snippet),
            "HubRowTrailingSlot must route its action affordance through HubRowActionButton; missing {snippet}"
        );
    }

    for forbidden in [
        "import { HubIconButton } from \"button_components.slint\";",
        "import { HubIconButton } from \"icon_button_components.slint\";",
        "HubIconButton {",
        "StateLayerArea {",
        "if root.action-visible && !root.action-framed: Rectangle",
        "source: root.action-icon-image;",
    ] {
        assert!(
            !trailing_slot.contains(forbidden) && !row_slots.contains(forbidden),
            "HubRowTrailingSlot should not own local action-button internals after the button-family extraction: {forbidden}"
        );
    }
}

#[test]
fn data_display_catalog_empty_state_uses_material_text() {
    let data_display = read_ui_file("data_display.slint");
    let catalog_components = read_ui_file("catalog_page_components.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let catalog_panel = catalog_components
        .split("export component CatalogListPanel")
        .nth(1)
        .and_then(|source| source.split("export component CatalogPage").next())
        .expect("catalog_page_components.slint must declare CatalogListPanel before CatalogPage");

    for snippet in [
        "if root.row-count == 0: EmptyStateBlock",
        "title: root.empty-title;",
        "detail: root.empty-detail;",
        "body-padding: MaterialStyleMetrics.padding_14;",
        "center-content: true;",
    ] {
        assert!(
            catalog_panel.contains(snippet),
            "CatalogListPanel empty state should route through EmptyStateBlock; missing {snippet}"
        );
    }

    let empty_block = surfaces
        .split("export component EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("export component EmptyStatePanel").next())
        .expect("surfaces.slint must declare EmptyStateBlock before EmptyStatePanel");
    for snippet in [
        "MaterialText {",
        "text: root.title;",
        "style: root.title-prominent ? MaterialTypography.title_medium : MaterialTypography.label_large;",
        "if root.detail != \"\": MutedText",
    ] {
        assert!(
            empty_block.contains(snippet),
            "EmptyStateBlock should own MaterialText title typography; missing {snippet}"
        );
    }

    assert!(
        !catalog_panel.lines().any(|line| line.trim() == "Text {")
            && !empty_block.lines().any(|line| line.trim() == "Text {"),
        "CatalogListPanel empty state should not return to a raw Text title"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !catalog_panel.contains(forbidden) && !empty_block.contains(forbidden),
            "CatalogListPanel empty state should not return to raw Text font bindings: {forbidden}"
        );
    }
    assert!(
        !data_display.contains("CatalogListPanel") && !data_display.contains("CatalogPage"),
        "data_display.slint should not regain catalog-page/list ownership after the focused catalog module split"
    );
}

#[test]
fn row_surface_owns_shared_selected_disabled_focus_state() {
    let data_display = read_ui_file("data_display.slint");
    let row_surface = data_display
        .split("export component HubRowSurface")
        .nth(1)
        .and_then(|source| {
            source
                .split("export component HubInteractiveRowSurface")
                .next()
        })
        .expect("data_display.slint must declare HubRowSurface before HubInteractiveRowSurface");

    for snippet in [
        "in property <bool> selected: false;",
        "in property <bool> focused: false;",
        "in property <bool> enabled: true;",
        "in property <length> row-height: HubTokens.list-row-md;",
        "in property <length> row-radius: HubVisualSpec.compact-radius;",
        "in property <length> idle-border-width: 0px;",
        "in property <length> selected-border-width: HubTokens.border-width;",
        "in property <color> idle-background: HubVisualSpec.panel-background;",
        "in property <color> selected-background: MaterialPalette.secondary_container;",
        "in property <color> disabled-background: HubVisualSpec.panel-background.with_alpha(HubVisualSpec.disabled-opacity);",
        "in property <color> content-foreground: root.selected ? root.selected-foreground : root.idle-foreground;",
        "border-width: root.focused ? HubVisualSpec.focus-ring-width : (root.selected ? root.selected-border-width : root.idle-border-width);",
        "border-color: root.focused ? HubVisualSpec.focus-ring-color : (root.selected ? root.selected-border : root.idle-border);",
        "background: !root.enabled ? root.disabled-background : (root.selected ? root.selected-background : root.idle-background);",
        "opacity: root.enabled ? 1.0 : HubVisualSpec.disabled-opacity;",
        "@children",
    ] {
        assert!(
            row_surface.contains(snippet),
            "HubRowSurface must centralize reusable Material row state; missing {snippet}"
        );
    }

    for forbidden in ["StateLayerArea", "ListTile {", "TouchArea"] {
        assert!(
            !row_surface.contains(forbidden),
            "HubRowSurface should own only the root row surface and leave content/interaction to typed row components: {forbidden}"
        );
    }
}

#[test]
fn interactive_row_surface_owns_shared_state_layer() {
    let components = read_ui_file("components.slint");
    let data_display = read_ui_file("data_display.slint");
    let interactive_surface = data_display
        .split("export component HubInteractiveRowSurface")
        .nth(1)
        .and_then(|source| source.split("component HubMenuRowTrailingText").next())
        .expect(
            "data_display.slint must declare HubInteractiveRowSurface before HubMenuRowTrailingText",
        );

    assert!(
        components.contains("HubInteractiveRowSurface,"),
        "components.slint must re-export HubInteractiveRowSurface for shared clickable row wrappers"
    );
    assert!(
        data_display.contains("export component HubInteractiveRowSurface inherits HubRowSurface"),
        "data_display.slint must declare HubInteractiveRowSurface as the shared clickable row wrapper"
    );

    for snippet in [
        "callback clicked;",
        "in property <bool> interaction-enabled: root.enabled;",
        "in property <color> interaction-foreground: root.content-foreground;",
        "row-state := StateLayerArea {",
        "color: root.interaction-foreground;",
        "border_radius: root.row-radius;",
        "if (root.interaction-enabled) {",
        "root.clicked();",
        "@children",
    ] {
        assert!(
            interactive_surface.contains(snippet),
            "HubInteractiveRowSurface must centralize reusable Material row interaction state; missing {snippet}"
        );
    }

    for forbidden in ["ListTile {", "MaterialText {", "IconButton {", "TouchArea"] {
        assert!(
            !interactive_surface.contains(forbidden),
            "HubInteractiveRowSurface should add only the shared state layer and leave row content to typed rows: {forbidden}"
        );
    }
}

#[test]
fn menu_row_uses_material_list_tile_state_contract() {
    let data_display = read_ui_file("data_display.slint");
    let trailing_text = data_display
        .split("component HubMenuRowTrailingText")
        .nth(1)
        .and_then(|source| source.split("export component HubMenuRow").next())
        .expect("data_display.slint must declare HubMenuRowTrailingText before HubMenuRow");
    for snippet in [
        "inherits MaterialText",
        "in property <string> value;",
        "in property <color> foreground;",
        "text: root.value;",
        "color: root.foreground;",
        "style: MaterialTypography.label_medium;",
        "horizontal_alignment: right;",
        "vertical_alignment: center;",
        "overflow: elide;",
    ] {
        assert!(
            trailing_text.contains(snippet),
            "HubMenuRowTrailingText must own menu-row trailing text typography; missing {snippet}"
        );
    }

    let menu_row = data_display
        .split("export component HubMenuRow")
        .nth(1)
        .and_then(|source| source.split("export component InfoRow").next())
        .expect("data_display.slint must declare HubMenuRow before InfoRow");

    for snippet in [
        "inherits HubRowSurface",
        "ListTile {",
        "text: root.title;",
        "supporting_text: root.detail;",
        "avatar_icon: root.leading-image;",
        "avatar_background: root.avatar-background;",
        "avatar_foreground: root.avatar-foreground;",
        "in property <bool> has-leading-image: false;",
        "in property <bool> show-trailing-icon: false;",
        "in property <bool> show-trailing-text: false;",
        "in property <bool> dense: false;",
        "private property <length> trailing-slot-width: root.show-trailing-text ? HubTokens.control-md * 2 : (root.show-trailing-icon ? HubTokens.control-sm : 0px);",
        "private property <color> avatar-foreground: root.enabled-avatar-foreground;",
        "row-height: root.dense ? HubTokens.list-row-sm : HubTokens.list-row-md;",
        "selected-border-width: 0px;",
        "idle-background: transparent;",
        "selected-background: MaterialPalette.secondary_container;",
        "enabled: root.enabled;",
        "clicked =>",
        "if root.show-trailing-text: HubMenuRowTrailingText",
        "value: root.trailing-text;",
        "foreground: root.content-foreground;",
        "if root.show-trailing-icon: Icon",
    ] {
        assert!(
            menu_row.contains(snippet),
            "HubMenuRow must own shared Material menu/list row state; missing {snippet}"
        );
    }

    for forbidden in [
        "area := TouchArea",
        "area.has-hover",
        "avatar-foreground: root.selected ? HubVisualSpec.accent-stroke",
        "MaterialText {",
        "component InfoRow",
        "component ActionRow",
        "border-radius: HubVisualSpec.panel-radius;",
    ] {
        assert!(
            !menu_row.contains(forbidden),
            "HubMenuRow should stay a reusable row primitive without local hover shells or row-specialization coupling: {forbidden}"
        );
    }
}

#[test]
fn info_row_uses_shared_row_surface_and_slots() {
    let data_display = read_ui_file("data_display.slint");
    let info_row = data_display
        .split("export component InfoRow")
        .nth(1)
        .and_then(|source| source.split("export component HubKeyValueRow").next())
        .expect("data_display.slint must declare InfoRow before ActionRow");

    for snippet in [
        "HubRowLeadingIconSlot {",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "alignment: stretch;",
        "preferred-width: 0px;",
        "title: root.title;",
        "detail: root.supporting-text;",
        "icon-image: root.visible-leading-image;",
        "shell-background: root.enabled ? root.enabled-avatar-background : MaterialPalette.surface_container_high;",
        "show-badge: root.show-trailing-badge;",
        "collapse-badge: root.collapse-trailing-label;",
        "show-action: root.show-arrow;",
        "action-size: root.trailing-icon-size;",
        "in property <bool> collapse-trailing-label: false;",
        "in property <bool> pressed: false;",
        "in property <bool> hovered: false;",
        "in property <color> enabled-avatar-background: root.avatar-background;",
        "in property <color> enabled-avatar-foreground: root.avatar-foreground;",
        "avatar-background: HubVisualSpec.neutral-icon-background;",
        "avatar-foreground: HubVisualSpec.neutral-icon-foreground;",
        "row-radius: HubVisualSpec.panel-radius;",
        "idle-border-width: HubTokens.border-width;",
        "selected-border-width: HubTokens.border-width;",
        "idle-border: HubVisualSpec.outline-muted;",
        "selected-border: HubVisualSpec.accent-stroke;",
        "idle-background: HubVisualSpec.panel-background;",
        "selected-background: MaterialPalette.secondary_container;",
        "disabled-background: HubVisualSpec.panel-background.with_alpha(HubVisualSpec.disabled-opacity);",
        "root.collapse-trailing-label ? (root.show-arrow ? HubTokens.control-md : 0px)",
        "clicked =>",
    ] {
        assert!(
            info_row.contains(snippet),
            "InfoRow must compose HubRowSurface with shared row slots and compact trailing-label behavior; missing {snippet}"
        );
    }
    assert!(
        data_display.contains("export component InfoRow inherits HubInteractiveRowSurface"),
        "InfoRow must inherit HubInteractiveRowSurface instead of owning its own interaction layer"
    );

    for forbidden in [
        "ListTile {",
        "IconButton {",
        "row-state := StateLayerArea {",
        "area := TouchArea",
        "border-color: area.has-hover",
        "background: area.has-hover",
        "preferred-width: HubTokens.panel-min-sm;",
        "avatar_icon:",
        "avatar_background: root.enabled ? root.avatar-background",
        "avatar_foreground: root.enabled ? root.avatar-foreground",
    ] {
        assert!(
            !info_row.contains(forbidden),
            "InfoRow should not return to a custom painted information row: {forbidden}"
        );
    }

    let key_value_row = data_display
        .split("export component HubKeyValueRow")
        .nth(1)
        .and_then(|source| source.split("export component ActionRow").next())
        .expect("data_display.slint must declare HubKeyValueRow before ActionRow");
    for snippet in [
        "inherits HubRowSurface",
        "in property <string> label;",
        "in property <string> value;",
        "in property <bool> badge-value: false;",
        "in property <string> badge-tone: \"neutral\";",
        "HubRowMetaSlot {",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "text: root.label;",
        "title: root.value;",
        "badge-text: root.value;",
        "badge-tone: root.badge-tone;",
        "show-badge: true;",
        "show-action: false;",
        "selected-border-width: 0px;",
        "idle-background: transparent;",
        "selected-background: transparent;",
    ] {
        assert!(
            key_value_row.contains(snippet),
            "HubKeyValueRow must compose compact key/value summary rows from shared row slots; missing {snippet}"
        );
    }
    for forbidden in [
        "Badge {",
        "ListTile {",
        "IconButton {",
        "row-state := StateLayerArea",
        "TouchArea",
    ] {
        assert!(
            !key_value_row.contains(forbidden),
            "HubKeyValueRow should stay a non-interactive slot-composed summary row: {forbidden}"
        );
    }

    let badge_meta_strip = data_display
        .split("export component HubBadgeMetaStrip")
        .nth(1)
        .and_then(|source| source.split("export component ActionRow").next())
        .expect("data_display.slint must declare HubBadgeMetaStrip before ActionRow");
    for snippet in [
        "inherits HubRowSurface",
        "in property <string> first-badge-text;",
        "in property <string> second-badge-text;",
        "in property <string> meta-text;",
        "HubRowTrailingSlot {",
        "HubRowMetaSlot {",
        "badge-text: root.first-badge-text;",
        "badge-text: root.second-badge-text;",
        "text: root.meta-text;",
        "show-action: false;",
        "slot-spacing: 0px;",
        "selected-border-width: 0px;",
        "idle-background: transparent;",
        "selected-background: transparent;",
    ] {
        assert!(
            badge_meta_strip.contains(snippet),
            "HubBadgeMetaStrip must compose compact badge/meta strips from shared row slots; missing {snippet}"
        );
    }
    for forbidden in [
        "Badge {",
        "MaterialText {",
        "ListTile {",
        "IconButton {",
        "row-state := StateLayerArea",
        "TouchArea",
    ] {
        assert!(
            !badge_meta_strip.contains(forbidden),
            "HubBadgeMetaStrip should stay a non-interactive slot-composed status strip: {forbidden}"
        );
    }

    let action_row = data_display
        .split("export component ActionRow")
        .nth(1)
        .and_then(|source| source.split("component MetricCardTextStack").next())
        .expect("data_display.slint must declare ActionRow before MetricCardTextStack");
    for snippet in [
        "interaction-enabled: root.enabled && root.action.enabled;",
        "interaction-foreground: root.action.enabled ? root.content-foreground : MaterialPalette.on_surface_variant;",
        "in property <string> title-override;",
        "in property <string> detail-override;",
        "in property <length> leading-shell-size-override: 0px;",
        "in property <length> leading-icon-size-override: 0px;",
        "private property <string> visible-title: root.title-override == \"\" ? root.action.title : root.title-override;",
        "private property <string> visible-detail: root.detail-override == \"\" ? root.action.detail : root.detail-override;",
        "private property <length> leading-shell-size:",
        "root.leading-shell-size-override > 0px ? root.leading-shell-size-override",
        "private property <length> leading-icon-size:",
        "root.leading-icon-size-override > 0px ? root.leading-icon-size-override",
        "HubRowLeadingIconSlot {",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "alignment: stretch;",
        "preferred-width: 0px;",
        "idle-border-width: HubTokens.border-width;",
        "idle-border: HubVisualSpec.outline-muted;",
        "title: root.visible-title;",
        "detail: root.visible-detail;",
    ] {
        assert!(
            action_row.contains(snippet),
            "ActionRow must expose shared row-surface focus state through HubRowSurface and row slots; missing {snippet}"
        );
    }
    assert!(
        data_display.contains("export component ActionRow inherits HubInteractiveRowSurface"),
        "ActionRow must inherit HubInteractiveRowSurface instead of owning its own interaction layer"
    );
}

#[test]
fn catalog_rows_opt_into_compact_trailing_labels() {
    let catalog_components = read_ui_file("catalog_page_components.slint");
    for (page, row_component) in [
        ("assets.slint", "AssetRow"),
        ("plugins.slint", "PluginRow"),
        ("learn.slint", "LearnRow"),
    ] {
        let source = read_ui_file(page);
        let surface = format!("{source}\n{catalog_components}");
        for snippet in [
            "in property <bool> collapse-label: false;",
            "collapse-trailing-label: root.collapse-label;",
            "label-collapse := ResponsiveCollapse {",
            "content-width: root.content-width;",
            "collapse-at: HubTokens.breakpoint-medium;",
            "collapse-label: label-collapse.collapsed;",
            "content-width: root.content-width;",
        ] {
            assert!(
                surface.contains(snippet),
                "{page} catalog rows should drive compact trailing-label behavior from the page width instead of squeezing body copy at narrow widths or deriving layout from row width; missing {snippet}"
            );
        }
        assert!(
            catalog_components.contains(&format!("export component {row_component}")),
            "catalog_page_components.slint should export {row_component}"
        );
        assert!(
            catalog_components
                .contains("in property <length> content-width: HubTokens.panel-min-lg;")
                && catalog_components.contains("root.content-width / 4")
                && !catalog_components.contains("root.width /"),
            "CatalogColumnRow should derive trailing/meta widths from page content-width instead of local root.width arithmetic"
        );
        assert!(
            !source.contains(&format!("component {row_component}")),
            "{page} should import {row_component} instead of defining it inline"
        );
    }

    let learn = read_ui_file("learn.slint");
    let learn_surface = format!("{learn}\n{catalog_components}");
    assert!(
        learn_surface.contains("show-arrow: true;"),
        "Learn rows should keep their compact arrow affordance after the category badge collapses"
    );
}

#[test]
fn action_row_uses_shared_row_surface_and_slots() {
    let data_display = read_ui_file("data_display.slint");
    let action_row = data_display
        .split("export component ActionRow")
        .nth(1)
        .and_then(|source| source.split("component MetricCardTextStack").next())
        .expect("data_display.slint must declare ActionRow before MetricCardTextStack");

    for snippet in [
        "in property <bool> plain-avatar: false;",
        "in property <bool> plain-trailing: false;",
        "in property <bool> show-trailing: true;",
        "in property <bool> compact-shell: false;",
        "in property <float> disabled-shell-opacity: 1.0;",
        "in property <color> enabled-avatar-background: HubVisualSpec.neutral-icon-background;",
        "in property <color> enabled-avatar-foreground: HubVisualSpec.neutral-icon-foreground;",
        "row-corner-radius: root.compact-shell ? HubVisualSpec.compact-radius : HubVisualSpec.panel-radius;",
        "action-avatar-background: root.plain-avatar ? transparent :",
        "action-avatar-foreground: root.plain-avatar ? HubVisualSpec.neutral-icon-foreground :",
        "row-radius: root.row-corner-radius;",
        "idle-border: HubVisualSpec.outline-muted;",
        "idle-background: root.action.enabled ? HubVisualSpec.panel-background : HubVisualSpec.panel-background.with_alpha(root.disabled-shell-opacity);",
        "interaction-enabled: root.enabled && root.action.enabled;",
        "interaction-foreground: root.action.enabled ? root.content-foreground : MaterialPalette.on_surface_variant;",
        "clicked =>",
        "HubRowLeadingIconSlot {",
        "HubRowMainSlot {",
        "HubRowTrailingSlot {",
        "alignment: stretch;",
        "preferred-width: 0px;",
        "title: root.visible-title;",
        "detail: root.visible-detail;",
        "shell-background: root.action-avatar-background;",
        "icon-foreground: root.action-avatar-foreground;",
        "clicked =>",
        "slot-width: root.trailing-size;",
        "show-action: true;",
        "action-framed: !root.plain-trailing;",
        "chevron-right.svg",
    ] {
        assert!(
            action_row.contains(snippet),
            "ActionRow must compose operation-row content from shared row slots; missing {snippet}"
        );
    }
    assert!(
        data_display.contains("export component ActionRow inherits HubInteractiveRowSurface"),
        "ActionRow must inherit HubInteractiveRowSurface instead of owning its own interaction layer"
    );

    for forbidden in [
        "CenteredIcon",
        "ListTile {",
        "IconButton {",
        "row-state := StateLayerArea {",
        "avatar_icon:",
        "area := TouchArea",
        "border-color: area.has-hover",
        "background: area.has-hover",
    ] {
        assert!(
            !action_row.contains(forbidden),
            "ActionRow should not return to a custom painted operation row: {forbidden}"
        );
    }
}

#[test]
fn build_history_rows_are_shared_between_editor_and_builds() {
    let data_display = read_ui_file("data_display.slint");
    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let builds = read_ui_file("builds.slint");
    let app = read_ui_file("app.slint");

    for snippet in [
        "export component BuildHistoryRow inherits InfoRow",
        "in property <SourceBuildHistoryRowData> record;",
        "in property <bool> collapse-label: false;",
        "title: root.record.detail;",
        "detail: root.record.log != \"\" ? root.record.log : root.record.output-path;",
        "meta: root.record.process-id != \"\" ? root.record.process-id + \" / \" + root.record.profile + \" / \" + root.record.finished : root.record.profile + \" / \" + root.record.finished;",
        "trailing-text: root.record.status;",
        "collapse-trailing-label: root.collapse-label;",
    ] {
        assert!(
            data_display.contains(snippet),
            "BuildHistoryRow must be a shared Material ListTile-backed data-display row; missing {snippet}"
        );
    }

    assert!(
        editor_components.contains("BuildHistoryRow,")
            && editor_components.contains("for record in root.source-build-history: BuildHistoryRow")
            && !editor.contains("BuildHistoryRow,")
            && !editor.contains("for record in root.source-build-history: BuildHistoryRow"),
        "EditorPage should reuse the shared BuildHistoryRow through EditorBuildHistoryPanel instead of owning a page-local row loop"
    );
    let builds_components = read_ui_file("builds_page_components.slint");
    let builds_surface = format!("{builds}\n{builds_components}");
    for snippet in [
        "in property <[SourceBuildHistoryRowData]> source-build-history;",
        "in property <int> source-build-history-count;",
        "PanelListViewport {",
        "for record in root.source-build-history: BuildHistoryRow",
        "collapse-label: workspace-split.compact-labels;",
        "if root.source-build-history-count == 0: EmptyStateBlock",
        "no-build-history-title: root.ui-text.no-build-history-short;",
        "title: root.no-build-history-title;",
    ] {
        assert!(
            builds_surface.contains(snippet),
            "BuildsPage must surface selected-project build-context history; missing {snippet}"
        );
    }
    assert!(
        builds.contains("BuildTaskHistoryPanel {")
            && !builds.contains("for record in root.source-build-history: BuildHistoryRow"),
        "BuildsPage should pass build-history state into BuildTaskHistoryPanel instead of owning list rows inline"
    );
    for snippet in [
        "source-build-history: root.source-build-history;",
        "source-build-history-count: root.source-build-history-count;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must forward build history rows into BuildsPage; missing {snippet}"
        );
    }
}

#[test]
fn hub_window_exposes_operation_timeline_rows_for_runtime_binding() {
    let shared = read_ui_file("shared.slint");
    let operation_timeline = read_ui_file("operation_timeline_components.slint");
    let app = read_ui_file("app.slint");

    for snippet in [
        "export struct OperationTimelineRowData",
        "action: string,",
        "status: string,",
        "finished: string,",
        "target: string,",
        "detail: string,",
        "log: string,",
        "recovery: string,",
        "command: string,",
        "output-path: string,",
        "process-id: string,",
        "success: bool,",
        "operation-timeline-empty-detail-short: string,",
    ] {
        assert!(
            shared.contains(snippet),
            "Operation timeline rows must keep a Slint struct matching the Rust view-model projection; missing {snippet}"
        );
    }

    for snippet in [
        "export component OperationTimelineRow inherits InfoRow",
        "in property <OperationTimelineRowData> record;",
        "title: root.record.action + \" / \" + root.record.target;",
        "detail: root.record.recovery != \"\" ? root.record.recovery : (root.record.log != \"\" ? root.record.log : (root.record.command != \"\" ? root.record.command : root.record.detail));",
        "trailing-text: root.record.status;",
        "collapse-trailing-label: root.collapse-label;",
    ] {
        assert!(
            operation_timeline.contains(snippet),
            "Operation timeline rows must stay available as shared Material ListTile-backed rows; missing {snippet}"
        );
    }

    for snippet in [
        "OperationTimelineRowData,",
        "in property <[OperationTimelineRowData]> operation-timeline;",
        "in property <int> operation-timeline-count;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must expose operation timeline rows for Rust binding setters; missing {snippet}"
        );
    }
}

#[test]
fn operation_timeline_rows_are_shared_between_builds_and_settings() {
    let data_display = read_ui_file("data_display.slint");
    let operation_timeline = read_ui_file("operation_timeline_components.slint");
    let components = read_ui_file("components.slint");
    let builds_components = read_ui_file("builds_page_components.slint");
    let builds = read_ui_file("builds.slint");
    let settings = read_ui_file("settings.slint");
    let app = read_ui_file("app.slint");

    for snippet in [
        "export component OperationTimelineRow inherits InfoRow",
        "in property <OperationTimelineRowData> record;",
        "title: root.record.action + \" / \" + root.record.target;",
        "detail: root.record.recovery != \"\" ? root.record.recovery : (root.record.log != \"\" ? root.record.log : (root.record.command != \"\" ? root.record.command : root.record.detail));",
        "meta: root.record.process-id != \"\" ? root.record.process-id + \" / \" + root.record.finished : root.record.finished;",
        "trailing-text: root.record.status;",
        "collapse-trailing-label: root.collapse-label;",
    ] {
        assert!(
            operation_timeline.contains(snippet),
            "OperationTimelineRow must be a shared Material ListTile-backed operation-timeline row; missing {snippet}"
        );
    }

    for snippet in [
        "export component OperationTimelinePanel inherits HubListPanelSlot",
        "in property <[OperationTimelineRowData]> operation-timeline;",
        "in property <int> operation-timeline-count;",
        "private property <length> timeline-empty-height: root.row-height + HubTokens.space-2;",
        "title: root.timeline-title;",
        "show-badge: true;",
        "badge-text: root.operation-timeline-count + \"\";",
        "scroll-y <=> root.timeline-scroll-y;",
        "row-count: root.operation-timeline-count;",
        "row-height: HubTokens.list-row-sm;",
        "vertical-padding: HubTokens.space-0;",
        "empty-height: root.timeline-empty-height;",
        "for record in root.operation-timeline: OperationTimelineRow",
        "row-height: root.row-height;",
        "if root.operation-timeline-count == 0: EmptyStateBlock",
        "height: root.timeline-empty-height;",
        "detail: root.empty-detail;",
    ] {
        assert!(
            operation_timeline.contains(snippet),
            "OperationTimelinePanel should consume the shared list-panel slot while preserving timeline rows and empty state; missing {snippet}"
        );
    }
    let timeline_panel = operation_timeline
        .split("export component OperationTimelinePanel inherits HubListPanelSlot")
        .nth(1)
        .expect("operation_timeline_components.slint must declare OperationTimelinePanel");
    for forbidden in ["PanelHeader {", "PanelListViewport {", "inherits PanelSlot"] {
        assert!(
            !timeline_panel.contains(forbidden),
            "OperationTimelinePanel should not reintroduce a local panel/list shell after moving to HubListPanelSlot: {forbidden}"
        );
    }
    assert!(
        components.contains("export { OperationTimelineRow, OperationTimelinePanel } from \"operation_timeline_components.slint\";"),
        "components.slint should re-export the shared OperationTimelinePanel from operation_timeline_components.slint"
    );
    assert!(
        !data_display.contains("OperationTimelinePanel")
            && !data_display.contains("OperationTimelineRow")
            && !data_display.contains("OperationTimelineRowData"),
        "data_display.slint should not regain operation-timeline ownership after the timeline module split"
    );
    assert!(
        !builds_components.contains("OperationTimelinePanel")
            && !builds_components.contains("OperationTimelineRow"),
        "builds_page_components.slint should not own the shared operation timeline panel or row"
    );
    assert!(
        !settings.contains("from \"builds_page_components.slint\""),
        "SettingsPage should import the shared OperationTimelinePanel through components.slint, not the Builds module"
    );

    for source in [&builds, &settings] {
        for snippet in [
            "OperationTimelinePanel {",
            "timeline-title: root.ui-text.operation-timeline;",
            "empty-title: root.ui-text.no-operation-timeline;",
            "operation-timeline: root.operation-timeline;",
            "operation-timeline-count: root.operation-timeline-count;",
        ] {
            assert!(
                source.contains(snippet),
                "Builds and Settings should both consume the shared operation timeline panel; missing {snippet}"
            );
        }
    }
    assert!(
        builds.contains("empty-detail: root.ui-text.operation-timeline-empty-detail-short;"),
        "Builds should use the short operation timeline empty copy in its compact three-panel row"
    );
    assert!(
        settings.contains("empty-detail: root.ui-text.operation-timeline-empty-detail;"),
        "Settings should keep the full operation timeline empty copy where it has a taller slot"
    );

    for snippet in [
        "in property <[OperationTimelineRowData]> operation-timeline;",
        "in property <int> operation-timeline-count;",
        "operation-timeline: root.operation-timeline;",
        "operation-timeline-count: root.operation-timeline-count;",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow should forward operation timeline rows into Builds and Settings; missing {snippet}"
        );
    }
}
