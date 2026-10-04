use super::super::chrome_command_stream::ChromeCommandStream;
use super::super::data::HostWindowPresentationData;
use serde_json::{json, Value};
use zircon_runtime::rhi::UiSurfaceTextLayoutSnapshot;
pub(in crate::ui::retained_host::host_contract) fn source_rows(
    stream: &ChromeCommandStream,
    presentation: &HostWindowPresentationData,
) -> Vec<Option<Value>> {
    let models = presentation.paint_node_models();
    stream.commands().iter().enumerate().map(|(index,command)| {
        let (frame,reference,fragment)=stream.resolve_command_source(index)?;
        let control=frame.arranged_tree.get(reference.node_id).and_then(|node|node.control_id.as_deref());
        let mut matches=Vec::new();
        for model in &models { for row in 0..model.row_count() { if let Some(node)=model.row_data(row) {
            if node.source_surface_frame.as_ref().is_some_and(|source| std::sync::Arc::ptr_eq(source, frame) && source.tree_id == frame.tree_id && source.generation == frame.generation) && node.surface_node_id==Some(reference.node_id) && control==Some(node.control_id.as_str()) { matches.push(node); }
        } } }
        let node=match matches.as_slice(){[node]=>Some(node),_=>None};
        Some(json!({"sourceRef":{"treeId":frame.tree_id,"generation":frame.generation,"nodeId":reference.node_id,"nodeCommandIndex":reference.node_command_index,"fragmentIndex":fragment,"authority":"surface-node-owner","primitiveIdentity":false},
            "sourcePath":node.map(|node|node.source_path.as_str()),"sourceNodeId":node.map(|node|node.source_node_id.as_str()),"instancePath":node.map(|node|node.instance_path.as_str()),"controlId":control,"propertyFieldId":node.map(|node|node.inspector_property_field_id.as_str()),"itemKey":node.map(|node|node.inspector_property_item_key.as_str()),
            "bounds":node.map(|node|json!({"x":node.frame.x,"y":node.frame.y,"width":node.frame.width,"height":node.frame.height})),"paintFragmentFrame":{"x":command.frame.x,"y":command.frame.y,"width":command.frame.width,"height":command.frame.height},"disabled":node.map(|node|node.disabled)}))
    }).collect()
}

pub(in crate::ui::retained_host::host_contract) fn bind_submitted(
    snapshot: UiSurfaceTextLayoutSnapshot,
    sources: &[Option<Value>],
    command_stream_full_rebuild: bool,
) -> Value {
    let runs=snapshot.runs.iter().map(|run|json!({"commandIndex":run.command_index,"source":sources.get(run.command_index).and_then(|source|source.as_ref()),"text":run.text,"clip":run.clip,"lines":run.lines,"faces":run.faces})).collect::<Vec<_>>();
    json!({"coverage":if snapshot.damage.is_some(){"damage-region"}else{"submitted-command-stream"},"commandStreamFullRebuild":command_stream_full_rebuild,"damage":snapshot.damage,"preparedThisPresent":snapshot.prepared_this_present,"retainedCacheCopyBytes":snapshot.retained_cache_copy_bytes,"coordinateSpace":"physical-pixels","presentedFrameCount":snapshot.presented_frame_count,"projectionSize":snapshot.projection_size,"drawListGeneration":snapshot.draw_list_generation,"runs":runs,"paintControls":sources.iter().flatten().collect::<Vec<_>>()})
}

#[cfg(test)]
#[path = "tests/submitted_text.rs"]
mod tests;
