//! Match actual native draw commands to the actual normal submission.
use super::super::chrome_command_stream::ChromeCommandStream;
use super::super::data::{HostWindowPresentationData, TemplatePaneNodeData};
use serde_json::{json, Value};
fn identity(node: &TemplatePaneNodeData) -> Option<Value> {
    (!node.source_path.is_empty()
        && !node.source_node_id.is_empty()
        && !node.instance_path.is_empty())
    .then(|| {
        json!({"sourcePath":node.source_path.as_str(),"sourceNodeId":node.source_node_id.as_str(),
            "instancePath":node.instance_path.as_str(),"controlId":node.control_id.as_str()})
    })
}
pub(in crate::ui::retained_host::host_contract) fn bind_native_hierarchy_draws(
    stream: &ChromeCommandStream,
    presentation: &HostWindowPresentationData,
    sources: &mut [Option<Value>],
    paint: &mut Value,
) {
    let actual = super::normal_present::observed_commands(stream, sources);
    let authored = super::super::componentized_workbench_regions::authored_hierarchy(presentation);
    let owners = presentation
        .workbench_window_nodes
        .iter()
        .filter(|node| node.control_id.as_str() == "WorkbenchSceneTree")
        .filter_map(identity)
        .collect::<Vec<_>>();
    let owner = match owners.as_slice() {
        [owner] => Some(owner.clone()),
        _ => None,
    };
    let origin = authored
        .as_ref()
        .and_then(|authored| identity(authored.prototype));
    let mut used = std::collections::HashSet::new();
    let Some(draws) = paint["nativeHierarchyDraws"].as_array_mut() else {
        return;
    };
    for draw in draws {
        let admitted = authored.as_ref().is_some_and(|authored| {
            draw["dataIdentity"]["index"]
                .as_u64()
                .and_then(|index| authored.pane.hierarchy.hierarchy_nodes.get(index as usize))
                .is_some_and(|node| {
                    draw["dataIdentity"]["nodeId"].as_str() == Some(node.id.as_str())
                })
        });
        let mut indices = Vec::new();
        let mut unique = true;
        let recorded = draw["commands"].as_array().cloned().unwrap_or_default();
        for command in &recorded {
            let matches = actual
                .iter()
                .filter(|candidate| {
                    candidate["bounds"] == command["bounds"]
                        && candidate["clip"] == command["clip"]
                        && candidate["paint"] == command["paint"]
                })
                .filter_map(|candidate| candidate["commandIndex"].as_u64())
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [index] if used.insert(*index) => indices.push(*index),
                _ => unique = false,
            }
        }
        let complete = admitted
            && owner.is_some()
            && origin.is_some()
            && unique
            && !recorded.is_empty()
            && indices.len() == recorded.len();
        draw["sourceOwner"] = owner.clone().unwrap_or(Value::Null);
        draw["originComponent"] = origin.clone().unwrap_or(Value::Null);
        draw["sourceOwnerMappingComplete"] = json!(complete);
        draw["submittedCommandIndices"] = json!(indices);
        if !complete {
            continue;
        }
        for index in indices {
            let Some(slot) = sources.get_mut(index as usize) else {
                continue;
            };
            let source = slot.get_or_insert_with(|| json!({}));
            source["nativeRowIdentity"] = draw["dataIdentity"].clone();
            source["sourceOwner"] = draw["sourceOwner"].clone();
            source["originComponent"] = draw["originComponent"].clone();
            source["nativeRowBounds"] = draw["bounds"].clone();
            source["nativeRowClip"] = draw["clip"].clone();
            source["nativeIdentityAuthority"] =
                json!("actual-native-row-draw-and-submitted-command");
        }
    }
}
