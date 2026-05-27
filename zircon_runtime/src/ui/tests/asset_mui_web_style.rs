use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

const MUI_WEB_STYLE_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.tests.mui_web_style"
version = 1
display_name = "MUI Web Style"

[[stylesheets]]
id = "mui_web"

[[stylesheets.rules]]
selector = "Button:hovered"
set = { self = { text = "Hovered", text_tone = "warning", background = { color = "#111111" }, corner_radius = 6.0 } }

[[stylesheets.rules]]
selector = ".MuiButton-contained.MuiButton-colorPrimary"
set = { self = { validation_level = "success" } }

[[stylesheets.rules]]
selector = ".Mui-disabled"
set = { self = { text = "Disabled State", surface_variant = "danger" } }

[[stylesheets.rules]]
selector = ".Mui-readOnly"
set = { self = { text_tone = "muted" } }

[[stylesheets.rules]]
selector = ".MuiButton-startIcon.slot-extra"
set = { self = { surface_variant = "success" } }

[[stylesheets.rules]]
selector = ".MuiAlert-colorWarning"
set = { self = { validation_level = "warning" } }

[[stylesheets.rules]]
selector = ".MuiSkeleton-root.MuiSkeleton-rounded.MuiSkeleton-wave.MuiSkeleton-withChildren"
set = { self = { validation_level = "info" } }

[[stylesheets.rules]]
selector = ".MuiSnackbar-anchorOriginTopRight"
set = { self = { surface_variant = "snackbar" } }

[[stylesheets.rules]]
selector = ".MuiAlert-icon.alert-icon-extra"
set = { self = { text_tone = "warning" } }

[[stylesheets.rules]]
selector = ".MuiAlert-action.alert-action-extra"
set = { self = { text_tone = "warning" } }

[[stylesheets.rules]]
selector = ".MuiSnackbarContent-action.snackbar-action-extra"
set = { self = { text_tone = "warning" } }

[[stylesheets.rules]]
selector = ".MuiPaper-elevation.MuiPaper-rounded.MuiPaper-elevation3"
set = { self = { surface_variant = "popup" } }

[[stylesheets.rules]]
selector = ".MuiAppBar-positionFixed.MuiAppBar-colorPrimary.mui-fixed"
set = { self = { surface_variant = "primary", text_tone = "inverse" } }

[[stylesheets.rules]]
selector = ".MuiToolbar-gutters.MuiToolbar-regular"
set = { self = { text_align = "center" } }

[[stylesheets.rules]]
selector = ".MuiCardActions-spacing"
set = { self = { border_width = 2.0 } }

[[stylesheets.rules]]
selector = ".MuiCardHeader-title.card-title-extra"
set = { self = { text_tone = "info" } }

[[stylesheets.rules]]
selector = ".MuiCardMedia-media.MuiCardMedia-img"
set = { self = { overflow = "clip" } }

[[stylesheets.rules]]
selector = ".MuiCardActionArea-focusHighlight.focus-highlight-extra"
set = { self = { state_layer_enabled = true } }

[[stylesheets.rules]]
selector = ".MuiTypography-h6.MuiTypography-alignCenter.MuiTypography-gutterBottom.MuiTypography-noWrap"
set = { self = { text_tone = "info" } }

[[stylesheets.rules]]
selector = ".MuiDivider-middle.MuiDivider-vertical.MuiDivider-flexItem.MuiDivider-withChildren"
set = { self = { surface_variant = "divider" } }

[[stylesheets.rules]]
selector = ".MuiDivider-wrapper.MuiDivider-wrapperVertical"
set = { self = { text_tone = "muted" } }

[[stylesheets.rules]]
selector = ".MuiAvatar-rounded.MuiAvatar-colorDefault"
set = { self = { surface_variant = "avatar" } }

[[stylesheets.rules]]
selector = ".MuiChip-outlined.MuiChip-sizeSmall.MuiChip-colorWarning.MuiChip-clickable.MuiChip-deletable"
set = { self = { validation_level = "warning" } }

[[stylesheets.rules]]
selector = ".MuiChip-label.chip-label-extra"
set = { self = { text_tone = "info" } }

[[stylesheets.rules]]
selector = ".MuiBadge-badge.MuiBadge-dot.MuiBadge-invisible.MuiBadge-anchorOriginBottomLeftCircular.MuiBadge-overlapCircular.MuiBadge-colorError"
set = { self = { validation_level = "error" } }

[[stylesheets.rules]]
selector = ".MuiList-padding.MuiList-dense.MuiList-subheader"
set = { self = { surface_variant = "list" } }

[[stylesheets.rules]]
selector = ".MuiImageList-masonry"
set = { self = { overflow = "scroll" } }

[[stylesheets.rules]]
selector = ".MuiTable-stickyHeader"
set = { self = { z_index = 2 } }

