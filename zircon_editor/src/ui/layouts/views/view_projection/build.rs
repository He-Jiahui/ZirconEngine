use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::ui::v2::UiV2SurfaceBuilder;
use zircon_runtime_interface::ui::{event_ui::UiTreeId, layout::UiSize};

use super::super::ViewTemplateNodeData;
use super::{
    materialization::view_template_nodes_from_surface, node_projection::ViewTemplateNodeProjection,
    projection_cache, projection_composition, projection_error::ViewTemplateProjectionError,
    projection_patch::ViewTemplateNodePatch, store_cache,
};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::v2_design_tokens::{
    active_editor_v2_design_tokens_snapshot, prepare_editor_v2_document_with_tokens,
};

pub(crate) fn compose_view_template_node_model<K, M, F>(
    composition_id: &str,
    projection: ViewTemplateNodeProjection,
    generation: &K,
    compose: F,
) -> ModelRc<ViewTemplateNodeData>
where
    K: 'static + Clone + PartialEq,
    M: 'static,
    F: FnOnce(&mut Vec<ViewTemplateNodeData>) -> M,
{
    projection_composition::compose_model(composition_id, projection, generation, compose)
}

#[cfg(test)]
pub(crate) fn clear_view_template_projection_caches_for_tests() {
    projection_cache::clear_for_tests();
    projection_composition::clear_for_tests();
}

pub(crate) fn build_view_template_node_projection(
    document_tree_id: &str,
    layout_asset_path: &str,
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
) -> Result<ViewTemplateNodeProjection, ViewTemplateProjectionError> {
    build_view_template_node_projection_with_patches(
        document_tree_id,
        layout_asset_path,
        style_imports,
        size,
        text_overrides,
        &BTreeMap::new(),
    )
}

pub(crate) fn build_view_template_node_projection_with_patches(
    document_tree_id: &str,
    layout_asset_path: &str,
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
    node_patches: &BTreeMap<String, ViewTemplateNodePatch>,
) -> Result<ViewTemplateNodeProjection, ViewTemplateProjectionError> {
    if !layout_asset_path.ends_with(".zui") {
        return Err(ViewTemplateProjectionError::NonV2AssetPath(
            layout_asset_path.to_string(),
        ));
    }
    build_view_template_node_projection_from_v2_asset(
        document_tree_id,
        layout_asset_path,
        style_imports,
        size,
        text_overrides,
        node_patches,
    )
}

#[cfg(test)]
pub(crate) fn build_view_template_nodes(
    document_tree_id: &str,
    layout_asset_path: &str,
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
) -> Result<Vec<ViewTemplateNodeData>, ViewTemplateProjectionError> {
    build_view_template_node_projection(
        document_tree_id,
        layout_asset_path,
        style_imports,
        size,
        text_overrides,
    )
    .map(ViewTemplateNodeProjection::into_vec)
}

#[cfg(test)]
pub(crate) fn build_view_template_nodes_with_imports(
    document_tree_id: &str,
    layout_asset_path: &str,
    widget_imports: &[(&str, &str)],
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
) -> Result<Vec<ViewTemplateNodeData>, ViewTemplateProjectionError> {
    let _ = widget_imports;
    build_view_template_nodes(
        document_tree_id,
        layout_asset_path,
        style_imports,
        size,
        text_overrides,
    )
}

fn build_view_template_node_projection_from_v2_asset(
    document_tree_id: &str,
    layout_asset_path: &str,
    style_imports: &[(&str, &str)],
    size: UiSize,
    text_overrides: &BTreeMap<String, String>,
    node_patches: &BTreeMap<String, ViewTemplateNodePatch>,
) -> Result<ViewTemplateNodeProjection, ViewTemplateProjectionError> {
    let outcome = {
        zircon_runtime::profile_scope!("editor", "template_projection", "load_compiled_store");
        store_cache::load_view_v2_store(layout_asset_path, style_imports)?
    };
    let design_tokens = active_editor_v2_design_tokens_snapshot();
    let projection = {
        zircon_runtime::profile_scope!("editor", "template_projection", "project_cached_nodes");
        projection_cache::projected_nodes(
            document_tree_id,
            size.width,
            size.height,
            &outcome.compiled,
            &design_tokens,
            text_overrides,
            node_patches,
            || {
                let document = prepare_editor_v2_document_with_tokens(
                    outcome.root_document.as_ref(),
                    design_tokens.as_ref(),
                );
                let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
                    UiTreeId::new(document_tree_id.to_string()),
                    &document,
                    outcome.compiled.as_ref(),
                )?;
                surface.compute_layout(size)?;
                let resource_generation = Arc::as_ptr(&outcome.compiled) as usize as u64;
                let materialization = view_template_nodes_from_surface(
                    &surface,
                    &BTreeMap::new(),
                    resource_generation,
                );
                Ok((surface, materialization))
            },
        )?
    };
    Ok(ViewTemplateNodeProjection {
        base_rows: projection.base_rows,
        row_patches: projection.row_patches,
    })
}
