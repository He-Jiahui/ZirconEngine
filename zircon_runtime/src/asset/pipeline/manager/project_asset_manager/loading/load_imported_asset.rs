use crate::core::CoreError;

use super::super::super::errors::asset_error_message;
use super::super::ProjectAssetManager;
use crate::asset::{AssetId, AssetKind, ImportedAsset};

impl ProjectAssetManager {
    pub fn load_imported_asset(&self, id: AssetId) -> Result<ImportedAsset, CoreError> {
        let kind = self
            .resource_manager()
            .registry()
            .get(id)
            .map(|record| record.kind)
            .ok_or_else(|| {
                asset_error_message(format!("missing resource record for asset id {id}"))
            })?;

        match kind {
            AssetKind::Data => self.load_data_asset(id).map(ImportedAsset::Data),
            AssetKind::Model => self.load_model_asset(id).map(ImportedAsset::Model),
            AssetKind::Material => self.load_material_asset(id).map(ImportedAsset::Material),
            AssetKind::MaterialGraph => self
                .load_material_graph_asset(id)
                .map(ImportedAsset::MaterialGraph),
            AssetKind::Sound => self.load_sound_asset(id).map(ImportedAsset::Sound),
            AssetKind::Font => self.load_font_asset(id).map(ImportedAsset::Font),
            AssetKind::PhysicsMaterial => self
                .load_physics_material_asset(id)
                .map(ImportedAsset::PhysicsMaterial),
            AssetKind::NavMesh => self.load_nav_mesh_asset(id).map(ImportedAsset::NavMesh),
            AssetKind::NavigationSettings => self
                .load_navigation_settings_asset(id)
                .map(ImportedAsset::NavigationSettings),
            AssetKind::Terrain => self.load_terrain_asset(id).map(ImportedAsset::Terrain),
            AssetKind::TerrainLayerStack => self
                .load_terrain_layer_stack_asset(id)
                .map(ImportedAsset::TerrainLayerStack),
            AssetKind::TileSet => self.load_tile_set_asset(id).map(ImportedAsset::TileSet),
            AssetKind::TileMap => self.load_tile_map_asset(id).map(ImportedAsset::TileMap),
            AssetKind::Prefab => self.load_prefab_asset(id).map(ImportedAsset::Prefab),
            AssetKind::Texture => self.load_texture_asset(id).map(ImportedAsset::Texture),
            AssetKind::Shader => self.load_shader_asset(id).map(ImportedAsset::Shader),
            AssetKind::Scene => self.load_scene_asset(id).map(ImportedAsset::Scene),
            AssetKind::AnimationSkeleton => self
                .load_animation_skeleton_asset(id)
                .map(ImportedAsset::AnimationSkeleton),
            AssetKind::AnimationClip => self
                .load_animation_clip_asset(id)
                .map(ImportedAsset::AnimationClip),
            AssetKind::AnimationSequence => self
                .load_animation_sequence_asset(id)
                .map(ImportedAsset::AnimationSequence),
            AssetKind::AnimationGraph => self
                .load_animation_graph_asset(id)
                .map(ImportedAsset::AnimationGraph),
            AssetKind::AnimationStateMachine => self
                .load_animation_state_machine_asset(id)
                .map(ImportedAsset::AnimationStateMachine),
            AssetKind::UiLayout => self.load_ui_layout_asset(id).map(ImportedAsset::UiLayout),
            AssetKind::UiWidget => self.load_ui_widget_asset(id).map(ImportedAsset::UiWidget),
            AssetKind::UiStyle => self.load_ui_style_asset(id).map(ImportedAsset::UiStyle),
        }
    }
}
