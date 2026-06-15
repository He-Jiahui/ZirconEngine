use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::{
    UiComponentState, UiValidationLevel, UiValidationState, UiValue,
};

pub(super) fn component_id_for_control(control_id: &str) -> Option<&'static str> {
    match control_id {
        "LabelDemo" => Some("Label"),
        "RichLabelDemo" => Some("RichLabel"),
        "ImageDemo" => Some("Image"),
        "IconDemo" => Some("Icon"),
        "SvgIconDemo" => Some("SvgIcon"),
        "SeparatorDemo" => Some("Separator"),
        "ProgressBarDemo" => Some("ProgressBar"),
        "SpinnerDemo" => Some("Spinner"),
        "BadgeDemo" => Some("Badge"),
        "HelpRowDemo" => Some("HelpRow"),
        "ButtonDemo" | "ButtonOutlinedDemo" | "ButtonTextDemo" | "ButtonDangerDemo"
        | "ButtonDisabledDemo" => Some("Button"),
        "IconButtonDemo" => Some("IconButton"),
        "ToggleButtonDemo" => Some("ToggleButton"),
        "CheckboxDemo" => Some("Checkbox"),
        "RadioDemo" => Some("Radio"),
        "SegmentedControlDemo" => Some("SegmentedControl"),
        "TabDemo" => Some("Tab"),
        "TabStripDemo" => Some("Tabs"),
        "InputFieldDemo" => Some("InputField"),
        "TextFieldDemo" => Some("TextField"),
        "NumberFieldDemo" => Some("NumberField"),
        "RangeFieldDemo" => Some("RangeField"),
        "SliderDemo" => Some("Slider"),
        "RangeSliderDemo" => Some("RangeSlider"),
        "ColorFieldDemo" => Some("ColorField"),
        "Vector2FieldDemo" => Some("Vector2Field"),
        "Vector3FieldDemo" => Some("Vector3Field"),
        "Vector4FieldDemo" => Some("Vector4Field"),
        "DropdownDemo" => Some("Dropdown"),
        "ComboBoxDemo" => Some("ComboBox"),
        "EnumFieldDemo" => Some("EnumField"),
        "FlagsFieldDemo" => Some("FlagsField"),
        "SearchSelectDemo" => Some("SearchSelect"),
        "ContextMenuDemo" => Some("ContextMenu"),
        "DropdownPopupDemo" => Some("DropdownPopup"),
        "SkeletonDemo" => Some("Skeleton"),
        "DialogDemo" => Some("Dialog"),
        "ConfirmDialogDemo" => Some("ConfirmDialog"),
        "CommandPaletteDemo" => Some("CommandPalette"),
        "NotificationCenterDemo" => Some("NotificationCenter"),
        "AssetFieldDemo" => Some("AssetField"),
        "InstanceFieldDemo" => Some("InstanceField"),
        "ObjectFieldDemo" => Some("ObjectField"),
        "GroupDemo" => Some("Group"),
        "FoldoutDemo" => Some("Foldout"),
        "PropertyRowDemo" => Some("PropertyRow"),
        "InspectorSectionDemo" => Some("InspectorSection"),
        "ArrayFieldDemo" => Some("ArrayField"),
        "MapFieldDemo" => Some("MapField"),
        "ListRowDemo" => Some("ListRow"),
        "TableRowDemo" => Some("TableRow"),
        "VirtualListDemo" => Some("VirtualList"),
        "PagedListDemo" => Some("PagedList"),
        "WorldSpaceSurfaceDemo" => Some("WorldSpaceSurface"),
        "TreeRowDemo" => Some("TreeRow"),
        "ContextActionMenuDemo" => Some("ContextActionMenu"),
        _ => None,
    }
}

