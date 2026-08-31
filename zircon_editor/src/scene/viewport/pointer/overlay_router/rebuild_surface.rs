use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::ui::surface::{UiAuthoredGeometryPublication, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiFrame, UiSize},
    tree::{UiInputPolicy, UiTreeNode},
};

use crate::scene::viewport::pointer::{
    candidates::{
        candidate_z_index, interactive_state_flags, passive_state_flags,
        precision_candidates_from_layout,
    },
    constants::{FIRST_CANDIDATE_NODE_ID, ROOT_NODE_ID, VIEWPORT_NODE_ID},
    precision::{lock_shared_resolution_state, PrecisionCandidate},
};

use super::ViewportOverlayPointerRouter;

struct ViewportOverlaySurfaceClassification {
    delta: ViewportOverlaySurfaceDelta,
    candidate_map_reusable: bool,
}

enum ViewportOverlaySurfaceDelta {
    NoChange {
        route_identity_changed: bool,
    },
    Geometry {
        changes: Vec<ViewportOverlayNodeGeometryChange>,
        ordering_changed: bool,
        route_identity_changed: bool,
    },
    Topology,
}

#[derive(Clone, Copy)]
struct ViewportOverlayNodeGeometryChange {
    node_id: UiNodeId,
    frame: UiFrame,
    z_index: Option<i32>,
}

