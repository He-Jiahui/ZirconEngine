use serde::{Deserialize, Serialize};

use crate::asset::assets::{
    MaterialAssetManagementRecordSet, MeshAssetManagementRecordSet, ModelAssetManagementRecordSet,
    SceneAssetManagementRecordSet, SceneEntityManagementRecordSet, ShaderAssetManagementRecordSet,
};
use crate::core::framework::render::RenderMaterialManagementRecordSet;

/// Header totals for the combined asset management read model.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementRecordSetSummary {
    pub managed_record_count: usize,
    pub degraded_record_count: usize,
    pub model_count: usize,
    pub model_mesh_referenced_model_count: usize,
    pub model_mesh_reference_count: usize,
    pub mesh_count: usize,
    pub valid_mesh_count: usize,
    pub invalid_mesh_count: usize,
    pub mesh_morph_target_count: usize,
    pub mesh_morph_target_attribute_count: usize,
    pub scene_count: usize,
    pub scene_entity_count: usize,
    pub entity_count: usize,
    pub active_entity_count: usize,
    pub root_entity_count: usize,
    pub entity_direct_reference_count: usize,
    pub entity_camera_count: usize,
    pub entity_mesh_instance_count: usize,
    pub entity_direct_mesh_reference_count: usize,
    pub entity_mesh_primitive_binding_count: usize,
    pub entity_morph_weight_count: usize,
    pub entity_mesh_material_binding_count: usize,
    pub entity_collider_material_binding_count: usize,
    pub entity_light_count: usize,
    pub entity_physics_component_count: usize,
    pub entity_animation_binding_count: usize,
    pub entity_terrain_count: usize,
    pub entity_tilemap_count: usize,
    pub entity_prefab_instance_count: usize,
    pub material_count: usize,
    pub material_ready_count: usize,
    pub material_degraded_count: usize,
    pub material_issue_row_count: usize,
    pub material_property_override_count: usize,
    pub material_texture_slot_count: usize,
    pub material_texture_reference_count: usize,
    pub material_fallback_texture_slot_count: usize,
    pub material_validation_error_count: usize,
    pub material_validation_diagnostic_count: usize,
    pub material_direct_reference_count: usize,
    pub prepared_material_count: usize,
    pub prepared_material_ready_count: usize,
    pub prepared_material_degraded_count: usize,
    pub prepared_material_issue_row_count: usize,
    pub shader_count: usize,
    pub shader_ready_count: usize,
    pub shader_not_ready_count: usize,
    pub shader_issue_row_count: usize,
    pub shader_validation_diagnostic_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssetManagementFamilyKind {
    Model,
    Mesh,
    Scene,
    Entity,
    Material,
    Shader,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetManagementFamilyStatus {
    Empty,
    Ready,
    Degraded,
}

/// Compact row for the top-level asset-family management overview.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilySummary {
    pub kind: AssetManagementFamilyKind,
    pub status: AssetManagementFamilyStatus,
    pub total_record_count: usize,
    pub ready_record_count: usize,
    pub degraded_record_count: usize,
    pub issue_row_count: usize,
}

/// Status buckets for the fixed top-level asset families.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilyStatusIndex {
    pub empty: Vec<AssetManagementFamilyKind>,
    pub ready: Vec<AssetManagementFamilyKind>,
    pub degraded: Vec<AssetManagementFamilyKind>,
}

/// Row-bearing companion for one top-level family status bucket.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilyStatusView {
    pub status: AssetManagementFamilyStatus,
    pub families: Vec<AssetManagementFamilyKind>,
    pub rows: Vec<AssetManagementFamilySummary>,
    pub total_record_count: usize,
    pub ready_record_count: usize,
    pub degraded_record_count: usize,
    pub issue_row_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetManagementFamilyIssueBucket {
    Clean,
    WithIssues,
}

/// Issue-row buckets for top-level asset-family drilldown navigation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilyIssueIndex {
    pub clean: Vec<AssetManagementFamilyKind>,
    pub with_issues: Vec<AssetManagementFamilyKind>,
}

/// Row-bearing companion for one top-level family issue bucket.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementFamilyIssueView {
    pub bucket: AssetManagementFamilyIssueBucket,
    pub families: Vec<AssetManagementFamilyKind>,
    pub rows: Vec<AssetManagementFamilySummary>,
    pub issue_row_count: usize,
}

/// Lightweight top-level state for management headers and navigation chrome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManagementOverview {
    pub summary: AssetManagementRecordSetSummary,
    pub families: Vec<AssetManagementFamilySummary>,
    #[serde(default)]
    pub family_status_index: AssetManagementFamilyStatusIndex,
    #[serde(default)]
    pub family_issue_index: AssetManagementFamilyIssueIndex,
}

