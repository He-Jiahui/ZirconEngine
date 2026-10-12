#[path = "arranged.rs"]
mod arranged;
#[path = "arranged_visibility.rs"]
mod arranged_visibility;
#[path = "binding_targets.rs"]
mod binding_targets;
#[path = "binding_transaction.rs"]
mod binding_transaction;
#[path = "clipboard_transfers.rs"]
mod clipboard_transfers;
#[path = "component_state.rs"]
mod component_state;
#[path = "control_index.rs"]
mod control_index;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "ecs_projection.rs"]
mod ecs_projection;
#[path = "focus.rs"]
mod focus;
#[path = "frame_hit_test.rs"]
mod frame_hit_test;
#[path = "host_font_assets.rs"]
mod host_font_assets;
#[path = "input/mod.rs"]
pub(crate) mod input;
#[path = "interaction_gate.rs"]
mod interaction_gate;
#[path = "invalidation.rs"]
mod invalidation;
#[path = "localized_text.rs"]
mod localized_text;
#[path = "mutation_snapshot.rs"]
mod mutation_snapshot;
#[path = "navigation_index.rs"]
mod navigation_index;
#[path = "node_pool.rs"]
mod node_pool;
#[path = "popup_stack.rs"]
mod popup_stack;
#[path = "property_mutation.rs"]
mod property_mutation;
#[path = "reflection_snapshot.rs"]
mod reflection_snapshot;
#[path = "render/mod.rs"]
mod render;
#[path = "render_fonts.rs"]
mod render_fonts;
pub use render_fonts::{current_ui_surface_font_snapshot, ui_surface_font_snapshot_for_context};
#[path = "secure_text_values.rs"]
mod secure_text_values;
#[path = "session_identity.rs"]
mod session_identity;
#[path = "slots.rs"]
mod slots;
#[path = "subtree_mount.rs"]
mod subtree_mount;
#[path = "surface.rs"]
mod surface;
#[path = "text_artifact.rs"]
mod text_artifact;
#[path = "text_geometry.rs"]
mod text_geometry;
#[path = "text_shape.rs"]
mod text_shape;
#[path = "timeline.rs"]
mod timeline;
#[path = "virtual_list_materialization.rs"]
mod virtual_list_materialization;
#[path = "virtual_list_prototype_pool.rs"]
mod virtual_list_prototype_pool;

use zircon_runtime_interface::ui::{
    layout::UiSize,
    surface::{UiResolvedStyle, UiTextRange},
};

pub use crate::ui::text::layout_text;
pub(crate) use crate::ui::text::{UiTextLayoutRequest, UiTextMeasureCache};
pub(crate) use arranged::{
    arranged_bubble_route, arranged_bubble_route_indexed, arranged_effective_input_policy,
    arranged_effective_input_policy_indexed, arranged_focus_path, arranged_focus_path_indexed,
    arranged_focus_path_matches_indexed, arranged_node_indexed, arranged_node_indices,
    arranged_slot_indices, authored_geometry_affected_node_ids, build_arranged_tree,
    is_arranged_child_hit_path_visible, is_arranged_child_hit_path_visible_indexed,
    is_arranged_render_visible, is_arranged_render_visible_indexed, patch_arranged_tree_geometry,
    patch_arranged_tree_input,
};
pub(crate) use arranged_visibility::UiArrangedVisibilityIndex;
pub(crate) use binding_transaction::UiBindingMutationTransaction;
pub(crate) use clipboard_transfers::UiSurfaceClipboardTransferSnapshot;
pub use component_state::UiSurfaceComponentStateStore;
pub(crate) use control_index::UiSurfaceControlIndex;
pub use diagnostics::{
    debug_surface_frame, debug_surface_frame_for_pick, debug_surface_frame_for_selection,
    debug_surface_frame_with_options,
};
use diagnostics::{
    debug_surface_frame_for_pick_with_ecs_projection,
    debug_surface_frame_for_selection_with_ecs_projection, debug_surface_frame_with_ecs_projection,
    debug_surface_frame_with_options_and_ecs_projection,
};
pub use frame_hit_test::{
    debug_hit_test_surface_frame, debug_hit_test_surface_frame_with_query, hit_test_surface_frame,
    hit_test_surface_frame_with_query,
};
pub use host_font_assets::{UiHostFontAssetError, UiHostFontAssetReady, UiHostFontAssets};
pub(crate) use input::{editable_text_input_is_secure, text_input_constraints_for_node};
pub use input::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult, UiSurfaceInputState};
pub(crate) use interaction_gate::{ui_surface_effective_disabled, ui_surface_node_disabled};
pub use invalidation::{
    UiInvalidationApplyError, UiInvalidationChange, UiInvalidationCommit,
    UiInvalidationGenerations, UiInvalidationReason, UiInvalidationTransaction,
    UiSurfaceInvalidationApplyError, UiSurfaceInvalidationState,
};
pub use localized_text::UiLocalizedTextSynchronizationError;
pub(crate) use mutation_snapshot::{UiSurfaceMutationDomains, UiSurfaceMutationSnapshot};
pub use node_pool::{UiSurfaceNodePool, UiSurfaceNodePoolReport};
pub use property_mutation::{
    UiPropertyMutationReport, UiPropertyMutationRequest, UiPropertyMutationStatus,
};
pub use reflection_snapshot::reflector_snapshot;
pub use render::{extract_ui_render_tree, extract_ui_render_tree_from_arranged};
pub(crate) use render::{
    measure_text_with_cache, measure_text_with_fixed_width_cache, metadata_has_inline_widget,
    resolve_inline_widget_layout_with_cache, resolve_rich_text_format,
    resolve_text_layout_with_cache,
};
pub(in crate::ui) use secure_text_values::UiPendingSecureTextModelUpdateStoreHandle;
pub(crate) use secure_text_values::UiTextComponentEventKind;
pub(crate) use session_identity::UiSurfaceSessionIdentityHandle;
pub use subtree_mount::UiSurfaceChildMountProof;
pub use surface::{
    UiAuthoredGeometryFallbackReason, UiAuthoredGeometryPublication, UiSurface,
    UiSurfaceRebuildReport,
};
pub use text_artifact::{
    current_resolved_text_font_generation, resolved_text_glyph_artifact_line,
    UiResolvedTextGlyphArtifactLine, UiTextGlyphArtifactFaceSnapshot,
    UiTextGlyphArtifactRasterFace, UiTextGlyphArtifactRasterFaceReceipt,
    UiTextGlyphArtifactRasterFaces,
};
pub use text_geometry::{text_caret_frame_for_layout, text_range_frames_for_layout};
pub use text_shape::shape_text_line;
pub use timeline::UiDebugTimelineStore;
pub use virtual_list_materialization::{
    UiVirtualListItemIdentity, UiVirtualListItemKey, UiVirtualListMaterializationChange,
    UiVirtualListMaterializationError, UiVirtualListMaterializationReport,
    UiVirtualListNodeBinding,
};
pub use virtual_list_prototype_pool::{
    UiVirtualListPrototypeNodeContext, UiVirtualListPrototypePoolError,
    UiVirtualListPrototypePoolReport,
};

pub fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    crate::ui::text::measure_text_size(text, style)
}

/// Measures a source byte range after text shaping with kerning included.
pub fn measure_text_source_range_width(
    text: &str,
    style: &UiResolvedStyle,
    range: UiTextRange,
) -> f32 {
    crate::ui::text::measure_text_source_range_width(text, style, range)
}
