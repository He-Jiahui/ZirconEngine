//! Captured host projection metadata; never an authored or live UiTree property.
use crate::ui::template_runtime::{
    RetainedUiHostNodeModel, RetainedUiHostProjection, RetainedUiHostValue,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiFrame, surface::UiSurfaceFrame};

const ROOT: &str = "UnrealViewportToolbar";
const MORE: &str = "UnrealViewportToolbarMore";
const POPUP: &str = "UnrealViewportToolbarOverflow";

/// Frozen metadata prepared after source semantic and geometry worksets merge.
/// This changes no UiTree property, host patch workset, cursor or redraw request.
pub(super) fn prepare(
    projection: &RetainedUiHostProjection,
    surface: &UiSurface,
) -> Option<(std::sync::Arc<UiSurfaceFrame>, std::sync::Arc<Value>)> {
    let frame = surface.surface_frame();
    let value = capture(projection, surface, &frame)?;
    Some((frame, std::sync::Arc::new(value)))
}
// Diagnostic labels are evaluated only in cfg(test). Both macros retain the
// exact Option/boolean admission boundary and never manufacture metadata.
macro_rules! toolbar_some {
    ($value:expr, $($label:tt)*) => {
        match $value {
            Some(value) => value,
            None => {
                #[cfg(test)] eprintln!("viewport-toolbar first-reject: {}", format_args!($($label)*));
                return None;
            }
        }
    };
}
macro_rules! toolbar_require {
    ($condition:expr, $($label:tt)*) => {
        if !$condition {
            #[cfg(test)] eprintln!("viewport-toolbar first-reject: {}", format_args!($($label)*));
            return None;
        }
    };
}