[[stylesheets.rules]]
selector = ".MuiIcon-root.MuiIcon-colorPrimary.MuiIcon-fontSizeLarge"
set = { self = { text_tone = "icon-primary-large" } }

[[stylesheets.rules]]
selector = ".MuiSvgIcon-root.MuiSvgIcon-colorSecondary.MuiSvgIcon-fontSizeLarge.svg-icon-extra"
set = { self = { text_tone = "svg-secondary-large" } }
"##;

const MUI_WEB_SX_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_sx"
version = 1
display_name = "MUI Web SX"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "sx_button"
kind = "native"
type = "Button"
control_id = "SxButton"
props = { text = "Base", hovered = true, mui_variant = "contained", mui_color = "primary", button_size = "medium", mui_sx = { text = "SX Wins", background = { color = "#333333" }, border_width = 3.0 } }
"##;

const MUI_WEB_STATE_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_state"
version = 1
display_name = "MUI Web State"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "disabled_button"
kind = "native"
type = "Button"
control_id = "DisabledButton"
props = { text = "Base", disabled = true, button_variant = "outlined", button_color = "secondary", button_size = "small", mui_classes = ["custom-mui-class"] }
"##;

const MUI_WEB_READONLY_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_readonly"
version = 1
display_name = "MUI Web ReadOnly"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "readonly_input"
kind = "native"
type = "InputBase"
control_id = "ReadOnlyInput"
props = { value = "Locked", readOnly = true }
"##;

const MUI_WEB_ICON_UTILITY_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_icon_utility"
version = 1
display_name = "MUI Web Icon Utility Classes"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "icon_utility_root"
kind = "native"
type = "VerticalBox"
control_id = "IconUtilityRoot"

[[root.children]]
[root.children.node]
node_id = "mui_icon"
kind = "native"
type = "Icon"
control_id = "MuiIcon"
props = { icon = "folder", text = "folder", color = "primary", fontSize = "large" }

[[root.children]]
[root.children.node]
node_id = "mui_svg_icon"
kind = "native"
type = "SvgIcon"
control_id = "MuiSvgIcon"
props = { icon = "AddCircle", color = "secondary", fontSize = "large", className = "svg-icon-extra", htmlColor = "#35c7d0", viewBox = "0 0 24 24", titleAccess = "Add circle", inheritViewBox = false }
"##;

const MUI_WEB_SLOT_PROPS_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_slot_props"
version = 1
display_name = "MUI Web Slot Props"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "slot_button"
kind = "native"
type = "Button"
control_id = "SlotButton"
props = { text = "Base", mui_slot_props = { root = { disabled = true }, startIcon = { text = "Slot Prop", mui_sx = { text_tone = "info" }, mui_classes = ["slot-extra"] } }, mui_slots = { startIcon = "IconButton" } }

[[root.children]]
mount = "startIcon"
[root.children.node]
node_id = "start_icon"
kind = "native"
type = "Label"
control_id = "StartIcon"
props = { text = "Original" }
"##;

const MUI_WEB_NATIVE_CUSTOMIZATION_ALIAS_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_native_aliases"
version = 1
display_name = "MUI Web Native Aliases"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "native_alias_button"
kind = "native"
type = "Button"
control_id = "NativeAliasButton"
props = { text = "Base", variant = "contained", color = "secondary", size = "small", className = "root-extra root-alias", classes = { root = "classes-root", startIcon = ["classes-start"] }, sx = { text = "SX Alias Wins", background = { color = "#444444" }, border_width = 4.0 }, slotProps = { root = { disabled = true }, startIcon = { text = "Plain Slot", sx = { text_tone = "info" }, className = "slot-extra slot-class" } }, slots = { startIcon = "IconButton" } }

[[root.children]]
mount = "startIcon"
[root.children.node]
node_id = "native_start_icon"
kind = "native"
type = "Label"
control_id = "NativeStartIcon"
props = { text = "Icon" }
"##;

const MUI_WEB_FEEDBACK_UTILITY_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_feedback_utility"
version = 1
display_name = "MUI Web Feedback Utility Classes"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "feedback_root"
kind = "native"
type = "VerticalBox"
control_id = "FeedbackRoot"

[[root.children]]
[root.children.node]
node_id = "feedback_alert"
kind = "native"
type = "Alert"
control_id = "FeedbackAlert"
props = { text = "Warning", severity = "warning", variant = "filled", action = "Fix", slotProps = { icon = { className = "alert-icon-extra", text = "!" }, action = { className = "alert-action-extra", text = "Fix" } } }

[[root.children.node.children]]
mount = "icon"
[root.children.node.children.node]
node_id = "feedback_alert_icon"
kind = "native"
type = "Label"
control_id = "FeedbackAlertIcon"
props = { text = "Icon" }