pub(super) fn default_state_for_control(control_id: &str) -> UiComponentState {
    match control_id {
        "NumberFieldDemo" => UiComponentState::new().with_value("value", UiValue::Float(42.0)),
        "RangeFieldDemo" => UiComponentState::new().with_value("value", UiValue::Float(68.0)),
        "SliderDemo" => UiComponentState::new()
            .with_value("value", UiValue::Float(42.0))
            .with_value("value_percent", UiValue::Float(0.42)),
        "RangeSliderDemo" => UiComponentState::new()
            .with_value("value", UiValue::Float(72.0))
            .with_value("range_min", UiValue::Float(28.0))
            .with_value("value_percent", UiValue::Float(0.72))
            .with_value("range_min_percent", UiValue::Float(0.28))
            .with_value("focused_thumb", UiValue::Enum("upper".to_string())),
        "ColorFieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Color("#4d89ff".to_string()))
        }
        "Vector2FieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Vec2([12.0, 24.0]))
        }
        "Vector3FieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Vec3([0.0, 1.0, 0.0]))
        }
        "Vector4FieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Vec4([0.0, 1.0, 0.0, 1.0]))
        }
        "DropdownDemo" => UiComponentState::new()
            .with_value("value", UiValue::Enum("runtime".to_string()))
            .with_value("multiple", UiValue::Bool(true))
            .with_value(
                "disabled_options",
                UiValue::Array(vec![UiValue::String("debug".to_string())]),
            ),
        "ComboBoxDemo" => {
            UiComponentState::new().with_value("value", UiValue::Enum("material".to_string()))
        }
        "EnumFieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Enum("RiderDocking".to_string()))
        }
        "FlagsFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::Flags(vec!["Selectable".to_string(), "Draggable".to_string()]),
        ),
        "SearchSelectDemo" => UiComponentState::new()
            .with_value("value", UiValue::Enum("runtime.ui.NumberField".to_string()))
            .with_value("query", UiValue::String("number".to_string())),
        "DialogDemo" => UiComponentState::new()
            .with_value("open", UiValue::Bool(true))
            .with_value("popup_open", UiValue::Bool(true))
            .with_value("title", UiValue::String("Scene Settings".to_string()))
            .with_value(
                "message",
                UiValue::String("Review scene-level settings before applying them.".to_string()),
            )
            .with_value("action", UiValue::String("Apply".to_string())),
        "ConfirmDialogDemo" => {
            let mut state = UiComponentState::new()
                .with_value("open", UiValue::Bool(true))
                .with_value("popup_open", UiValue::Bool(true))
                .with_value(
                    "title",
                    UiValue::String("Delete selected prefab?".to_string()),
                )
                .with_value(
                    "message",
                    UiValue::String(
                        "This removes the prefab reference from the scene.".to_string(),
                    ),
                )
                .with_value("confirm_text", UiValue::String("Delete".to_string()))
                .with_value("cancel_text", UiValue::String("Cancel".to_string()))
                .with_value("severity", UiValue::String("error".to_string()))
                .with_value("validation_level", UiValue::String("error".to_string()))
                .with_value("destructive", UiValue::Bool(true))
                .with_value("confirm_enabled", UiValue::Bool(false));
            state.validation = UiValidationState {
                level: UiValidationLevel::Error,
                message: None,
            };
            state
        }
        "CommandPaletteDemo" => UiComponentState::new()
            .with_value("open", UiValue::Bool(true))
            .with_value("popup_open", UiValue::Bool(true))
            .with_value("query", UiValue::String("build".to_string()))
            .with_value(
                "placeholder",
                UiValue::String("Search commands".to_string()),
            )
            .with_value("command_source", UiValue::String("workbench".to_string()))
            .with_value(
                "commands",
                UiValue::Array(vec![
                    command_palette_command(
                        "open_scene",
                        "Open Scene",
                        "workbench",
                        "Ctrl+O",
                        false,
                    ),
                    command_palette_command(
                        "build_project",
                        "Build Project",
                        "workbench",
                        "Ctrl+B",
                        false,
                    ),
                    command_palette_command(
                        "build_assets",
                        "Build Assets",
                        "workbench",
                        "Ctrl+Shift+B",
                        true,
                    ),
                    command_palette_command(
                        "reload_runtime",
                        "Reload Runtime",
                        "runtime",
                        "Ctrl+R",
                        false,
                    ),
                ]),
            )
            .with_value(
                "filtered_commands",
                UiValue::Array(vec![
                    UiValue::String("build_project".to_string()),
                    UiValue::String("build_assets".to_string()),
                ]),
            )
            .with_value(
                "disabled_commands",
                UiValue::Array(vec![UiValue::String("build_assets".to_string())]),
            )
            .with_value(
                "selected_command_id",
                UiValue::String("build_project".to_string()),
            )
            .with_value("focused_index", UiValue::Int(0)),
        "NotificationCenterDemo" => UiComponentState::new()
            .with_value("open", UiValue::Bool(true))
            .with_value("popup_open", UiValue::Bool(true))
            .with_value("title", UiValue::String("Notifications".to_string()))
            .with_value(
                "empty_text",
                UiValue::String("No notifications".to_string()),
            )
            .with_value("visible_limit", UiValue::Int(2))
            .with_value("unread_count", UiValue::Int(2))
            .with_value("keyboard_navigation", UiValue::Bool(true))
            .with_value(
                "selected_notification_id",
                UiValue::String("build".to_string()),
            )
            .with_value("focused_index", UiValue::Int(1))
            .with_value(
                "notifications",
                UiValue::Array(vec![
                    notification_center_notification(
                        "build",
                        "Build failed",
                        "Shader compile error",
                        "error",
                        true,
                        false,
                    ),
                    notification_center_notification(
                        "asset",
                        "Asset import complete",
                        "StoneWall.mesh ready",
                        "success",
                        true,
                        false,
                    ),
                    notification_center_notification(
                        "source",
                        "Source control synced",
                        "No local conflicts",
                        "info",
                        false,
                        true,
                    ),
                ]),
            ),
        "AssetFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::AssetRef("res://textures/grid.albedo.png".to_string()),
        ),
        "InstanceFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::InstanceRef("scene://Root/CameraRig".to_string()),
        ),
        "ObjectFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::InstanceRef("object://Selection/MainCamera".to_string()),
        ),
        "PropertyRowDemo" => UiComponentState::new()
            .with_value("value", UiValue::String("Label + Field".to_string())),
        "GroupDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(true)),
        "FoldoutDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(false)),
        "InspectorSectionDemo" => {
            UiComponentState::new().with_value("expanded", UiValue::Bool(true))
        }
        "ArrayFieldDemo" => UiComponentState::new().with_value(
            "items",
            UiValue::Array(vec![
                UiValue::String("Label".to_string()),
                UiValue::String("NumberField".to_string()),
                UiValue::String("AssetField".to_string()),
            ]),
        ),
        "MapFieldDemo" => {
            let mut entries = BTreeMap::new();
            entries.insert("speed".to_string(), UiValue::Float(1.0));
            entries.insert("visible".to_string(), UiValue::Bool(true));
            UiComponentState::new().with_value("entries", UiValue::Map(entries))
        }
        "ToggleButtonDemo" | "CheckboxDemo" => {
            UiComponentState::new().with_value("value", UiValue::Bool(true))
        }
        "RadioDemo" => UiComponentState::new().with_value("value", UiValue::Bool(false)),
        "TabDemo" | "TabStripDemo" => {
            UiComponentState::new().with_value("value", UiValue::Enum("scene".to_string()))
        }
        "ListRowDemo" => {
            UiComponentState::new().with_value("value", UiValue::String("selected".to_string()))
        }
        "VirtualListDemo" => UiComponentState::new()
            .with_value(
                "data_source",
                UiValue::String("showcase.large_items".to_string()),
            )
            .with_value("total_count", UiValue::Int(10000))
            .with_value("viewport_start", UiValue::Int(0))
            .with_value("viewport_count", UiValue::Int(25))
            .with_value("item_extent", UiValue::Float(28.0))
            .with_value("overscan", UiValue::Int(4))
            .with_value("selected_index", UiValue::Int(-1)),
        "PagedListDemo" => UiComponentState::new()
            .with_value(
                "data_source",
                UiValue::String("showcase.large_items".to_string()),
            )
            .with_value("total_count", UiValue::Int(10000))
            .with_value("page_index", UiValue::Int(0))
            .with_value("page_size", UiValue::Int(100))
            .with_value("page_count", UiValue::Int(100)),
        "WorldSpaceSurfaceDemo" => UiComponentState::new()
            .with_value("world_position", UiValue::Vec3([0.0, 1.5, 3.0]))
            .with_value("world_rotation", UiValue::Vec3([0.0, 180.0, 0.0]))
            .with_value("world_scale", UiValue::Vec3([1.0, 1.0, 1.0]))
            .with_value("world_size", UiValue::Vec2([2.0, 1.0]))
            .with_value("pixels_per_meter", UiValue::Float(256.0))
            .with_value("billboard", UiValue::Bool(true))
            .with_value("depth_test", UiValue::Bool(true))
            .with_value("render_order", UiValue::Int(0))
            .with_value(
                "camera_target",
                UiValue::String("viewport-main".to_string()),
            ),
        "TreeRowDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(true)),
        "ContextActionMenuDemo" => {
            UiComponentState::new().with_value("value", UiValue::String("Inspect".to_string()))
        }
        _ => UiComponentState::new(),
    }
}

