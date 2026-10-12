use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;
use zircon_runtime_interface::ui::component::{UiDragPayload, UiDragPayloadKind};

use super::super::{action_matches, action_matches_binding_suffix};

const ASSET_FIELD_CLEAR_BINDING_SUFFIX: &str = "AssetFieldClear";
const ASSET_FIELD_LOCATE_BINDING_SUFFIX: &str = "AssetFieldLocate";
const ASSET_FIELD_OPEN_BINDING_SUFFIX: &str = "AssetFieldOpen";

pub(super) fn demo_reference_input(action_id: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    match action_id {
        action if action_matches(action, "asset_field_dropped") => {
            Some(UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://materials/runtime_demo.mat",
                ),
            })
        }
        action if action_matches(action, "asset_field_drop_hovered") => {
            Some(UiComponentShowcaseDemoEventInput::DropHover(true))
        }
        action if action_matches(action, "asset_field_active_drag_target") => {
            Some(UiComponentShowcaseDemoEventInput::ActiveDragTarget(true))
        }
        action
            if action_matches_binding_suffix(action, ASSET_FIELD_CLEAR_BINDING_SUFFIX)
                || action_matches_binding_suffix(action, ASSET_FIELD_LOCATE_BINDING_SUFFIX)
                || action_matches_binding_suffix(action, ASSET_FIELD_OPEN_BINDING_SUFFIX) =>
        {
            Some(UiComponentShowcaseDemoEventInput::None)
        }
        action if action_matches(action, "instance_field_dropped") => {
            Some(UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::SceneInstance,
                    "scene://Root/RuntimeDemoLight",
                ),
            })
        }
        action if action_matches(action, "object_field_dropped") => {
            Some(UiComponentShowcaseDemoEventInput::DropReference {
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Object,
                    "object://Selection/RuntimeDemo",
                ),
            })
        }
        _ => None,
    }
}
