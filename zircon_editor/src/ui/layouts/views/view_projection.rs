//! Typed `.zui` view projection facade.
//!
//! Projection behavior is owned by the named leaf modules below. This root only
//! defines the curated editor-layout surface and keeps cache consumers off a
//! monolithic projection implementation.

mod build;
mod component_semantics;
mod materialization;
mod metadata;
mod node_projection;
mod projection_cache;
mod projection_composition;
mod projection_error;
mod projection_patch;
mod resource_generation;
mod retained_binding;
mod store_cache;
mod visual_assets;

pub(crate) use build::{
    build_view_template_node_projection, build_view_template_node_projection_with_patches,
    compose_view_template_node_model,
};
#[cfg(test)]
pub(crate) use build::{
    build_view_template_nodes, build_view_template_nodes_with_imports,
    clear_view_template_projection_caches_for_tests,
};
pub(crate) use component_semantics::{
    default_transition_duration_ms, default_transition_easing, preferred_binding_id,
    resolve_commit_action_id, resolve_component_role, resolve_component_variant,
    resolve_edit_action_id, resolve_node_popup_open, resolve_node_value_number,
    resolve_node_value_percent, resolve_node_value_text, resolve_transition_in,
    resolve_transition_kind, resolve_transition_progress,
};
pub(crate) use node_projection::ViewTemplateNodeProjection;
pub(crate) use projection_composition::AssetWorkspaceProjectionGeneration;
pub(crate) use projection_error::ViewTemplateProjectionError;
pub(crate) use projection_patch::ViewTemplateNodePatch;
pub(crate) use resource_generation::{
    view_template_resource_generation, ViewTemplateResourceGeneration,
};
pub(crate) use visual_assets::{resolve_visual_assets, ViewTemplateVisualAssets};

// Internal contracts retained by the cache and materialization owners.
use component_semantics::{icon_button_hides_label, resolve_role};
use materialization::{
    view_template_projection_row_signatures, ViewTemplateNodeMaterialization,
    ViewTemplateProjectionRowSignature,
};
use metadata::{
    bool_attribute, integer_attribute, number_attribute, string_array_attribute, string_attribute,
    text_align_name, value_to_display_text,
};
use retained_binding::{
    text_binding_for_metadata, ViewTemplateTextBinding, ViewTemplateTextOverrideSemantics,
};
use visual_assets::resolve_visual_assets_for_generation;

#[cfg(test)]
mod tests;
