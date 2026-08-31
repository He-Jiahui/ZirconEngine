mod arranged;
mod arranged_visibility;
mod binding_targets;
mod binding_transaction;
mod clipboard_transfers;
mod component_state;
mod control_index;
mod diagnostics;
mod ecs_projection;
mod focus;
mod frame_hit_test;
pub(crate) mod input;
mod interaction_gate;
mod invalidation;
mod mutation_snapshot;
mod navigation_index;
mod node_pool;
mod popup_stack;
mod property_mutation;
mod reflection_snapshot;
mod render;
mod secure_text_values;
mod session_identity;
mod slots;
mod surface;
mod text_artifact;
mod text_geometry;
mod text_shape;
mod timeline;
mod virtual_list_materialization;
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
pub(crate) use input::{editable_text_input_is_secure, text_input_constraints_for_node};
pub use input::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult, UiSurfaceInputState};
pub(crate) use interaction_gate::{ui_surface_effective_disabled, ui_surface_node_disabled};
pub use invalidation::{
    UiInvalidationApplyError, UiInvalidationChange, UiInvalidationCommit,
    UiInvalidationGenerations, UiInvalidationReason, UiInvalidationTransaction,
    UiSurfaceInvalidationApplyError, UiSurfaceInvalidationState,
};
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
pub use surface::{
    UiAuthoredGeometryFallbackReason, UiAuthoredGeometryPublication, UiSurface,
    UiSurfaceRebuildReport,
};
pub use text_artifact::{
    current_resolved_text_font_generation, resolved_text_glyph_artifact_line,
    UiResolvedTextGlyphArtifactLine, UiTextGlyphArtifactFaceSnapshot,
    UiTextGlyphArtifactRasterFace, UiTextGlyphArtifactRasterFaces,
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