/// One payload for renderer/editor panels that need all asset-management lists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetManagementRecordSets {
    pub summary: AssetManagementRecordSetSummary,
    pub families: Vec<AssetManagementFamilySummary>,
    #[serde(default)]
    pub family_status_index: AssetManagementFamilyStatusIndex,
    #[serde(default)]
    pub family_issue_index: AssetManagementFamilyIssueIndex,
    pub models: ModelAssetManagementRecordSet,
    pub meshes: MeshAssetManagementRecordSet,
    pub scenes: SceneAssetManagementRecordSet,
    pub scene_entities: SceneEntityManagementRecordSet,
    pub material_assets: MaterialAssetManagementRecordSet,
    /// Renderer-prepared material detail records kept beside asset-level material rows.
    pub materials: RenderMaterialManagementRecordSet,
    pub shaders: ShaderAssetManagementRecordSet,
}

impl AssetManagementFamilySummary {
    pub fn new(
        kind: AssetManagementFamilyKind,
        total_record_count: usize,
        ready_record_count: usize,
        degraded_record_count: usize,
        issue_row_count: usize,
    ) -> Self {
        let status = if total_record_count == 0 {
            AssetManagementFamilyStatus::Empty
        } else if degraded_record_count > 0 {
            AssetManagementFamilyStatus::Degraded
        } else {
            AssetManagementFamilyStatus::Ready
        };
        Self {
            kind,
            status,
            total_record_count,
            ready_record_count,
            degraded_record_count,
            issue_row_count,
        }
    }
}

impl AssetManagementOverview {
    pub fn from_summary(summary: AssetManagementRecordSetSummary) -> Self {
        let families = summary.family_summaries();
        let family_status_index = AssetManagementFamilyStatusIndex::from_families(&families);
        let family_issue_index = AssetManagementFamilyIssueIndex::from_families(&families);
        Self {
            summary,
            families,
            family_status_index,
            family_issue_index,
        }
    }

    pub fn family_summaries(&self) -> &[AssetManagementFamilySummary] {
        &self.families
    }

    pub fn family_status_index(&self) -> &AssetManagementFamilyStatusIndex {
        &self.family_status_index
    }