[[root.children.node.children]]
mount = "action"
[root.children.node.children.node]
node_id = "feedback_alert_action"
kind = "native"
type = "Button"
control_id = "FeedbackAlertAction"
props = { text = "Fix" }

[[root.children]]
[root.children.node]
node_id = "feedback_snackbar"
kind = "native"
type = "Snackbar"
control_id = "FeedbackSnackbar"
props = { open = true, message = "Saved", anchorOrigin = { vertical = "top", horizontal = "right" } }

[[root.children]]
[root.children.node]
node_id = "feedback_default_snackbar"
kind = "native"
type = "Snackbar"
control_id = "FeedbackDefaultSnackbar"
props = { open = true, message = "Queued" }

[[root.children]]
[root.children.node]
node_id = "feedback_snackbar_content"
kind = "native"
type = "SnackbarContent"
control_id = "FeedbackSnackbarContent"
props = { message = "Content", slotProps = { action = { className = "snackbar-action-extra", text = "Undo" } } }

[[root.children.node.children]]
mount = "action"
[root.children.node.children.node]
node_id = "feedback_snackbar_action"
kind = "native"
type = "Button"
control_id = "FeedbackSnackbarAction"
props = { text = "Undo" }

[[root.children]]
[root.children.node]
node_id = "feedback_alert_title"
kind = "native"
type = "AlertTitle"
control_id = "FeedbackAlertTitle"
props = { text = "Heads up" }

[[root.children]]
[root.children.node]
node_id = "feedback_skeleton"
kind = "native"
type = "Skeleton"
control_id = "FeedbackSkeleton"
props = { variant = "rounded", animation = "wave" }

[[root.children.node.children]]
[root.children.node.children.node]
node_id = "feedback_skeleton_child"
kind = "native"
type = "Label"
control_id = "FeedbackSkeletonChild"
props = { text = "Loading" }
"##;

const MUI_WEB_SURFACE_UTILITY_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_surface_utility"
version = 1
display_name = "MUI Web Surface Utility Classes"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "surface_root"
kind = "native"
type = "VerticalBox"
control_id = "SurfaceRoot"

[[root.children]]
[root.children.node]
node_id = "surface_paper"
kind = "native"
type = "Paper"
control_id = "SurfacePaper"
props = { elevation = 3.0 }

[[root.children]]
[root.children.node]
node_id = "surface_outlined_paper"
kind = "native"
type = "Paper"
control_id = "SurfaceOutlinedPaper"
props = { variant = "outlined", square = true }

[[root.children]]
[root.children.node]
node_id = "surface_app_bar"
kind = "native"
type = "AppBar"
control_id = "SurfaceAppBar"
props = { }

[[root.children]]
[root.children.node]
node_id = "surface_toolbar"
kind = "native"
type = "Toolbar"
control_id = "SurfaceToolbar"
props = { }

[[root.children]]
[root.children.node]
node_id = "surface_card"
kind = "native"
type = "Card"
control_id = "SurfaceCard"
props = { variant = "outlined", raised = true }

[[root.children]]
[root.children.node]
node_id = "surface_card_header"
kind = "native"
type = "CardHeader"
control_id = "SurfaceCardHeader"
props = { title = "Scene", subheader = "Ready", slotProps = { title = { className = "card-title-extra", text = "Slot Title" } } }

[[root.children.node.children]]
mount = "title"
[root.children.node.children.node]
node_id = "surface_card_header_title"
kind = "native"
type = "Label"
control_id = "SurfaceCardHeaderTitle"
props = { text = "Scene" }

[[root.children]]
[root.children.node]
node_id = "surface_card_actions"
kind = "native"
type = "CardActions"
control_id = "SurfaceCardActions"
props = { }

[[root.children]]
[root.children.node]
node_id = "surface_card_media"
kind = "native"
type = "CardMedia"
control_id = "SurfaceCardMedia"
props = { component = "img", image = "res://textures/albedo.png" }

[[root.children]]
[root.children.node]
node_id = "surface_card_action_area"
kind = "native"
type = "CardActionArea"
control_id = "SurfaceCardActionArea"
props = { focused = true, focusVisibleClassName = "keyboard-focus", slotProps = { focusHighlight = { className = "focus-highlight-extra" } } }

[[root.children.node.children]]
mount = "focusHighlight"
[root.children.node.children.node]
node_id = "surface_focus_highlight"
kind = "native"
type = "Label"
control_id = "SurfaceFocusHighlight"
props = { text = "" }
"##;

const MUI_WEB_DATA_DISPLAY_UTILITY_LAYOUT_TOML: &str = r##"
[asset]
kind = "layout"
id = "ui.tests.mui_web_data_display_utility"
version = 1
display_name = "MUI Web Data Display Utility Classes"

[imports]
styles = ["asset://ui/tests/mui_web_style.ui"]

