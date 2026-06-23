use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

mod data_display;
mod feedback;
mod slots_native;
mod state_icons;
mod surface;

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
