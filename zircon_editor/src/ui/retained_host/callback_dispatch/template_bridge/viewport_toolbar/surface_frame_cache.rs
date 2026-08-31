use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiSize},
    surface::UiSurfaceFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use crate::ui::template_runtime::RetainedUiHostProjection;

const SURFACE_FRAME_CACHE_CAPACITY: usize = 64;

#[derive(Default)]
pub(super) struct ViewportToolbarSurfaceFrameCache {
    entries: BTreeMap<String, CachedSurfaceFrame>,
    access_generation: u64,
    #[cfg(test)]
    hit_control_projection_count: usize,
}

struct CachedSurfaceFrame {
    signature: SurfaceFrameSignature,
    hit_route_key: Option<Vec<String>>,
    frame: Arc<UiSurfaceFrame>,
    last_used_generation: u64,
}

#[derive(Clone, PartialEq)]
struct SurfaceFrameSignature {
    width_bits: u32,
    height_bits: u32,
    nodes: Vec<SurfaceFrameNode>,
}

#[derive(Clone, PartialEq)]
struct SurfaceFrameNode {
    projection_control_id: String,
    hit_control_id: Option<String>,
    component: String,
    frame: UiFrame,
}

impl ViewportToolbarSurfaceFrameCache {
    /// The owning bridge keeps one immutable template projection; only surface size can relayout it.
    pub(super) fn resolve_if_layout_matches<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_route_key: Option<&[&str]>,
        mut hit_control_id: F,
    ) -> Option<Arc<UiSurfaceFrame>>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let access_generation = self.next_access_generation();
        if self
            .entries
            .get(surface_key)
            .is_some_and(|cached| cached.signature.matches_surface_size(surface_size))
        {
            let cached = self
                .entries
                .get_mut(surface_key)
                .expect("matching viewport toolbar cache entry must remain available");
            if hit_route_keys_match(cached.hit_route_key.as_deref(), hit_route_key) {
                cached.last_used_generation = access_generation;
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.viewport_toolbar.prelayout_surface_frame_cache_hit_count",
                    1_u8
                );
                return Some(Arc::clone(&cached.frame));
            }

            let mapped_hit_control_ids =
                cached.signature.mapped_hit_control_ids(&mut hit_control_id);
            record_hit_control_projection(mapped_hit_control_ids.len());
            #[cfg(test)]
            {
                self.hit_control_projection_count = self
                    .hit_control_projection_count
                    .saturating_add(mapped_hit_control_ids.len());
            }
            if cached
                .signature
                .hit_control_ids_match(&mapped_hit_control_ids)
            {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.viewport_toolbar.prelayout_surface_frame_cache_route_key_update_count",
                    1_u8
                );
                cached.hit_route_key = own_hit_route_key(hit_route_key);
                cached.last_used_generation = access_generation;
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.viewport_toolbar.prelayout_surface_frame_cache_hit_count",
                    1_u8
                );
                return Some(Arc::clone(&cached.frame));
            }

            cached
                .signature
                .replace_hit_control_ids(mapped_hit_control_ids);
            let frame = build_surface_frame(surface_key, &cached.signature);
            cached.hit_route_key = own_hit_route_key(hit_route_key);
            cached.frame = Arc::clone(&frame);
            cached.last_used_generation = access_generation;
            zircon_runtime::profile_counter!(
                "editor",
                "ui.viewport_toolbar.prelayout_surface_frame_cache_reproject_count",
                1_u8
            );
            return Some(frame);
        }

        let shared_layout = self
            .entries
            .values()
            .find(|cached| cached.signature.matches_surface_size(surface_size))
            .map(|cached| cached.signature.clone());
        let Some(shared_layout) = shared_layout else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.viewport_toolbar.prelayout_surface_frame_cache_miss_count",
                1_u8
            );
            return None;
        };
        let mapped_hit_control_ids = shared_layout.mapped_hit_control_ids(&mut hit_control_id);
        record_hit_control_projection(mapped_hit_control_ids.len());
        #[cfg(test)]
        {
            self.hit_control_projection_count = self
                .hit_control_projection_count
                .saturating_add(mapped_hit_control_ids.len());
        }
        let mut signature = shared_layout;
        signature.replace_hit_control_ids(mapped_hit_control_ids);
        let frame = build_surface_frame(surface_key, &signature);
        self.entries.insert(
            surface_key.to_string(),
            CachedSurfaceFrame {
                signature,
                hit_route_key: own_hit_route_key(hit_route_key),
                frame: Arc::clone(&frame),
                last_used_generation: access_generation,
            },
        );
        self.evict_oldest_entry_over_capacity();
        zircon_runtime::profile_counter!(
            "editor",
            "ui.viewport_toolbar.prelayout_surface_frame_cache_shared_layout_count",
            1_u8
        );
        Some(frame)
    }

    pub(super) fn resolve<F>(
        &mut self,
        projection: &RetainedUiHostProjection,
        surface_key: &str,
        surface_size: UiSize,
        hit_route_key: Option<&[&str]>,
        hit_control_id: F,
    ) -> Arc<UiSurfaceFrame>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let signature =
            SurfaceFrameSignature::from_projection(projection, surface_size, hit_control_id);
        record_hit_control_projection(signature.nodes.len());
        #[cfg(test)]
        {
            self.hit_control_projection_count = self
                .hit_control_projection_count
                .saturating_add(signature.nodes.len());
        }
        let access_generation = self.next_access_generation();

        if let Some(cached) = self.entries.get_mut(surface_key) {
            if cached.signature == signature {
                cached.hit_route_key = own_hit_route_key(hit_route_key);
                cached.last_used_generation = access_generation;
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.viewport_toolbar.surface_frame_cache_hit_count",
                    1_u8
                );
                return Arc::clone(&cached.frame);
            }
        }

        zircon_runtime::profile_counter!(
            "editor",
            "ui.viewport_toolbar.surface_frame_cache_miss_count",
            1_u8
        );
        let frame = build_surface_frame(surface_key, &signature);
        self.entries.insert(
            surface_key.to_string(),
            CachedSurfaceFrame {
                signature,
                hit_route_key: own_hit_route_key(hit_route_key),
                frame: Arc::clone(&frame),
                last_used_generation: access_generation,
            },
        );
        self.evict_oldest_entry_over_capacity();
        frame
    }

    fn next_access_generation(&mut self) -> u64 {
        self.access_generation = self.access_generation.saturating_add(1);
        self.access_generation
    }

    fn evict_oldest_entry_over_capacity(&mut self) {
        if self.entries.len() <= SURFACE_FRAME_CACHE_CAPACITY {
            return;
        }
        let oldest_key = self
            .entries
            .iter()
            .min_by(|(left_key, left), (right_key, right)| {
                left.last_used_generation
                    .cmp(&right.last_used_generation)
                    .then_with(|| left_key.cmp(right_key))
            })
            .map(|(key, _)| key.clone());
        if let Some(oldest_key) = oldest_key {
            self.entries.remove(&oldest_key);
        }
    }

    #[cfg(test)]
    pub(super) fn hit_control_projection_count(&self) -> usize {
        self.hit_control_projection_count
    }
}