impl ViewportOverlayPointerRouter {
    pub(in crate::scene::viewport::pointer) fn rebuild_surface(&mut self) {
        let viewport_frame = UiFrame::new(
            0.0,
            0.0,
            self.layout.viewport.x.max(1) as f32,
            self.layout.viewport.y.max(1) as f32,
        );
        let candidates = retained_candidate_entries(&self.layout);
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.surface_delta_projected_candidate_count",
            candidates.len(),
        );
        let classification = self.classify_surface_delta(viewport_frame, &candidates);
        let candidate_map_reusable = classification.candidate_map_reusable;
        let surface_reused =
            !matches!(&classification.delta, ViewportOverlaySurfaceDelta::Topology);
        self.apply_surface_delta(viewport_frame, &candidates, classification.delta);
        if surface_reused {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.surface_authority_reuse_count",
                1_u8,
            );
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.surface_authority_rebuild_count",
                1_u8,
            );
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.surface_authority_rebuild_candidate_count",
                candidates.len(),
            );
        }

        let mut shared = lock_shared_resolution_state(self.shared.as_ref());
        let candidate_map_reused =
            publish_retained_candidates(&mut shared, candidates, candidate_map_reusable);
        if candidate_map_reused {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.candidate_authority_map_reuse_count",
                1_u8,
            );
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.candidate_authority_map_rebuild_count",
                1_u8,
            );
        }
        shared.last_route = None;
        shared.last_debug_feed = None;
    }

    fn classify_surface_delta(
        &self,
        viewport_frame: UiFrame,
        candidates: &[RetainedCandidateEntry],
    ) -> ViewportOverlaySurfaceClassification {
        let mut topology_matches = self.retained_candidate_ids.len() == candidates.len();
        let mut changes = Vec::new();
        topology_matches &= classify_node_geometry(
            &self.surface,
            ROOT_NODE_ID,
            viewport_frame,
            None,
            &mut changes,
        );
        topology_matches &= classify_node_geometry(
            &self.surface,
            VIEWPORT_NODE_ID,
            viewport_frame,
            None,
            &mut changes,
        );

        let shared = lock_shared_resolution_state(self.shared.as_ref());
        let mut candidate_map_reusable = shared.candidates.len() == candidates.len();
        let mut route_identity_changed = false;
        let mut ordering_changed = false;
        for (candidate_index, candidate) in candidates.iter().enumerate() {
            let identity_matches =
                self.retained_candidate_ids.get(candidate_index) == Some(&candidate.node_id);
            topology_matches &= identity_matches;
            if identity_matches {
                let change_start = changes.len();
                topology_matches &= classify_node_geometry(
                    &self.surface,
                    candidate.node_id,
                    candidate.frame,
                    Some(candidate.z_index),
                    &mut changes,
                );
                ordering_changed |= changes
                    .get(change_start)
                    .is_some_and(|change| change.z_index.is_some());
            }
            match shared.candidates.get(&candidate.node_id) {
                Some(current) => {
                    route_identity_changed |= current.route != candidate.candidate.route;
                }
                None => {
                    route_identity_changed = true;
                    candidate_map_reusable = false;
                }
            }
        }
        candidate_map_reusable &= topology_matches;
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.surface_delta_classified_candidate_count",
            candidates.len(),
        );
        let delta = if !topology_matches {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.surface_delta_topology_count",
                1_u8,
            );
            ViewportOverlaySurfaceDelta::Topology
        } else if changes.is_empty() {
            ViewportOverlaySurfaceDelta::NoChange {
                route_identity_changed,
            }
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.surface_delta_changed_node_count",
                changes.len(),
            );
            ViewportOverlaySurfaceDelta::Geometry {
                changes,
                ordering_changed,
                route_identity_changed,
            }
        };
        ViewportOverlaySurfaceClassification {
            delta,
            candidate_map_reusable,
        }
    }

    fn apply_surface_delta(
        &mut self,
        viewport_frame: UiFrame,
        candidates: &[RetainedCandidateEntry],
        delta: ViewportOverlaySurfaceDelta,
    ) {
        match delta {
            ViewportOverlaySurfaceDelta::NoChange {
                route_identity_changed,
            } => {
                release_capture_for_route_change(&mut self.surface, route_identity_changed);
            }
            ViewportOverlaySurfaceDelta::Geometry {
                changes,
                ordering_changed,
                route_identity_changed,
            } => {
                release_capture_for_route_change(&mut self.surface, route_identity_changed);
                self.apply_geometry_delta(viewport_frame, changes, ordering_changed);
            }
            ViewportOverlaySurfaceDelta::Topology => {
                self.rebuild_surface_from_scratch(viewport_frame, candidates);
            }
        }
    }

    fn apply_geometry_delta(
        &mut self,
        viewport_frame: UiFrame,
        changes: Vec<ViewportOverlayNodeGeometryChange>,
        ordering_changed: bool,
    ) {
        let observed_topology_generation = self.surface.tree.layout_order_generation();
        let applied_node_count = changes.len();
        let mut changed_node_ids = BTreeSet::new();
        for change in changes {
            patch_retained_node_frame(
                &mut self.surface,
                change.node_id,
                change.frame,
                &mut changed_node_ids,
            );
            if let Some(z_index) = change.z_index {
                self.surface
                    .tree
                    .node_mut(change.node_id)
                    .expect("overlay geometry receipt must reference retained topology")
                    .z_index = z_index;
            }
        }
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.surface_delta_applied_node_count",
            applied_node_count,
        );
        if ordering_changed {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.surface_ordering_fallback_count",
                1_u8,
            );
            self.surface
                .rebuild_authored_frames(UiSize::new(viewport_frame.width, viewport_frame.height));
        } else if !changed_node_ids.is_empty() {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.pointer.surface_geometry_patch_node_count",
                changed_node_ids.len(),
            );
            match self.surface.publish_authored_geometry(
                UiSize::new(viewport_frame.width, viewport_frame.height),
                &changed_node_ids,
                observed_topology_generation,
            ) {
                UiAuthoredGeometryPublication::Local(_) => zircon_runtime::profile_counter!(
                    "editor",
                    "viewport.pointer.surface_geometry_local_publication_count",
                    1_u8,
                ),
                UiAuthoredGeometryPublication::FullFallback { .. } => {
                    zircon_runtime::profile_counter!(
                        "editor",
                        "viewport.pointer.surface_geometry_fallback_count",
                        1_u8,
                    );
                }
                UiAuthoredGeometryPublication::Unchanged => {}
            }
        }
    }

    fn rebuild_surface_from_scratch(
        &mut self,
        viewport_frame: UiFrame,
        candidates: &[RetainedCandidateEntry],
    ) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.viewport.pointer"));
        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                UiNodePath::new("editor.viewport.pointer.root"),
            )
            .with_frame(viewport_frame)
            .with_state_flags(passive_state_flags())
            .with_input_policy(UiInputPolicy::Receive),
        );
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(
                    VIEWPORT_NODE_ID,
                    UiNodePath::new("editor.viewport.pointer.viewport"),
                )
                .with_frame(viewport_frame)
                .with_state_flags(interactive_state_flags())
                .with_input_policy(UiInputPolicy::Receive),
            )
            .expect("viewport pointer root must exist");

        for candidate in candidates {
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        candidate.node_id,
                        UiNodePath::new(format!(
                            "editor.viewport.pointer/candidate_{}",
                            candidate.path_suffix,
                        )),
                    )
                    .with_frame(candidate.frame)
                    .with_state_flags(interactive_state_flags())
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_z_index(candidate.z_index),
                )
                .expect("viewport pointer viewport must exist");
        }

        surface.rebuild_authored_frames(UiSize::new(viewport_frame.width, viewport_frame.height));
        self.surface = surface;
        self.retained_candidate_ids = candidates
            .iter()
            .map(|candidate| candidate.node_id)
            .collect::<Vec<_>>()
            .into();
    }
}

