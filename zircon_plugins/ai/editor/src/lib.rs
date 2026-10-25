#[path = "authoring/mod.rs"]
mod authoring;
#[path = "capability.rs"]
mod capability;
#[path = "extension_ids.rs"]
mod extension_ids;
#[path = "overlay.rs"]
mod overlay;
#[path = "plugin.rs"]
mod plugin;
#[path = "runtime_mirror.rs"]
mod runtime_mirror;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/authoring.rs"]
mod authoring_tests;

#[cfg(test)]
#[path = "tests/gameplay_debugger.rs"]
mod gameplay_debugger_tests;

pub use authoring::{
    AiBehaviorTreeAsset, AiBehaviorTreeConnectionCandidate, AiBehaviorTreeConnectionRequest,
    AiBehaviorTreeEdgeRouteRequest, AiBehaviorTreeEdit, AiBehaviorTreeEditCandidate,
    AiBehaviorTreeEditError, AiBehaviorTreeGraphAdapter, AiBehaviorTreeGraphError,
    AiBehaviorTreeGraphGeometry, AiBehaviorTreeGraphProjection, AiBehaviorTreeMenuAdapter,
    AiBehaviorTreeMenuCommand, AiBehaviorTreeMenuDisabledReason, AiBehaviorTreeMenuError,
    AiBehaviorTreeMenuRequest, AiBehaviorTreeMenuRow, AiBehaviorTreeNodeMenuRow,
    AiBlackboardDependencies, AiBlackboardKeyMenuRow, AiBlackboardKeyReference,
    AiBlackboardKeyUsage, AiBlackboardMenuAdapter, AiBlackboardMenuCandidate,
    AiBlackboardMenuCommand, AiBlackboardMenuDisabledReason, AiBlackboardMenuError,
    AiBlackboardMenuRequest, AiBlackboardMenuRow, AiBlackboardProviderSnapshot,
    AiBlackboardReferenceBlocker, AiBlackboardReferenceKnowledge, AiBlackboardReferencePlan,
    AiBlackboardRenameCandidate, AiBlackboardRenameError, AiBlackboardSchemaEditCandidate,
    AiBlackboardSchemaInput, AiBlackboardSourceVersion, AiBlackboardTreeAssociation,
    AiBlackboardTreeInput, AiBlackboardTreeRewrite,
};

pub use capability::{
    AI_AUTHORING_CAPABILITY, AI_DEBUG_CAPABILITY, EDITOR_CAPABILITIES, PLUGIN_ID,
};
pub use extension_ids::{
    AI_BEHAVIOR_TREE_ASSET_TYPE, AI_BEHAVIOR_TREE_COMPILE_OPERATION,
    AI_BEHAVIOR_TREE_IMPORT_OPERATION, AI_BEHAVIOR_TREE_PALETTE_ID, AI_BEHAVIOR_TREE_TEMPLATE_ID,
    AI_BEHAVIOR_TREE_VALIDATE_OPERATION, AI_BEHAVIOR_TREE_VIEW_ID, AI_BT_NODE_RESULT_CONSUMER_ID,
    AI_PERCEPTION_DEBUG_TEMPLATE_ID, AI_PERCEPTION_DEBUG_VIEW_ID, AI_PERCEPTION_OVERLAY_MODE_ID,
    AI_PERCEPTION_OVERLAY_PROVIDER_ID, AI_TOGGLE_PERCEPTION_OVERLAY_OPERATION,
};
pub use overlay::{
    build_ai_perception_overlay, AiPerceptionOverlayController, AiPerceptionOverlayOptions,
    AiPerceptionViewportGizmoSink,
};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_declaration,
    editor_plugin_descriptor, package_manifest, plugin_registration, AiEditorPlugin,
};
pub use runtime_mirror::{
    ai_runtime_event_consumers, AiBtNodeResultMirror, AiBtNodeResultMirrorApply,
    AiBtNodeResultMirrorError, AiPieFrameRef, AiPieMirror, AiPieMirrorApply, AiPieMirrorError,
    AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID, AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA,
    BT_NODE_RESULT_EVENT_ID, BT_NODE_RESULT_PAYLOAD_SCHEMA,
};