[root]
node_id = "data_display_root"
kind = "native"
type = "VerticalBox"
control_id = "DataDisplayRoot"

[[root.children]]
[root.children.node]
node_id = "data_typography"
kind = "native"
type = "Typography"
control_id = "DataTypography"
props = { text = "Title", variant = "h6", align = "center", gutterBottom = true, noWrap = true }

[[root.children]]
[root.children.node]
node_id = "data_divider"
kind = "native"
type = "Divider"
control_id = "DataDivider"
props = { text = "Meta", variant = "middle", orientation = "vertical", flexItem = true }

[[root.children.node.children]]
mount = "wrapper"
[root.children.node.children.node]
node_id = "data_divider_wrapper"
kind = "native"
type = "Label"
control_id = "DataDividerWrapper"
props = { text = "Meta" }

[[root.children]]
[root.children.node]
node_id = "data_avatar"
kind = "native"
type = "Avatar"
control_id = "DataAvatar"
props = { text = "A", variant = "rounded" }

[[root.children]]
[root.children.node]
node_id = "data_chip"
kind = "native"
type = "Chip"
control_id = "DataChip"
props = { label = "Warn", variant = "outlined", size = "small", color = "warning", clickable = true, onDelete = "MaterialLab.Chip.Delete", deleteIcon = "cancel", slotProps = { label = { className = "chip-label-extra", text = "Styled Warn" } } }

[[root.children.node.children]]
mount = "label"
[root.children.node.children.node]
node_id = "data_chip_label"
kind = "native"
type = "Label"
control_id = "DataChipLabel"
props = { text = "Warn" }

[[root.children.node.children]]
mount = "deleteIcon"
[root.children.node.children.node]
node_id = "data_chip_delete_icon"
kind = "native"
type = "Icon"
control_id = "DataChipDeleteIcon"
props = { icon = "cancel" }

[[root.children]]
[root.children.node]
node_id = "data_badge"
kind = "native"
type = "Badge"
control_id = "DataBadge"
props = { variant = "dot", color = "error", invisible = true, overlap = "circular", anchorOrigin = { vertical = "bottom", horizontal = "left" }, slotProps = { badge = { text = "" } } }

[[root.children.node.children]]
mount = "badge"
[root.children.node.children.node]
node_id = "data_badge_slot"
kind = "native"
type = "Label"
control_id = "DataBadgeSlot"
props = { text = "" }

[[root.children]]
[root.children.node]
node_id = "data_list"
kind = "native"
type = "List"
control_id = "DataList"
props = { dense = true, subheader = "Group" }

[[root.children]]
[root.children.node]
node_id = "data_image_list"
kind = "native"
type = "ImageList"
control_id = "DataImageList"
props = { variant = "masonry", cols = 3, gap = 6.0 }

[[root.children]]
[root.children.node]
node_id = "data_table"
kind = "native"
type = "Table"
control_id = "DataTable"
props = { stickyHeader = true }
"##;

#[test]
fn mui_sx_merges_as_high_priority_style_override_and_state_selectors_match() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_SX_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(str_attr(root, "text"), Some("SX Wins"));
    assert_eq!(str_attr(root, "text_tone"), Some("warning"));
    assert_eq!(str_attr(root, "validation_level"), Some("success"));
    assert_eq!(float_attr(root, "border_width"), Some(3.0));
    assert_eq!(float_attr(root, "corner_radius"), Some(6.0));
    assert_eq!(table_str_attr(root, "background", "color"), Some("#333333"));

    assert_eq!(
        root.style_overrides.get("text").and_then(Value::as_str),
        Some("SX Wins")
    );
    assert_eq!(
        root.style_overrides
            .get("background")
            .and_then(|background| background.get("color"))
            .and_then(Value::as_str),
        Some("#333333")
    );
    assert_eq!(
        root.style_overrides
            .get("border_width")
            .and_then(Value::as_float),
        Some(3.0)
    );

    assert_classes(
        root,
        &[
            "MuiButton-root",
            "MuiButton-contained",
            "MuiButton-colorPrimary",
            "MuiButton-sizeMedium",
            "Mui-hovered",
        ],
    );
}

#[test]
fn mui_state_classes_match_stylesheet_rules() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_STATE_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(str_attr(root, "text"), Some("Disabled State"));
    assert_eq!(str_attr(root, "surface_variant"), Some("danger"));
    assert_classes(
        root,
        &[
            "MuiButton-root",
            "MuiButton-outlined",
            "MuiButton-colorSecondary",
            "MuiButton-sizeSmall",
            "Mui-disabled",
            "custom-mui-class",
        ],
    );
}

#[test]
fn mui_readonly_alias_generates_mui_state_class() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_READONLY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(bool_attr(root, "readOnly"), Some(true));
    assert_eq!(str_attr(root, "text_tone"), Some("muted"));
    assert_classes(root, &["MuiInputBase-root", "Mui-readOnly"]);
}