fn record_hit_control_projection(visit_count: usize) {
    zircon_runtime::profile_counter!(
        "editor",
        "ui.viewport_toolbar.hit_control_projection_batch_count",
        1_u8
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.viewport_toolbar.hit_control_projection_visit_count",
        visit_count as u64
    );
}

fn hit_route_keys_match(cached: Option<&[String]>, requested: Option<&[&str]>) -> bool {
    let (Some(cached), Some(requested)) = (cached, requested) else {
        return false;
    };
    cached.len() == requested.len()
        && cached
            .iter()
            .zip(requested)
            .all(|(cached, requested)| cached.as_str() == *requested)
}

fn own_hit_route_key(hit_route_key: Option<&[&str]>) -> Option<Vec<String>> {
    hit_route_key.map(|hit_route_key| {
        hit_route_key
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    })
}

impl SurfaceFrameSignature {
    fn from_projection<F>(
        projection: &RetainedUiHostProjection,
        surface_size: UiSize,
        mut hit_control_id: F,
    ) -> Self
    where
        F: FnMut(&str) -> Option<String>,
    {
        let nodes = projection
            .nodes
            .iter()
            .filter_map(|projection_node| {
                let projection_control_id = projection_node.control_id.as_deref()?;
                if projection_node.routes.is_empty() || projection_node.disabled {
                    return None;
                }
                Some(SurfaceFrameNode {
                    projection_control_id: projection_control_id.to_string(),
                    hit_control_id: hit_control_id(projection_control_id),
                    component: projection_node.component.clone(),
                    frame: projection_node.frame,
                })
            })
            .collect();
        Self {
            width_bits: surface_size.width.to_bits(),
            height_bits: surface_size.height.to_bits(),
            nodes,
        }
    }

