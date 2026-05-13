use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::v2::{UiV2AssetError, UiV2CompiledDocument};

use crate::ui::surface::UiSurface;

use super::style::{UiV2RuntimeStyleIndex, UiV2StyleResolver};
use super::surface_tree::build_tree_from_arena;

#[derive(Default)]
pub struct UiV2SurfaceBuilder;

impl UiV2SurfaceBuilder {
    pub fn build_surface(
        tree_id: UiTreeId,
        document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
    ) -> Result<UiSurface, UiV2AssetError> {
        let compiled = super::compiler::UiV2DocumentCompiler::compile(document)?;
        Self::build_surface_from_compiled_document(tree_id, document, &compiled)
    }

    pub fn build_surface_from_compiled_document(
        tree_id: UiTreeId,
        document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
        compiled: &UiV2CompiledDocument,
    ) -> Result<UiSurface, UiV2AssetError> {
        let resolved_styles = UiV2StyleResolver::resolve_static(document, &compiled.arena)?;
        let tree = build_tree_from_arena(
            &compiled.asset_id,
            tree_id.clone(),
            &compiled.arena,
            &resolved_styles,
        )?;
        let mut runtime_style = UiV2RuntimeStyleIndex::from_document(document)?;
        runtime_style.capture_baseline_from_tree(&tree);
        let mut surface = UiSurface::new(tree_id);
        surface.tree = tree;
        surface.set_runtime_style_index(runtime_style);
        surface.seed_component_states_from_tree_metadata();
        surface
            .apply_runtime_state_style_all(false)
            .map_err(|error| UiV2AssetError::InvalidDocument {
                asset_id: compiled.asset_id.clone(),
                detail: error.to_string(),
            })?;
        Ok(surface)
    }
}
