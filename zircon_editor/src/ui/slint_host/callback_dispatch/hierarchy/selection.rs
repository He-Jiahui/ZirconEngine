use zircon_scene::NodeId;

use crate::core::editor_event::{host_adapter, EditorEventRuntime};
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_hierarchy_selection(
    runtime: &EditorEventRuntime,
    node_id: NodeId,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_hierarchy_selection(node_id))
}