struct RetainedCandidateEntry {
    node_id: UiNodeId,
    path_suffix: u64,
    frame: UiFrame,
    z_index: i32,
    candidate: PrecisionCandidate,
}

fn retained_candidate_entries(
    layout: &crate::scene::viewport::pointer::viewport_pointer_layout::ViewportPointerLayout,
) -> Vec<RetainedCandidateEntry> {
    let projected_candidates = precision_candidates_from_layout(layout);
    let mut candidates = Vec::with_capacity(projected_candidates.len());
    for (candidate_index, candidate) in projected_candidates.into_iter().enumerate() {
        let node_id_value = FIRST_CANDIDATE_NODE_ID.saturating_add(candidate_index as u64);
        let Some(frame) = candidate.shape.hit_frame() else {
            continue;
        };
        let node_id = UiNodeId::new(node_id_value);
        candidates.push(RetainedCandidateEntry {
            node_id,
            path_suffix: node_id_value.saturating_add(1),
            frame,
            z_index: candidate_z_index(candidate.priority),
            candidate,
        });
    }
    candidates
}

fn patch_retained_node_frame(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    changed_node_ids: &mut BTreeSet<UiNodeId>,
) {
    if surface.tree.node(node_id).is_some_and(|node| {
        node.layout_cache.frame == frame && node.layout_cache.clip_frame.is_none()
    }) {
        return;
    }
    let node = surface
        .tree
        .node_mut(node_id)
        .expect("overlay geometry receipt must reference retained topology");
    node.layout_cache.frame = frame;
    node.layout_cache.clip_frame = None;
    changed_node_ids.insert(node_id);
}

fn classify_node_geometry(
    surface: &UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    z_index: Option<i32>,
    changes: &mut Vec<ViewportOverlayNodeGeometryChange>,
) -> bool {
    let Some(node) = surface.tree.node(node_id) else {
        return false;
    };
    let frame_changed = node.layout_cache.frame != frame || node.layout_cache.clip_frame.is_some();
    let changed_z_index = z_index.filter(|z_index| node.z_index != *z_index);
    if frame_changed || changed_z_index.is_some() {
        changes.push(ViewportOverlayNodeGeometryChange {
            node_id,
            frame,
            z_index: changed_z_index,
        });
    }
    true
}

fn release_capture_for_route_change(surface: &mut UiSurface, route_identity_changed: bool) {
    if route_identity_changed {
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.surface_delta_route_change_count",
            1_u8,
        );
        surface.release_pointer_capture();
    }
}

