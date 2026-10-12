//! Typed `.zui` view projection facade.
//!
//! Projection behavior is owned by the named leaf modules below. This root only
//! defines the curated editor-layout surface and keeps cache consumers off a
//! monolithic projection implementation.

#[path = "view_projection/build.rs"]
mod build;
#[path = "view_projection/component_semantics.rs"]
mod component_semantics;
#[path = "view_projection/materialization.rs"]
mod materialization;
#[path = "view_projection/metadata.rs"]
mod metadata;
#[path = "view_projection/node_projection.rs"]
mod node_projection;
#[path = "view_projection/projection_cache.rs"]
mod projection_cache;
#[path = "view_projection/projection_composition.rs"]
mod projection_composition;
#[path = "view_projection/projection_error.rs"]
mod projection_error;
#[path = "view_projection/projection_patch.rs"]
mod projection_patch;
#[path = "view_projection/resource_generation.rs"]
mod resource_generation;
#[path = "view_projection/retained_binding.rs"]
mod retained_binding;
#[path = "view_projection/store_cache.rs"]
mod store_cache;
#[path = "view_projection/visual_assets.rs"]
mod visual_assets;

pub(crate) use build::{
    build_view_template_node_projection, build_view_template_node_projection_with_patches,
    compose_view_template_node_model, UnrealDockHeaderSource,
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

#[cfg(test)]
#[path = "view_projection/tests/cases.rs"]
mod tests;
