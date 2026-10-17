//! Evidence for Source views consumed by the normal retained Header painter.
//! No document is reloaded and no observed geometry is copied into a projection.
use crate::ui::layouts::views::UnrealDockHeaderSource;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::{FrameRect, HostWindowPresentationData, TemplatePaneNodeData};
use serde_json::{json, Value};
use std::sync::Arc;
use zircon_runtime::ui::v2::{UiV2SourceFileReceipt, UiV2UnresolvedSourceImport};

const HEADER_SOURCE: &str = "zircon_editor/assets/ui/editor/unreal/components/dock_header.zui";

pub(super) struct CapturedHeaders {
    pub(super) snapshots: Vec<Value>,
    pub(super) document_ids: Vec<String>,
    pub(super) receipts: Vec<UiV2SourceFileReceipt>,
    pub(super) unresolved_imports: Vec<UiV2UnresolvedSourceImport>,
}

pub(super) fn capture(
    presentation: &HostWindowPresentationData,
) -> Result<CapturedHeaders, String> {
    let mut output = CapturedHeaders {
        snapshots: Vec::new(),
        document_ids: Vec::new(),
        receipts: Vec::new(),
        unresolved_imports: Vec::new(),
    };
    let scene = &presentation.host_scene_data;
    for dock in scene.document_surfaces() {
        append(
            &mut output,
            "document",
            dock.surface_key.as_str(),
            &dock.header_nodes,
            &dock.region_frame,
            &dock.header_frame,
        )?;
    }
    let dock = &scene.bottom_dock;
    append(
        &mut output,
        "bottom",
        dock.surface_key.as_str(),
        &dock.header_nodes,
        &dock.region_frame,
        &dock.header_frame,
    )?;
    for window in scene.floating_layer.floating_windows.iter() {
        append(
            &mut output,
            "floating",
            window.window_id.as_str(),
            &window.header_nodes,
            &window.frame,
            &window.header_frame,
        )?;
    }
    for window in presentation
        .native_floating_surface_data
        .floating_windows
        .iter()
    {
        append(
            &mut output,
            "nativeFloating",
            window.window_id.as_str(),
            &window.header_nodes,
            &window.frame,
            &window.header_frame,
        )?;
    }
    Ok(output)
}

fn visible(frame: &FrameRect) -> bool {
    frame.width.is_finite() && frame.height.is_finite() && frame.width > 0.0 && frame.height > 0.0
}

fn rect(frame: &FrameRect) -> Value {
    json!({"x": frame.x, "y": frame.y, "width": frame.width, "height": frame.height})
}

fn append(
    output: &mut CapturedHeaders,
    kind: &str,
    surface_key: &str,
    nodes: &ModelRc<TemplatePaneNodeData>,
    region: &FrameRect,
    header: &FrameRect,
) -> Result<(), String> {
    // These are the same visible region/header guards as draw_panel_header.
    if !visible(region) || !visible(header) {
        return Ok(());
    }
    let source = nodes
        .metadata::<Option<Arc<UnrealDockHeaderSource>>>()
        .and_then(Option::as_ref)
        .ok_or_else(|| {
            format!("{kind} Header {surface_key:?} did not consume an authored Unreal Source view")
        })?;
    if nodes.row_count() == 0 {
        return Err(format!("{kind} Header has no consumed Source rows"));
    }
    let mut rows = Vec::with_capacity(nodes.row_count());
    for node in nodes.iter() {
        let frame = node
            .source_surface_frame
            .as_ref()
            .filter(|frame| Arc::ptr_eq(frame, &source.frame))
            .ok_or_else(|| format!("Header {} lost its exact Source frame", node.control_id))?;
        let id = node
            .surface_node_id
            .ok_or("Header row has no actual Source node id")?;
        let reference = node.surface_render_command_ref;
        if reference.is_some_and(|reference| {
            reference.node_id != id || frame.render_extract.command_by_ref(reference).is_none()
        }) || frame.arranged_tree.get(id).is_none()
            || node.source_path.as_str() != HEADER_SOURCE
            || node.source_node_id.is_empty()
            || node.instance_path.is_empty()
        {
            return Err(format!(
                "Header {} has unresolved Source identity/render provenance",
                node.control_id
            ));
        }
        let instance_path: Value = serde_json::from_str(node.instance_path.as_str())
            .map_err(|error| format!("Header instance path is invalid: {error}"))?;
        rows.push(json!({
            "controlId": node.control_id.as_str(),
            "sourcePath": node.source_path.as_str(),
            "sourceNodeId": node.source_node_id.as_str(),
            "instancePath": instance_path,
            "role": node.role.as_str(),
            "text": node.text.as_str(),
            "selected": node.selected,
            "hovered": node.hovered,
            "pressed": node.pressed,
            "disabled": node.disabled,
            "frame": {"x":node.frame.x,"y":node.frame.y,"width":node.frame.width,"height":node.frame.height},
            "renderRef": reference,
        }));
    }
    let mut fingerprints = std::collections::BTreeMap::new();
    for receipt in &source.source_receipts {
        let resource = receipt.resource_uri.as_deref().unwrap_or(&receipt.asset_id);
        let relative = resource.strip_prefix("res://").ok_or_else(|| {
            format!("Header receipt has no source-addressable resource URI: {resource}")
        })?;
        let path = format!("zircon_editor/assets/{relative}");
        if let Some(previous) = fingerprints.insert(path, receipt.sha256.clone()) {
            if previous != receipt.sha256 {
                return Err("Header closure contains conflicting source fingerprints".into());
            }
        }
    }
    if !fingerprints.contains_key(HEADER_SOURCE) {
        return Err("Header capture lacks its actual root file receipt".into());
    }
    let source_fingerprints = fingerprints
        .into_iter()
        .map(|(source_path, sha256)| json!({"sourcePath":source_path,"sha256":sha256}))
        .collect::<Vec<_>>();
    output.snapshots.push(json!({
        "kind": kind, "surfaceKey": surface_key,
        "assetId": source.document.asset.id,
        "sourcePath": HEADER_SOURCE,
        "sourceDocument": source.document.as_ref(),
        "viewport": {"width":source.viewport.width,"height":source.viewport.height},
        "mount": {"region":rect(region),"header":rect(header)},
        "sourceFingerprints": source_fingerprints,
        "rows": rows,
    }));
    output.document_ids.push(source.document.asset.id.clone());
    output.receipts.extend(source.source_receipts.clone());
    output
        .unresolved_imports
        .extend(source.unresolved_imports.clone());
    Ok(())
}