    pub fn family_status_view(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> AssetManagementFamilyStatusView {
        AssetManagementFamilyStatusView::from_families(&self.families, status)
    }

    pub fn family_issue_index(&self) -> &AssetManagementFamilyIssueIndex {
        &self.family_issue_index
    }

    pub fn family_issue_view(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> AssetManagementFamilyIssueView {
        AssetManagementFamilyIssueView::from_families(&self.families, bucket)
    }
}

impl AssetManagementFamilyStatus {
    fn matches(self, family: &AssetManagementFamilySummary) -> bool {
        family.status == self
    }
}

impl AssetManagementFamilyStatusIndex {
    pub fn from_families(families: &[AssetManagementFamilySummary]) -> Self {
        let mut index = Self::default();
        for family in families {
            match family.status {
                AssetManagementFamilyStatus::Empty => index.empty.push(family.kind),
                AssetManagementFamilyStatus::Ready => index.ready.push(family.kind),
                AssetManagementFamilyStatus::Degraded => index.degraded.push(family.kind),
            }
        }
        index
    }

    pub fn total_family_count(&self) -> usize {
        self.empty.len() + self.ready.len() + self.degraded.len()
    }

    pub fn degraded_family_count(&self) -> usize {
        self.degraded.len()
    }

    pub fn has_degraded_families(&self) -> bool {
        !self.degraded.is_empty()
    }

    pub fn families_for_status(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> &[AssetManagementFamilyKind] {
        match status {
            AssetManagementFamilyStatus::Empty => &self.empty,
            AssetManagementFamilyStatus::Ready => &self.ready,
            AssetManagementFamilyStatus::Degraded => &self.degraded,
        }
    }
}

impl AssetManagementFamilyStatusView {
    pub fn from_families(
        families: &[AssetManagementFamilySummary],
        status: AssetManagementFamilyStatus,
    ) -> Self {
        let rows = families
            .iter()
            .filter(|family| status.matches(family))
            .cloned()
            .collect::<Vec<_>>();
        let total_record_count = rows.iter().map(|family| family.total_record_count).sum();
        let ready_record_count = rows.iter().map(|family| family.ready_record_count).sum();
        let degraded_record_count = rows.iter().map(|family| family.degraded_record_count).sum();
        let issue_row_count = rows.iter().map(|family| family.issue_row_count).sum();
        let families = rows.iter().map(|family| family.kind).collect();
        Self {
            status,
            families,
            rows,
            total_record_count,
            ready_record_count,
            degraded_record_count,
            issue_row_count,
        }
    }
}

impl AssetManagementFamilyIssueBucket {
    fn matches(self, family: &AssetManagementFamilySummary) -> bool {
        match self {
            Self::Clean => family.issue_row_count == 0,
            Self::WithIssues => family.issue_row_count > 0,
        }
    }
}

impl AssetManagementFamilyIssueIndex {
    pub fn from_families(families: &[AssetManagementFamilySummary]) -> Self {
        let mut index = Self::default();
        for family in families {
            if family.issue_row_count > 0 {
                index.with_issues.push(family.kind);
            } else {
                index.clean.push(family.kind);
            }
        }
        index
    }

    pub fn total_family_count(&self) -> usize {
        self.clean.len() + self.with_issues.len()
    }

    pub fn issue_family_count(&self) -> usize {
        self.with_issues.len()
    }

    pub fn has_issue_families(&self) -> bool {
        !self.with_issues.is_empty()
    }

    pub fn families_with_issues(&self) -> &[AssetManagementFamilyKind] {
        &self.with_issues
    }

    pub fn families_without_issues(&self) -> &[AssetManagementFamilyKind] {
        &self.clean
    }

    pub fn families_for_bucket(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> &[AssetManagementFamilyKind] {
        match bucket {
            AssetManagementFamilyIssueBucket::Clean => &self.clean,
            AssetManagementFamilyIssueBucket::WithIssues => &self.with_issues,
        }
    }
}

impl AssetManagementFamilyIssueView {
    pub fn from_families(
        families: &[AssetManagementFamilySummary],
        bucket: AssetManagementFamilyIssueBucket,
    ) -> Self {
        let rows = families
            .iter()
            .filter(|family| bucket.matches(family))
            .cloned()
            .collect::<Vec<_>>();
        let issue_row_count = rows.iter().map(|family| family.issue_row_count).sum();
        let families = rows.iter().map(|family| family.kind).collect();
        Self {
            bucket,
            families,
            rows,
            issue_row_count,
        }
    }
}

impl AssetManagementRecordSetSummary {
    pub fn from_record_sets(
        models: &ModelAssetManagementRecordSet,
        meshes: &MeshAssetManagementRecordSet,
        scenes: &SceneAssetManagementRecordSet,
        scene_entities: &SceneEntityManagementRecordSet,
        material_assets: &MaterialAssetManagementRecordSet,
        materials: &RenderMaterialManagementRecordSet,
        shaders: &ShaderAssetManagementRecordSet,
    ) -> Self {
        let material_degraded_count = material_assets.summary.degraded_count();
        let prepared_material_degraded_count = materials.summary.degraded_count();
        Self {
            managed_record_count: models.summary.model_count
                + meshes.summary.mesh_count
                + scenes.summary.scene_count
                + scene_entities.summary.entity_count
                + material_assets.summary.material_count
                + shaders.summary.shader_count,
            degraded_record_count: meshes.summary.invalid_mesh_count
                + material_degraded_count
                + shaders.summary.not_ready_count,
            model_count: models.summary.model_count,
            model_mesh_referenced_model_count: models.summary.mesh_referenced_model_count,
            model_mesh_reference_count: models.summary.mesh_reference_count,
            mesh_count: meshes.summary.mesh_count,
            valid_mesh_count: meshes.summary.valid_mesh_count,
            invalid_mesh_count: meshes.summary.invalid_mesh_count,
            mesh_morph_target_count: meshes.summary.morph_target_count,
            mesh_morph_target_attribute_count: meshes.summary.morph_target_attribute_count,
            scene_count: scenes.summary.scene_count,
            scene_entity_count: scenes.summary.entity_count,
            entity_count: scene_entities.summary.entity_count,
            active_entity_count: scene_entities.summary.active_entity_count,
            root_entity_count: scene_entities.summary.root_entity_count,
            entity_direct_reference_count: scene_entities.summary.direct_reference_count,
            entity_camera_count: scene_entities.summary.camera_count,
            entity_mesh_instance_count: scene_entities.summary.mesh_instance_count,
            entity_direct_mesh_reference_count: scene_entities.summary.direct_mesh_reference_count,
            entity_mesh_primitive_binding_count: scene_entities
                .summary
                .mesh_primitive_binding_count,
            entity_morph_weight_count: scene_entities.summary.morph_weight_count,
            entity_mesh_material_binding_count: scene_entities.summary.mesh_material_binding_count,
            entity_collider_material_binding_count: scene_entities
                .summary
                .collider_material_binding_count,
            entity_light_count: scene_entities.summary.light_count,
            entity_physics_component_count: scene_entities.summary.physics_component_count,
            entity_animation_binding_count: scene_entities.summary.animation_binding_count,
            entity_terrain_count: scene_entities.summary.terrain_count,
            entity_tilemap_count: scene_entities.summary.tilemap_count,
            entity_prefab_instance_count: scene_entities.summary.prefab_instance_count,
            material_count: material_assets.summary.material_count,
            material_ready_count: material_assets.summary.ready_count,
            material_degraded_count,
            material_issue_row_count: material_assets.summary.issue_row_count(),
            material_property_override_count: material_assets.summary.property_override_count,
            material_texture_slot_count: material_assets.summary.texture_slot_count,
            material_texture_reference_count: material_assets.summary.texture_reference_count,
            material_fallback_texture_slot_count: material_assets
                .summary
                .fallback_texture_slot_count,
            material_validation_error_count: material_assets.summary.validation_error_count,
            material_validation_diagnostic_count: material_assets
                .summary
                .validation_diagnostic_count,
            material_direct_reference_count: material_assets.summary.direct_reference_count,
            prepared_material_count: materials.summary.total_count,
            prepared_material_ready_count: materials.summary.ready_count,
            prepared_material_degraded_count,
            prepared_material_issue_row_count: materials.summary.issue_row_count(),
            shader_count: shaders.summary.shader_count,
            shader_ready_count: shaders.summary.ready_count,
            shader_not_ready_count: shaders.summary.not_ready_count,
            shader_issue_row_count: shaders.summary.issue_row_count(),
            shader_validation_diagnostic_count: shaders.summary.validation_diagnostic_count,
        }
    }

    pub fn family_summaries(&self) -> Vec<AssetManagementFamilySummary> {
        vec![
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Model,
                self.model_count,
                self.model_count,
                0,
                0,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Mesh,
                self.mesh_count,
                self.valid_mesh_count,
                self.invalid_mesh_count,
                self.invalid_mesh_count,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Scene,
                self.scene_count,
                self.scene_count,
                0,
                0,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Entity,
                self.entity_count,
                self.entity_count,
                0,
                0,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Material,
                self.material_count,
                self.material_ready_count,
                self.material_degraded_count,
                self.material_issue_row_count,
            ),
            AssetManagementFamilySummary::new(
                AssetManagementFamilyKind::Shader,
                self.shader_count,
                self.shader_ready_count,
                self.shader_not_ready_count,
                self.shader_issue_row_count,
            ),
        ]
    }
}

impl AssetManagementRecordSets {
    pub fn from_record_sets(
        models: ModelAssetManagementRecordSet,
        meshes: MeshAssetManagementRecordSet,
        scenes: SceneAssetManagementRecordSet,
        scene_entities: SceneEntityManagementRecordSet,
        material_assets: MaterialAssetManagementRecordSet,
        materials: RenderMaterialManagementRecordSet,
        shaders: ShaderAssetManagementRecordSet,
    ) -> Self {
        let summary = AssetManagementRecordSetSummary::from_record_sets(
            &models,
            &meshes,
            &scenes,
            &scene_entities,
            &material_assets,
            &materials,
            &shaders,
        );
        let families = summary.family_summaries();
        let family_status_index = AssetManagementFamilyStatusIndex::from_families(&families);
        let family_issue_index = AssetManagementFamilyIssueIndex::from_families(&families);
        Self {
            summary,
            families,
            family_status_index,
            family_issue_index,
            models,
            meshes,
            scenes,
            scene_entities,
            material_assets,
            materials,
            shaders,
        }
    }

    pub fn family_summaries(&self) -> &[AssetManagementFamilySummary] {
        &self.families
    }

    pub fn family_status_index(&self) -> &AssetManagementFamilyStatusIndex {
        &self.family_status_index
    }

    pub fn family_status_view(
        &self,
        status: AssetManagementFamilyStatus,
    ) -> AssetManagementFamilyStatusView {
        AssetManagementFamilyStatusView::from_families(&self.families, status)
    }

    pub fn family_issue_index(&self) -> &AssetManagementFamilyIssueIndex {
        &self.family_issue_index
    }

    pub fn family_issue_view(
        &self,
        bucket: AssetManagementFamilyIssueBucket,
    ) -> AssetManagementFamilyIssueView {
        AssetManagementFamilyIssueView::from_families(&self.families, bucket)
    }

    pub fn overview(&self) -> AssetManagementOverview {
        AssetManagementOverview {
            summary: self.summary.clone(),
            families: self.families.clone(),
            family_status_index: self.family_status_index.clone(),
            family_issue_index: self.family_issue_index.clone(),
        }
    }
}