    fn surface_size(&self) -> UiSize {
        UiSize::new(
            f32::from_bits(self.width_bits),
            f32::from_bits(self.height_bits),
        )
    }

    fn matches_surface_size(&self, surface_size: UiSize) -> bool {
        self.width_bits == surface_size.width.to_bits()
            && self.height_bits == surface_size.height.to_bits()
    }

    fn mapped_hit_control_ids<F>(&self, hit_control_id: &mut F) -> Vec<Option<String>>
    where
        F: FnMut(&str) -> Option<String>,
    {
        self.nodes
            .iter()
            .map(|node| hit_control_id(&node.projection_control_id))
            .collect()
    }

    fn hit_control_ids_match(&self, mapped_hit_control_ids: &[Option<String>]) -> bool {
        self.nodes
            .iter()
            .zip(mapped_hit_control_ids)
            .all(|(node, mapped)| node.hit_control_id.as_deref() == mapped.as_deref())
    }

    fn replace_hit_control_ids(&mut self, mapped_hit_control_ids: Vec<Option<String>>) {
        for (node, mapped) in self.nodes.iter_mut().zip(mapped_hit_control_ids) {
            node.hit_control_id = mapped;
        }
    }
}

fn build_surface_frame(
    surface_key: &str,
    signature: &SurfaceFrameSignature,
) -> Arc<UiSurfaceFrame> {
    let mut surface = UiSurface::new(UiTreeId::new(format!(
        "zircon.editor.viewport_toolbar.{surface_key}"
    )));
    let surface_size = signature.surface_size();
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        surface_size.width.max(1.0),
        surface_size.height.max(1.0),
    );
    let mut root = UiTreeNode::new(
        UiNodeId::new(1),
        UiNodePath::new(format!("viewport_toolbar/{surface_key}/root")),
    )
    .with_frame(root_frame)
    .with_clip_to_bounds(true)
    .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);

    for (index, (projection_node, hit_control_id)) in signature
        .nodes
        .iter()
        .filter_map(|node| {
            node.hit_control_id
                .as_ref()
                .map(|hit_control_id| (node, hit_control_id))
        })
        .enumerate()
    {
        let mut metadata = UiTemplateNodeMetadata {
            component: projection_node.component.clone(),
            control_id: Some(hit_control_id.clone()),
            ..Default::default()
        };
        metadata.attributes.insert(
            "source".to_string(),
            toml::Value::String("viewport_toolbar".to_string()),
        );
        metadata.attributes.insert(
            "projection_control_id".to_string(),
            toml::Value::String(projection_node.projection_control_id.clone()),
        );
        let node = UiTreeNode::new(
            UiNodeId::new(index as u64 + 2),
            UiNodePath::new(format!(
                "viewport_toolbar/{surface_key}/{}",
                projection_node.projection_control_id
            )),
        )
        .with_frame(projection_node.frame)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: true,
            pressed: false,
            checked: false,
            dirty: false,
        })
        .with_input_policy(UiInputPolicy::Receive)
        .with_template_metadata(metadata);
        let _ = surface.tree.insert_child(UiNodeId::new(1), node);
    }

    surface.rebuild();
    surface.surface_frame()
}