#[test]
fn mui_icon_utility_classes_match_local_mui_selectors() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_ICON_UTILITY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let icon = &root.children[0];
    let svg_icon = &root.children[1];

    assert_eq!(str_attr(icon, "text_tone"), Some("icon-primary-large"));
    assert_classes(
        icon,
        &[
            "MuiIcon-root",
            "MuiIcon-colorPrimary",
            "MuiIcon-fontSizeLarge",
        ],
    );
    assert_no_classes(icon, &["MuiIcon-sizeMedium", "MuiIcon-default"]);

    assert_eq!(str_attr(svg_icon, "text_tone"), Some("svg-secondary-large"));
    assert_classes(
        svg_icon,
        &[
            "MuiSvgIcon-root",
            "MuiSvgIcon-colorSecondary",
            "MuiSvgIcon-fontSizeLarge",
            "svg-icon-extra",
        ],
    );
    assert_no_classes(svg_icon, &["MuiSvgIcon-sizeMedium", "MuiSvgIcon-default"]);
}

#[test]
fn mui_slot_props_apply_to_root_and_named_slot_children() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_SLOT_PROPS_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let child = root.children.first().expect("start icon child");

    assert_eq!(bool_attr(root, "disabled"), Some(true));
    assert_classes(root, &["MuiButton-root", "Mui-disabled"]);

    assert_eq!(
        child
            .slot_attributes
            .get("mui_slot")
            .and_then(Value::as_str),
        Some("startIcon")
    );
    assert_eq!(str_attr(child, "text"), Some("Slot Prop"));
    assert_eq!(str_attr(child, "text_tone"), Some("info"));
    assert_eq!(str_attr(child, "surface_variant"), Some("success"));
    assert_eq!(str_attr(child, "mui_slot_component"), Some("IconButton"));
    assert_classes(
        child,
        &["MuiLabel-root", "MuiButton-startIcon", "slot-extra"],
    );
}

#[test]
fn mui_native_customization_aliases_match_web_prop_names() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout =
        UiAssetLoader::load_toml_str(MUI_WEB_NATIVE_CUSTOMIZATION_ALIAS_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let child = root.children.first().expect("start icon child");

    assert_eq!(str_attr(root, "text"), Some("SX Alias Wins"));
    assert_eq!(float_attr(root, "border_width"), Some(4.0));
    assert_eq!(bool_attr(root, "disabled"), Some(true));
    assert_eq!(table_str_attr(root, "background", "color"), Some("#444444"));
    assert_classes(
        root,
        &[
            "MuiButton-root",
            "MuiButton-contained",
            "MuiButton-colorSecondary",
            "MuiButton-sizeSmall",
            "Mui-disabled",
            "root-extra",
            "root-alias",
            "classes-root",
        ],
    );

    assert_eq!(str_attr(child, "text"), Some("Plain Slot"));
    assert_eq!(str_attr(child, "text_tone"), Some("info"));
    assert_eq!(str_attr(child, "surface_variant"), Some("success"));
    assert_eq!(str_attr(child, "mui_slot_component"), Some("IconButton"));
    assert_classes(
        child,
        &[
            "MuiLabel-root",
            "MuiButton-startIcon",
            "slot-extra",
            "slot-class",
            "classes-start",
        ],
    );
}