fn capture(
    projection: &RetainedUiHostProjection,
    surface: &UiSurface,
    frame: &UiSurfaceFrame,
) -> Option<Value> {
    let mut by_id = BTreeMap::new();
    for node in &projection.nodes {
        let Some(id) = node.surface_node_id else {
            continue;
        };
        toolbar_require!(
            by_id.insert(id, node).is_none(),
            "duplicate surface node {:?}",
            id
        );
    }
    let root = toolbar_some!(
        unique_control(projection, ROOT),
        "missing or duplicate root {}",
        ROOT
    );
    let root_id = toolbar_some!(root.surface_node_id, "root has no surface node id");
    let row_control = match toolbar_some!(
        root.properties.get("toolbar_overflow_row_control"),
        "missing row control property"
    ) {
        RetainedUiHostValue::String(value) => value,
        _ => {
            toolbar_require!(false, "row control is not a string");
            unreachable!()
        }
    };
    let row = toolbar_some!(
        unique_control(projection, row_control),
        "missing or duplicate row {}",
        row_control
    );
    let more = toolbar_some!(
        unique_control(projection, MORE),
        "missing or duplicate More"
    );
    let popup = toolbar_some!(
        unique_control(projection, POPUP),
        "missing or duplicate popup"
    );
    let controls = match toolbar_some!(
        root.properties.get("toolbar_overflow_group_controls"),
        "missing group controls property"
    ) {
        RetainedUiHostValue::Array(values) => values,
        _ => {
            toolbar_require!(false, "group controls are not an array");
            unreachable!()
        }
    };
    toolbar_require!(
        !controls.is_empty() && controls.len() <= 32,
        "group count {}",
        controls.len()
    );
    let mut group_ids = Vec::with_capacity(controls.len());
    let mut seen_controls = BTreeSet::new();
    for control in controls {
        let RetainedUiHostValue::String(control) = control else {
            toolbar_require!(false, "group control is not a string");
            unreachable!()
        };
        toolbar_require!(
            seen_controls.insert(control),
            "duplicate group control {}",
            control
        );
        let group = toolbar_some!(
            unique_control(projection, control),
            "missing or duplicate group {}",
            control
        );
        let group_id = toolbar_some!(group.surface_node_id, "group {} lacks surface id", control);
        let group_frame = toolbar_some!(
            frame.arranged_tree.get(group_id),
            "group {} lacks arranged node",
            control
        );
        let parent = toolbar_some!(
            group_frame.parent,
            "group {} lacks arranged parent",
            control
        );
        toolbar_require!(
            parent == toolbar_some!(row.surface_node_id, "row lacks surface id")
                || parent == toolbar_some!(popup.surface_node_id, "popup lacks surface id"),
            "group {} parent {:?} is outside row/popup",
            control,
            parent
        );
        group_ids.push(group.node_id.clone());
    }
    for node in [row, more, popup] {
        let id = toolbar_some!(
            node.surface_node_id,
            "owner {:?} lacks surface id",
            node.control_id
        );
        toolbar_require!(
            toolbar_some!(
                under(frame, id, root_id),
                "owner {:?} has invalid arranged ancestry",
                node.control_id
            ),
            "owner {:?} is outside root",
            node.control_id
        );
    }
    let mut stack = vec![root_id];
    let mut seen = BTreeSet::new();
    let mut nodes = Vec::new();
    while let Some(id) = stack.pop() {
        toolbar_require!(
            seen.insert(id) && seen.len() <= 512,
            "cycle, duplicate or excessive arranged descendants at {:?}",
            id
        );
        let arranged = toolbar_some!(
            frame.arranged_tree.get(id),
            "missing arranged node {:?}",
            id
        );
        let raw = *toolbar_some!(
            by_id.get(&id),
            "missing raw projection for {:?} {}",
            id,
            arranged.node_path.0
        );
        toolbar_require!(
            raw.node_id == arranged.node_path.0 && raw.control_id == arranged.control_id,
            "raw/arranged identity mismatch at {:?}",
            id
        );
        let live = toolbar_some!(surface.tree.node(id), "missing live source {:?}", id);
        toolbar_require!(
            live.node_path == arranged.node_path
                && live.parent == arranged.parent
                && live.children == arranged.children,
            "live/arranged topology mismatch at {:?} {:?}",
            id,
            raw.control_id
        );
        let metadata = toolbar_some!(
            live.template_metadata.as_ref(),
            "missing source metadata at {:?} {:?}",
            id,
            raw.control_id
        );
        toolbar_require!(
            raw.source_path == metadata.source_path
                && raw.source_node_id == metadata.source_node_id
                && raw.instance_path == metadata.instance_path
                && raw.parent_id
                    == arranged
                        .parent
                        .and_then(|id| frame.arranged_tree.get(id))
                        .map(|node| node.node_path.0.clone()),
            "raw/source identity or parent mismatch at {:?} {:?}",
            id,
            raw.control_id
        );
        // A cached row from an earlier source revision cannot be stamped with a
        // newer geometry frame. State and displayed-text properties must agree
        // with the same prepared source metadata, including absent-vs-present.
        for key in [
            "selected",
            "checked",
            "popup_open",
            "value",
            "value_text",
            "text",
            "label",
            "visibility",
            "visible",
            "disabled",
            "enabled",
            "selection_state",
            "placeholder",
        ] {
            toolbar_require!(
                same_primitive(raw.properties.get(key), metadata.attributes.get(key)),
                "property mismatch at {:?} {:?} key={} raw={:?} source={:?} style={:?}",
                id,
                raw.control_id,
                key,
                raw.properties.get(key),
                metadata.attributes.get(key),
                metadata.style_overrides.get(key)
            );
        }
        let source_disabled = metadata
            .attributes
            .get("disabled")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
            || metadata
                .attributes
                .get("enabled")
                .and_then(toml::Value::as_bool)
                == Some(false);
        let source_text = metadata
            .attributes
            .get("text")
            .and_then(toml::Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                (![
                    "placeholder",
                    "value",
                    "value_text",
                    "options",
                    "items",
                    "entries",
                ]
                .iter()
                .any(|key| metadata.attributes.contains_key(*key)))
                .then(|| {
                    metadata
                        .attributes
                        .get("label")
                        .and_then(toml::Value::as_str)
                        .filter(|value| !value.is_empty())
                        .map(str::to_owned)
                })
                .flatten()
            });
        let source_value_text = metadata
            .attributes
            .get("value_text")
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                metadata
                    .attributes
                    .get("value")
                    .or_else(|| metadata.attributes.get("items"))
                    .or_else(|| metadata.attributes.get("entries"))
                    .map(zircon_runtime_interface::ui::component::UiValue::from_toml)
                    .map(|value| value.display_text())
            });
        toolbar_require!(
            raw.disabled == source_disabled,
            "derived disabled mismatch at {:?} {:?}: raw={} source={}",
            id,
            raw.control_id,
            raw.disabled,
            source_disabled
        );
        toolbar_require!(
            raw.text == source_text,
            "derived text mismatch at {:?} {:?}: raw={:?} source={:?}",
            id,
            raw.control_id,
            raw.text,
            source_text
        );
        toolbar_require!(
            raw.value_text == source_value_text,
            "derived value_text mismatch at {:?} {:?}: raw={:?} source={:?}",
            id,
            raw.control_id,
            raw.value_text,
            source_value_text
        );
        toolbar_require!(
            raw.popup_open
                == metadata
                    .attributes
                    .get("popup_open")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
            "derived popup_open mismatch at {:?} {:?}",
            id,
            raw.control_id
        );
        toolbar_require!(
            raw.checked
                == metadata
                    .attributes
                    .get("checked")
                    .and_then(toml::Value::as_bool)
                    .or_else(|| metadata
                        .attributes
                        .get("value")
                        .and_then(toml::Value::as_bool))
                    .unwrap_or(false),
            "derived checked mismatch at {:?} {:?}",
            id,
            raw.control_id
        );
        let parent = arranged.parent.and_then(|id| by_id.get(&id).copied());
        let source_path = toolbar_some!(
            raw.source_path.as_ref().filter(|value| !value.is_empty()),
            "missing source path at {:?} {:?}",
            id,
            raw.control_id
        );
        let source_node_id = toolbar_some!(
            raw.source_node_id
                .as_ref()
                .filter(|value| !value.is_empty()),
            "missing source node id at {:?} {:?}",
            id,
            raw.control_id
        );
        let source_instance = toolbar_some!(
            raw.instance_path.as_ref(),
            "missing source instance path at {:?} {:?}",
            id,
            raw.control_id
        );
        let instance_path = toolbar_some!(
            serde_json::to_string(source_instance).ok(),
            "unserializable source instance path at {:?}",
            id
        );
        let scroll = match live.scroll_state {
            Some(scroll) => {
                toolbar_require!(
                    [scroll.offset, scroll.viewport_extent, scroll.content_extent]
                        .iter()
                        .all(|value| value.is_finite() && *value >= 0.0),
                    "non-finite or negative scroll at {:?} {:?}",
                    id,
                    raw.control_id
                );
                Some(
                    json!({"offset":scroll.offset,"viewportExtent":scroll.viewport_extent,"contentExtent":scroll.content_extent}),
                )
            }
            None => None,
        };
        nodes.push(json!({
            "nodeId":raw.node_id,"parentNodeId":arranged.parent.and_then(|id| frame.arranged_tree.get(id)).map(|node| node.node_path.0.clone()),
            "sourcePath":source_path,"sourceNodeId":source_node_id,"instancePath":instance_path,
            "parentSourcePath":parent.and_then(|node| node.source_path.as_ref()),
            "parentSourceNodeId":parent.and_then(|node| node.source_node_id.as_ref()),
            "parentInstancePath":parent.and_then(|node| node.instance_path.as_ref()).and_then(|path| serde_json::to_string(path).ok()),
            "controlId":raw.control_id,"component":raw.component,
            "visible":toolbar_some!(visible(frame,id), "invalid visibility ancestry at {:?}", id),"bounds":toolbar_some!(bounds(arranged.frame), "invalid arranged bounds at {:?} {:?}: {:?}", id, raw.control_id, arranged.frame),"clip":toolbar_some!(bounds(arranged.clip_frame), "invalid arranged clip at {:?} {:?}: {:?}", id, raw.control_id, arranged.clip_frame),
            "state":{"selected":boolean(raw,"selected"),"checked":boolean(raw,"checked"),"popupOpen":boolean(raw,"popup_open"),
                "value":toolbar_some!(primitive(raw.properties.get("value")), "unsupported value at {:?} {:?}: {:?}", id, raw.control_id, raw.properties.get("value")),"text":raw.text,"valueText":raw.value_text,"disabled":raw.disabled},
            "scroll":scroll
        }));
        stack.extend(arranged.children.iter().rev().copied());
    }
    Some(
        json!({"schema":"dev.zircon.editor.viewport-toolbar-publication","version":1,
        "authority":"prepared-source-surface","observed":true,"coordinateSpace":"logical-pixels",
        "surfaceGeneration":frame.generation,"surfaceTreeId":frame.tree_id,
        "rootNodeId":root.node_id,"rowNodeId":row.node_id,"moreNodeId":more.node_id,"popupNodeId":popup.node_id,
        "groupNodeIds":group_ids,"nodes":nodes}),
    )
}