fn command_palette_command(
    id: &str,
    label: &str,
    source: &str,
    shortcut: &str,
    disabled: bool,
) -> UiValue {
    let mut command = BTreeMap::new();
    command.insert("id".to_string(), UiValue::String(id.to_string()));
    command.insert("label".to_string(), UiValue::String(label.to_string()));
    command.insert("source".to_string(), UiValue::String(source.to_string()));
    command.insert(
        "shortcut".to_string(),
        UiValue::String(shortcut.to_string()),
    );
    command.insert("disabled".to_string(), UiValue::Bool(disabled));
    UiValue::Map(command)
}

fn notification_center_notification(
    id: &str,
    title: &str,
    message: &str,
    tone: &str,
    unread: bool,
    disabled: bool,
) -> UiValue {
    let mut notification = BTreeMap::new();
    notification.insert("id".to_string(), UiValue::String(id.to_string()));
    notification.insert("title".to_string(), UiValue::String(title.to_string()));
    notification.insert("message".to_string(), UiValue::String(message.to_string()));
    notification.insert("tone".to_string(), UiValue::String(tone.to_string()));
    notification.insert("unread".to_string(), UiValue::Bool(unread));
    notification.insert("disabled".to_string(), UiValue::Bool(disabled));
    UiValue::Map(notification)
}