#[test]
fn mui_feedback_utility_classes_match_alert_and_snackbar_selectors() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_FEEDBACK_UTILITY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let alert = &root.children[0];
    let snackbar = &root.children[1];
    let default_snackbar = &root.children[2];
    let snackbar_content = &root.children[3];
    let alert_title = &root.children[4];
    let skeleton = &root.children[5];

    assert_eq!(str_attr(alert, "validation_level"), Some("warning"));
    assert!(
        str_attr(alert, "component_variant").is_some_and(|value| {
            ["filled", "warning", "colorWarning", "hasIcon", "hasAction"]
                .iter()
                .all(|token| value.split_whitespace().any(|part| part == *token))
        }),
        "Alert root should carry retained severity and slot metadata"
    );
    assert_classes(
        alert,
        &["MuiAlert-root", "MuiAlert-filled", "MuiAlert-colorWarning"],
    );
    assert!(
        !alert
            .classes
            .iter()
            .any(|class_name| class_name == "MuiAlert-filledWarning"),
        "MUI v9 Alert utility classes no longer emit variant+severity combo classes"
    );
    assert!(
        !alert
            .classes
            .iter()
            .any(|class_name| class_name == "MuiAlert-colorPrimary"
                || class_name == "MuiAlert-sizeMedium"),
        "Alert should not inherit generic MUI color/size classes that local Alert.js does not emit"
    );
    let alert_icon = &alert.children[0];
    assert_eq!(str_attr(alert_icon, "text_tone"), Some("warning"));
    assert_classes(alert_icon, &["MuiAlert-icon", "alert-icon-extra"]);
    assert!(
        str_attr(alert_icon, "component_variant").is_some_and(|value| value
            .split_whitespace()
            .all(|part| part != "")
            && ["muiAlertSlot", "alertSlotIcon"]
                .iter()
                .all(|token| value.split_whitespace().any(|part| part == *token))),
        "Alert icon slot should carry retained hide metadata"
    );
    let alert_action = &alert.children[1];
    assert_eq!(str_attr(alert_action, "text_tone"), Some("warning"));
    assert_classes(alert_action, &["MuiAlert-action", "alert-action-extra"]);
    assert!(
        str_attr(alert_action, "component_variant").is_some_and(|value| [
            "muiAlertSlot",
            "alertSlotAction"
        ]
        .iter()
        .all(|token| value.split_whitespace().any(|part| part == *token))),
        "Alert action slot should carry retained hide metadata"
    );

    assert_eq!(str_attr(snackbar, "surface_variant"), Some("snackbar"));
    assert_classes(
        snackbar,
        &[
            "MuiSnackbar-root",
            "MuiSnackbar-anchorOriginTopRight",
            "Mui-open",
        ],
    );
    assert!(
        !snackbar
            .classes
            .iter()
            .any(|class_name| class_name == "MuiSnackbar-colorPrimary"
                || class_name == "MuiSnackbar-sizeMedium"),
        "Snackbar should only emit root and anchor utility classes"
    );
    assert_classes(
        default_snackbar,
        &[
            "MuiSnackbar-root",
            "MuiSnackbar-anchorOriginBottomLeft",
            "Mui-open",
        ],
    );
    assert_classes(snackbar_content, &["MuiSnackbarContent-root"]);
    assert!(
        !snackbar_content
            .classes
            .iter()
            .any(|class_name| class_name == "MuiSnackbarContent-colorPrimary"
                || class_name == "MuiSnackbarContent-sizeMedium"),
        "SnackbarContent should not inherit generic MUI color/size classes"
    );
    let snackbar_action = &snackbar_content.children[0];
    assert_eq!(str_attr(snackbar_action, "text_tone"), Some("warning"));
    assert_classes(
        snackbar_action,
        &["MuiSnackbarContent-action", "snackbar-action-extra"],
    );
    assert_classes(alert_title, &["MuiAlertTitle-root"]);
    assert!(
        !alert_title
            .classes
            .iter()
            .any(|class_name| class_name == "MuiAlertTitle-colorPrimary"
                || class_name == "MuiAlertTitle-sizeMedium"),
        "AlertTitle should only emit local MUI root utility classes"
    );

    assert_eq!(str_attr(skeleton, "validation_level"), Some("info"));
    assert_classes(
        skeleton,
        &[
            "MuiSkeleton-root",
            "MuiSkeleton-rounded",
            "MuiSkeleton-wave",
            "MuiSkeleton-withChildren",
            "MuiSkeleton-fitContent",
            "MuiSkeleton-heightAuto",
        ],
    );
    assert_no_classes(
        skeleton,
        &["MuiSkeleton-colorPrimary", "MuiSkeleton-sizeMedium"],
    );
    assert!(
        str_attr(skeleton, "component_variant").is_some_and(|value| {
            [
                "rounded",
                "wave",
                "withChildren",
                "fitContent",
                "heightAuto",
            ]
            .iter()
            .all(|token| value.split_whitespace().any(|part| part == *token))
        }),
        "Skeleton root should carry retained painter metadata"
    );
    let skeleton_child = &skeleton.children[0];
    assert!(
        str_attr(skeleton_child, "component_variant").is_some_and(|value| value
            .split_whitespace()
            .any(|part| part == "muiSkeletonChild")),
        "Skeleton child should carry retained painter hide metadata"
    );
    assert_classes(skeleton_child, &["MuiLabel-root"]);
}