fn unique_control<'a>(
    projection: &'a RetainedUiHostProjection,
    control: &str,
) -> Option<&'a RetainedUiHostNodeModel> {
    let mut nodes = projection
        .nodes
        .iter()
        .filter(|node| node.control_id.as_deref() == Some(control));
    let node = nodes.next()?;
    nodes.next().is_none().then_some(node)
}

fn boolean(node: &RetainedUiHostNodeModel, key: &str) -> Option<bool> {
    match node.properties.get(key) {
        Some(RetainedUiHostValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn primitive(value: Option<&RetainedUiHostValue>) -> Option<Value> {
    Some(match value {
        None => Value::Null,
        Some(RetainedUiHostValue::Bool(value)) => json!(value),
        Some(RetainedUiHostValue::String(value)) => json!(value),
        Some(RetainedUiHostValue::Integer(value)) => json!(value),
        Some(RetainedUiHostValue::Float(value)) if value.is_finite() => json!(value),
        _ => return None,
    })
}

fn bounds(frame: UiFrame) -> Option<Value> {
    ([frame.x, frame.y, frame.width, frame.height]
        .iter()
        .all(|value| value.is_finite())
        && frame.width >= 0.0
        && frame.height >= 0.0)
        .then(|| json!({"x":frame.x,"y":frame.y,"width":frame.width,"height":frame.height}))
}

fn under(frame: &UiSurfaceFrame, mut id: UiNodeId, ancestor: UiNodeId) -> Option<bool> {
    let mut seen = BTreeSet::new();
    loop {
        if id == ancestor {
            return Some(true);
        }
        if !seen.insert(id) {
            return None;
        }
        match frame.arranged_tree.get(id)?.parent {
            Some(parent) => id = parent,
            None => return Some(false),
        }
    }
}

fn visible(frame: &UiSurfaceFrame, mut id: UiNodeId) -> Option<bool> {
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(id) {
            return None;
        }
        let node = frame.arranged_tree.get(id)?;
        if !node.is_render_visible() {
            return Some(false);
        }
        match node.parent {
            Some(parent) => id = parent,
            None => return Some(true),
        }
    }
}

fn same_primitive(raw: Option<&RetainedUiHostValue>, source: Option<&toml::Value>) -> bool {
    match (raw, source) {
        (None, None) => true,
        (Some(RetainedUiHostValue::Bool(a)), Some(toml::Value::Boolean(b))) => a == b,
        (Some(RetainedUiHostValue::String(a)), Some(toml::Value::String(b))) => a == b,
        (Some(RetainedUiHostValue::Integer(a)), Some(toml::Value::Integer(b))) => a == b,
        (Some(RetainedUiHostValue::Float(a)), Some(toml::Value::Float(b))) => {
            a.is_finite() && b.is_finite() && a == b
        }
        _ => false,
    }
}
