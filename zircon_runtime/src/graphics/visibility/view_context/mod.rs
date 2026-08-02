use std::collections::{BTreeMap, BTreeSet};

mod build_views;

use crate::core::framework::render::{PrimitiveRelevance, RenderLayerSet, ViewportCameraSnapshot};
use crate::core::framework::scene::EntityId;

use super::declarations::{VisibilityBounds, VisibilityBvhInstance, VisibilityRelevanceEntry};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameVisibility {
    /// Stable primitive index space for all per-view visible lists in this frame.
    pub entities: Vec<EntityId>,
    /// Stable render-instance keys aligned with `entities`.
    pub stable_instance_keys: Vec<u64>,
    pub bounds: Vec<VisibilityBounds>,
    pub render_layer_masks: Vec<RenderLayerSet>,
    pub relevance: Vec<PrimitiveRelevance>,
    pub relevance_generation: u64,
    pub views: Vec<ViewVisibilityContext>,
}

impl FrameVisibility {
    pub(crate) fn from_main_view(
        camera: &ViewportCameraSnapshot,
        bvh_instances: &[VisibilityBvhInstance],
        primitive_relevance: &[VisibilityRelevanceEntry],
        visible_stable_instance_keys: &BTreeSet<u64>,
    ) -> Self {
        let relevance_by_stable_instance_key = primitive_relevance
            .iter()
            .map(|entry| (entry.stable_instance_key, entry.relevance))
            .collect::<BTreeMap<_, _>>();
        let entities = bvh_instances
            .iter()
            .map(|instance| instance.entity)
            .collect::<Vec<_>>();
        let stable_instance_keys = bvh_instances
            .iter()
            .map(|instance| instance.stable_instance_key)
            .collect::<Vec<_>>();
        let bounds = bvh_instances
            .iter()
            .map(|instance| instance.bounds)
            .collect::<Vec<_>>();
        let render_layer_masks = bvh_instances
            .iter()
            .map(|instance| instance.key.render_layer_mask.clone())
            .collect::<Vec<_>>();
        let relevance = bvh_instances
            .iter()
            .map(|instance| {
                relevance_by_stable_instance_key
                    .get(&instance.stable_instance_key)
                    .copied()
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();
        let main_view = ViewVisibilityContext::main_camera(
            camera.clone(),
            &stable_instance_keys,
            &relevance,
            visible_stable_instance_keys,
        );

        Self {
            entities,
            stable_instance_keys,
            bounds,
            render_layer_masks,
            relevance,
            relevance_generation: 0,
            views: vec![main_view],
        }
    }

    pub fn main_view(&self) -> Option<&ViewVisibilityContext> {
        self.view(&VisibilityViewKey::MainCamera)
    }

    pub fn view(&self, key: &VisibilityViewKey) -> Option<&ViewVisibilityContext> {
        self.views.iter().find(|view| &view.view == key)
    }

    pub fn shadow_views(&self) -> impl Iterator<Item = &ViewVisibilityContext> {
        self.views.iter().filter(|view| {
            matches!(
                view.view,
                VisibilityViewKey::ShadowCascade { .. }
                    | VisibilityViewKey::ShadowPointFace { .. }
                    | VisibilityViewKey::ShadowSpot { .. }
            )
        })
    }

    pub fn visible_entities_for_view(&self, key: &VisibilityViewKey) -> Vec<EntityId> {
        self.view(key)
            .map(|view| self.visible_entities_from_indices(&view.visible))
            .unwrap_or_default()
    }

    pub fn visible_entity_set_for_view(&self, key: &VisibilityViewKey) -> BTreeSet<EntityId> {
        self.visible_entities_for_view(key).into_iter().collect()
    }

    pub fn main_view_visible_entities(&self) -> Vec<EntityId> {
        self.visible_entities_for_view(&VisibilityViewKey::MainCamera)
    }

    pub fn main_view_visible_entity_set(&self) -> BTreeSet<EntityId> {
        self.visible_entity_set_for_view(&VisibilityViewKey::MainCamera)
    }

    pub(crate) fn main_view_visible_stable_instance_key_set(&self) -> BTreeSet<u64> {
        self.visible_stable_instance_key_set_for_view(&VisibilityViewKey::MainCamera)
    }

    pub fn shadow_visible_entity_set(&self) -> BTreeSet<EntityId> {
        self.shadow_views()
            .flat_map(|view| self.visible_entities_from_indices(&view.visible))
            .collect()
    }

    fn visible_entities_from_indices(&self, visible_indices: &[u32]) -> Vec<EntityId> {
        visible_indices
            .iter()
            .filter_map(|index| self.entities.get(*index as usize).copied())
            .collect()
    }

    fn visible_stable_instance_key_set_for_view(&self, key: &VisibilityViewKey) -> BTreeSet<u64> {
        self.view(key)
            .into_iter()
            .flat_map(|view| view.visible.iter())
            .filter_map(|index| self.stable_instance_keys.get(*index as usize).copied())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewVisibilityContext {
    pub view: VisibilityViewKey,
    pub camera: ViewportCameraSnapshot,
    /// Indices into the aligned `FrameVisibility` owner/key arrays, kept in frame order.
    pub visible: Vec<u32>,
    pub stats: ViewCullingStats,
}

impl ViewVisibilityContext {
    fn main_camera(
        camera: ViewportCameraSnapshot,
        stable_instance_keys: &[u64],
        relevance: &[PrimitiveRelevance],
        visible_stable_instance_keys: &BTreeSet<u64>,
    ) -> Self {
        let visible = stable_instance_keys
            .iter()
            .enumerate()
            .filter_map(|(index, stable_instance_key)| {
                visible_stable_instance_keys
                    .contains(stable_instance_key)
                    .then(|| {
                        u32::try_from(index)
                            .expect("frame visibility primitive index exceeds u32 range")
                    })
            })
            .collect::<Vec<_>>();
        let layer_filtered_count = relevance
            .iter()
            .filter(|relevance| !relevance.main_view())
            .count();
        let visible_count = visible.len();
        let frustum_culled_count = stable_instance_keys
            .len()
            .saturating_sub(layer_filtered_count)
            .saturating_sub(visible_count);

        Self {
            view: VisibilityViewKey::MainCamera,
            camera,
            visible,
            stats: ViewCullingStats {
                input_count: stable_instance_keys.len(),
                layer_filtered_count,
                frustum_culled_count,
                occlusion_culled_count: 0,
                visible_count,
            },
        }
    }
}

impl Default for ViewVisibilityContext {
    fn default() -> Self {
        Self {
            view: VisibilityViewKey::MainCamera,
            camera: ViewportCameraSnapshot::default(),
            visible: Vec::new(),
            stats: ViewCullingStats::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum VisibilityViewKey {
    #[default]
    MainCamera,
    ShadowCascade {
        light: EntityId,
        cascade: u8,
    },
    ShadowPointFace {
        light: EntityId,
        face: u8,
    },
    ShadowSpot {
        light: EntityId,
    },
    CustomTarget {
        camera: EntityId,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewCullingStats {
    pub input_count: usize,
    pub layer_filtered_count: usize,
    pub frustum_culled_count: usize,
    pub occlusion_culled_count: usize,
    pub visible_count: usize,
}