#[test]
fn mui_surface_utility_classes_match_paper_card_and_app_bar_selectors() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_SURFACE_UTILITY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let paper = &root.children[0];
    let outlined_paper = &root.children[1];
    let app_bar = &root.children[2];
    let toolbar = &root.children[3];
    let card = &root.children[4];
    let card_header = &root.children[5];
    let card_actions = &root.children[6];
    let card_media = &root.children[7];
    let card_action_area = &root.children[8];

    assert_eq!(str_attr(paper, "surface_variant"), Some("popup"));
    assert_classes(
        paper,
        &[
            "MuiPaper-root",
            "MuiPaper-elevation",
            "MuiPaper-rounded",
            "MuiPaper-elevation3",
        ],
    );
    assert_no_classes(paper, &["MuiPaper-colorPrimary", "MuiPaper-sizeMedium"]);

    assert_classes(outlined_paper, &["MuiPaper-root", "MuiPaper-outlined"]);
    assert_no_classes(outlined_paper, &["MuiPaper-rounded", "MuiPaper-elevation1"]);

    assert_eq!(str_attr(app_bar, "surface_variant"), Some("primary"));
    assert_eq!(str_attr(app_bar, "text_tone"), Some("inverse"));
    assert_classes(
        app_bar,
        &[
            "MuiAppBar-root",
            "MuiAppBar-colorPrimary",
            "MuiAppBar-positionFixed",
            "mui-fixed",
        ],
    );
    assert_no_classes(app_bar, &["MuiAppBar-sizeMedium"]);

    assert_eq!(str_attr(toolbar, "text_align"), Some("center"));
    assert_classes(
        toolbar,
        &[
            "MuiToolbar-root",
            "MuiToolbar-gutters",
            "MuiToolbar-regular",
        ],
    );
    assert_no_classes(
        toolbar,
        &["MuiToolbar-colorPrimary", "MuiToolbar-sizeMedium"],
    );

    assert_classes(card, &["MuiCard-root"]);
    assert_no_classes(
        card,
        &[
            "MuiCard-outlined",
            "MuiCard-colorPrimary",
            "MuiCard-sizeMedium",
        ],
    );

    assert_classes(card_header, &["MuiCardHeader-root"]);
    assert_no_classes(
        card_header,
        &["MuiCardHeader-colorPrimary", "MuiCardHeader-sizeMedium"],
    );
    let card_title = &card_header.children[0];
    assert_eq!(str_attr(card_title, "text"), Some("Slot Title"));
    assert_eq!(str_attr(card_title, "text_tone"), Some("info"));
    assert_classes(
        card_title,
        &["MuiLabel-root", "MuiCardHeader-title", "card-title-extra"],
    );

    assert_eq!(float_attr(card_actions, "border_width"), Some(2.0));
    assert_classes(
        card_actions,
        &["MuiCardActions-root", "MuiCardActions-spacing"],
    );
    assert_no_classes(
        card_actions,
        &["MuiCardActions-colorPrimary", "MuiCardActions-sizeMedium"],
    );

    assert_eq!(str_attr(card_media, "overflow"), Some("clip"));
    assert_classes(
        card_media,
        &[
            "MuiCardMedia-root",
            "MuiCardMedia-media",
            "MuiCardMedia-img",
        ],
    );
    assert_no_classes(
        card_media,
        &["MuiCardMedia-colorPrimary", "MuiCardMedia-sizeMedium"],
    );

    assert_classes(
        card_action_area,
        &[
            "MuiCardActionArea-root",
            "MuiCardActionArea-focusVisible",
            "keyboard-focus",
        ],
    );
    assert_no_classes(
        card_action_area,
        &[
            "MuiCardActionArea-colorPrimary",
            "MuiCardActionArea-sizeMedium",
        ],
    );
    let focus_highlight = &card_action_area.children[0];
    assert_eq!(
        bool_attr(focus_highlight, "state_layer_enabled"),
        Some(true)
    );
    assert_classes(
        focus_highlight,
        &["MuiCardActionArea-focusHighlight", "focus-highlight-extra"],
    );
}