fn publish_retained_candidates(
    shared: &mut crate::scene::viewport::pointer::precision::SharedResolutionState,
    candidates: Vec<RetainedCandidateEntry>,
    candidate_map_reusable: bool,
) -> bool {
    if !candidate_map_reusable {
        shared.candidates = candidates
            .into_iter()
            .map(|candidate| (candidate.node_id, candidate.candidate))
            .collect::<BTreeMap<_, _>>();
        return false;
    }

    for candidate in candidates {
        *shared
            .candidates
            .get_mut(&candidate.node_id)
            .expect("surface classification must prove retained candidate map identity") =
            candidate.candidate;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::viewport::pointer::{
        precision::PrecisionShape, viewport_pointer_route::ViewportPointerRoute,
    };
    use zircon_runtime_interface::math::Vec2;

    #[test]
    fn stable_candidate_topology_patches_geometry_without_replacing_nodes() {
        let mut router = ViewportOverlayPointerRouter::new();
        let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
        let initial = candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(10.0, 20.0, 30.0, 40.0),
            100,
        );
        router.rebuild_surface_from_scratch(viewport_frame, &[initial]);
        let root_address = router
            .surface
            .tree
            .node(ROOT_NODE_ID)
            .expect("root must exist") as *const UiTreeNode;
        let candidate_address = router
            .surface
            .tree
            .node(UiNodeId::new(FIRST_CANDIDATE_NODE_ID))
            .expect("candidate must exist") as *const UiTreeNode;

        let next_frame = UiFrame::new(50.0, 60.0, 70.0, 80.0);
        let next = candidate_entry(FIRST_CANDIDATE_NODE_ID, next_frame, 300);
        assert!(try_apply_surface_delta(
            &mut router,
            viewport_frame,
            &[next]
        ));

        assert_eq!(
            router
                .surface
                .tree
                .node(ROOT_NODE_ID)
                .expect("root must remain") as *const UiTreeNode,
            root_address,
        );
        let candidate = router
            .surface
            .tree
            .node(UiNodeId::new(FIRST_CANDIDATE_NODE_ID))
            .expect("candidate must remain");
        assert_eq!(candidate as *const UiTreeNode, candidate_address);
        assert_eq!(candidate.layout_cache.frame, next_frame);
        assert_eq!(candidate.z_index, 300);
    }

    #[test]
    fn stable_candidate_frame_uses_exact_runtime_geometry_publication() {
        let mut router = ViewportOverlayPointerRouter::new();
        let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
        let initial = candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(10.0, 20.0, 30.0, 40.0),
            100,
        );
        router.rebuild_surface_from_scratch(viewport_frame, &[initial]);

        let next = candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(50.0, 60.0, 30.0, 40.0),
            100,
        );
        assert!(try_apply_surface_delta(
            &mut router,
            viewport_frame,
            &[next]
        ));

        assert_eq!(
            router
                .surface
                .last_rebuild_report
                .arranged_outer_node_visit_count,
            1
        );
        assert_eq!(
            router
                .surface
                .last_rebuild_report
                .hit_grid_outer_node_visit_count,
            1
        );
        assert_eq!(
            router
                .surface
                .last_rebuild_report
                .render_outer_node_visit_count,
            1
        );
    }

    #[test]
    fn changed_candidate_count_rejects_patch_before_geometry_mutation() {
        let mut router = ViewportOverlayPointerRouter::new();
        let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
        let initial_frame = UiFrame::new(10.0, 20.0, 30.0, 40.0);
        let initial = candidate_entry(FIRST_CANDIDATE_NODE_ID, initial_frame, 100);
        router.rebuild_surface_from_scratch(viewport_frame, &[initial]);

        let next = [
            candidate_entry(
                FIRST_CANDIDATE_NODE_ID,
                UiFrame::new(50.0, 60.0, 70.0, 80.0),
                300,
            ),
            candidate_entry(
                FIRST_CANDIDATE_NODE_ID + 1,
                UiFrame::new(100.0, 120.0, 70.0, 80.0),
                300,
            ),
        ];
        assert!(!try_apply_surface_delta(&mut router, viewport_frame, &next));
        let candidate = router
            .surface
            .tree
            .node(UiNodeId::new(FIRST_CANDIDATE_NODE_ID))
            .expect("rejected patch must preserve the existing candidate");
        assert_eq!(candidate.layout_cache.frame, initial_frame);
        assert_eq!(candidate.z_index, 100);
    }

    #[test]
    fn changed_candidate_id_rejects_patch_before_geometry_mutation() {
        let mut router = ViewportOverlayPointerRouter::new();
        let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
        let initial_frame = UiFrame::new(10.0, 20.0, 30.0, 40.0);
        let initial = candidate_entry(FIRST_CANDIDATE_NODE_ID, initial_frame, 100);
        router.rebuild_surface_from_scratch(viewport_frame, &[initial]);

        let next = candidate_entry(
            FIRST_CANDIDATE_NODE_ID + 1,
            UiFrame::new(50.0, 60.0, 70.0, 80.0),
            300,
        );
        assert!(!try_apply_surface_delta(
            &mut router,
            viewport_frame,
            &[next]
        ));
        let candidate = router
            .surface
            .tree
            .node(UiNodeId::new(FIRST_CANDIDATE_NODE_ID))
            .expect("rejected patch must preserve the existing candidate");
        assert_eq!(candidate.layout_cache.frame, initial_frame);
        assert_eq!(candidate.z_index, 100);
    }

    #[test]
    fn changed_candidate_route_releases_pointer_capture_before_reuse() {
        let mut router = ViewportOverlayPointerRouter::new();
        let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
        let candidate_id = UiNodeId::new(FIRST_CANDIDATE_NODE_ID);
        let initial = candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(10.0, 20.0, 30.0, 40.0),
            100,
        );
        router.rebuild_surface_from_scratch(viewport_frame, &[initial]);
        lock_shared_resolution_state(router.shared.as_ref())
            .candidates
            .insert(
                candidate_id,
                PrecisionCandidate {
                    route: ViewportPointerRoute::Renderable {
                        owner: FIRST_CANDIDATE_NODE_ID,
                    },
                    priority: 0,
                    shape: PrecisionShape::Circle {
                        center: Vec2::new(10.0, 20.0),
                        radius_px: 1.0,
                        threshold_px: 0.0,
                        depth: 0.0,
                    },
                },
            );
        router
            .surface
            .capture_pointer(candidate_id)
            .expect("candidate must be a valid pointer-capture owner");

        let mut next = candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(50.0, 60.0, 70.0, 80.0),
            300,
        );
        next.candidate.route = ViewportPointerRoute::Renderable { owner: 999 };
        assert!(try_apply_surface_delta(
            &mut router,
            viewport_frame,
            &[next]
        ));
        assert_eq!(router.surface.focus.captured, None);
    }

    #[test]
    fn stable_candidate_keys_patch_shared_map_values_without_reallocating_nodes() {
        let candidate_id = UiNodeId::new(FIRST_CANDIDATE_NODE_ID);
        let mut shared =
            crate::scene::viewport::pointer::precision::SharedResolutionState::default();
        let initial = candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(10.0, 20.0, 30.0, 40.0),
            100,
        );
        shared
            .candidates
            .insert(candidate_id, initial.candidate.clone());
        let initial_address = shared
            .candidates
            .get(&candidate_id)
            .expect("initial candidate must exist")
            as *const PrecisionCandidate;

        let mut next = candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(50.0, 60.0, 70.0, 80.0),
            300,
        );
        next.candidate.route = ViewportPointerRoute::Renderable { owner: 999 };
        assert!(publish_retained_candidates(&mut shared, vec![next], true));

        let current = shared
            .candidates
            .get(&candidate_id)
            .expect("patched candidate must exist");
        assert_eq!(current as *const PrecisionCandidate, initial_address);
        assert_eq!(
            current.route,
            ViewportPointerRoute::Renderable { owner: 999 }
        );
    }

    #[test]
    fn changed_candidate_keys_rebuild_shared_map_before_value_publication() {
        let mut shared =
            crate::scene::viewport::pointer::precision::SharedResolutionState::default();
        let initial = candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(10.0, 20.0, 30.0, 40.0),
            100,
        );
        shared.candidates.insert(initial.node_id, initial.candidate);

        let next = candidate_entry(
            FIRST_CANDIDATE_NODE_ID + 1,
            UiFrame::new(50.0, 60.0, 70.0, 80.0),
            300,
        );
        assert!(!publish_retained_candidates(&mut shared, vec![next], false));
        assert!(!shared
            .candidates
            .contains_key(&UiNodeId::new(FIRST_CANDIDATE_NODE_ID)));
        assert!(shared
            .candidates
            .contains_key(&UiNodeId::new(FIRST_CANDIDATE_NODE_ID + 1)));
    }

    fn try_apply_surface_delta(
        router: &mut ViewportOverlayPointerRouter,
        viewport_frame: UiFrame,
        candidates: &[RetainedCandidateEntry],
    ) -> bool {
        let classification = router.classify_surface_delta(viewport_frame, candidates);
        if matches!(&classification.delta, ViewportOverlaySurfaceDelta::Topology) {
            return false;
        }
        router.apply_surface_delta(viewport_frame, candidates, classification.delta);
        true
    }

    fn candidate_entry(node_id: u64, frame: UiFrame, z_index: i32) -> RetainedCandidateEntry {
        RetainedCandidateEntry {
            node_id: UiNodeId::new(node_id),
            path_suffix: node_id.saturating_add(1),
            frame,
            z_index,
            candidate: PrecisionCandidate {
                route: ViewportPointerRoute::Renderable { owner: node_id },
                priority: 0,
                shape: PrecisionShape::Circle {
                    center: Vec2::new(frame.x, frame.y),
                    radius_px: 1.0,
                    threshold_px: 0.0,
                    depth: 0.0,
                },
            },
        }
    }
}