#[test]
fn mui_data_display_utility_classes_match_local_mui_selectors() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_DATA_DISPLAY_UTILITY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let typography = &root.children[0];
    let divider = &root.children[1];
    let avatar = &root.children[2];
    let chip = &root.children[3];
    let badge = &root.children[4];
    let list = &root.children[5];
    let image_list = &root.children[6];
    let table = &root.children[7];

    assert_eq!(str_attr(typography, "text_tone"), Some("info"));
    assert_classes(
        typography,
        &[
            "MuiTypography-root",
            "MuiTypography-h6",
            "MuiTypography-alignCenter",
            "MuiTypography-gutterBottom",
            "MuiTypography-noWrap",
        ],
    );
    assert_no_classes(
        typography,
        &["MuiTypography-colorPrimary", "MuiTypography-sizeMedium"],
    );

    assert_eq!(str_attr(divider, "surface_variant"), Some("divider"));
    assert_classes(
        divider,
        &[
            "MuiDivider-root",
            "MuiDivider-middle",
            "MuiDivider-vertical",
            "MuiDivider-flexItem",
            "MuiDivider-withChildren",
        ],
    );
    let divider_wrapper = &divider.children[0];
    assert_eq!(str_attr(divider_wrapper, "text_tone"), Some("muted"));
    assert_classes(
        divider_wrapper,
        &["MuiDivider-wrapper", "MuiDivider-wrapperVertical"],
    );

    assert_eq!(str_attr(avatar, "surface_variant"), Some("avatar"));
    assert_classes(
        avatar,
        &[
            "MuiAvatar-root",
            "MuiAvatar-rounded",
            "MuiAvatar-colorDefault",
        ],
    );
    assert_no_classes(avatar, &["MuiAvatar-colorPrimary", "MuiAvatar-sizeMedium"]);

    assert_eq!(str_attr(chip, "validation_level"), Some("warning"));
    assert_classes(
        chip,
        &[
            "MuiChip-root",
            "MuiChip-outlined",
            "MuiChip-sizeSmall",
            "MuiChip-colorWarning",
            "MuiChip-clickable",
            "MuiChip-deletable",
        ],
    );
    let chip_label = &chip.children[0];
    assert_eq!(str_attr(chip_label, "text"), Some("Styled Warn"));
    assert_eq!(str_attr(chip_label, "text_tone"), Some("info"));
    assert_classes(
        chip_label,
        &["MuiChip-label", "chip-label-extra", "MuiLabel-root"],
    );
    assert!(
        str_attr(chip, "component_variant").is_some_and(|value| {
            value.contains("hasDeleteIcon") && value.contains("colorWarning")
        }),
        "Chip root should carry retained painter metadata"
    );
    let chip_delete_icon = &chip.children[1];
    assert!(
        str_attr(chip_delete_icon, "component_variant").is_some_and(|value| {
            value.contains("muiChipSlot") && value.contains("chipSlotDeleteIcon")
        }),
        "Chip deleteIcon slot should carry retained painter metadata"
    );
    assert_classes(chip_delete_icon, &["MuiChip-deleteIcon", "MuiIcon-root"]);

    assert_classes(badge, &["MuiBadge-root"]);
    assert_no_classes(badge, &["MuiBadge-dot", "MuiBadge-colorError"]);
    let badge_slot = &badge.children[0];
    assert_eq!(str_attr(badge_slot, "validation_level"), Some("error"));
    assert!(
        str_attr(badge_slot, "component_variant")
            .is_some_and(|value| value.contains("muiBadgeSlot") && value.contains("invisible")),
        "Badge slot should carry retained painter metadata"
    );
    assert_classes(
        badge_slot,
        &[
            "MuiBadge-badge",
            "MuiBadge-dot",
            "MuiBadge-invisible",
            "MuiBadge-anchorOriginBottomLeft",
            "MuiBadge-anchorOriginBottomLeftCircular",
            "MuiBadge-overlapCircular",
            "MuiBadge-colorError",
        ],
    );

    assert_eq!(str_attr(list, "surface_variant"), Some("list"));
    assert_classes(
        list,
        &[
            "MuiList-root",
            "MuiList-padding",
            "MuiList-dense",
            "MuiList-subheader",
        ],
    );
    assert_no_classes(list, &["MuiList-colorPrimary", "MuiList-sizeMedium"]);

    assert_eq!(str_attr(image_list, "overflow"), Some("scroll"));
    assert_classes(image_list, &["MuiImageList-root", "MuiImageList-masonry"]);
    assert_no_classes(
        image_list,
        &["MuiImageList-colorPrimary", "MuiImageList-sizeMedium"],
    );

    assert_eq!(int_attr(table, "z_index"), Some(2));
    assert_classes(table, &["MuiTable-root", "MuiTable-stickyHeader"]);
    assert_no_classes(table, &["MuiTable-colorPrimary", "MuiTable-sizeMedium"]);
}

fn str_attr<'a>(node: &'a UiTemplateNode, name: &str) -> Option<&'a str> {
    node.attributes.get(name).and_then(Value::as_str)
}

fn bool_attr(node: &UiTemplateNode, name: &str) -> Option<bool> {
    node.attributes.get(name).and_then(Value::as_bool)
}

fn float_attr(node: &UiTemplateNode, name: &str) -> Option<f64> {
    node.attributes.get(name).and_then(Value::as_float)
}

fn int_attr(node: &UiTemplateNode, name: &str) -> Option<i64> {
    node.attributes.get(name).and_then(Value::as_integer)
}

fn table_str_attr<'a>(node: &'a UiTemplateNode, table: &str, name: &str) -> Option<&'a str> {
    node.attributes
        .get(table)
        .and_then(|value| value.get(name))
        .and_then(Value::as_str)
}

fn assert_classes(node: &UiTemplateNode, expected: &[&str]) {
    for class_name in expected {
        assert!(
            node.classes.iter().any(|value| value == class_name),
            "missing {class_name} in {:?}",
            node.classes
        );
    }
}

fn assert_no_classes(node: &UiTemplateNode, unexpected: &[&str]) {
    for class_name in unexpected {
        assert!(
            !node.classes.iter().any(|value| value == class_name),
            "unexpected {class_name} in {:?}",
            node.classes
        );
    }
}
