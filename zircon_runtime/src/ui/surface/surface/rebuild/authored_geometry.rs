use std::{collections::BTreeSet, time::Instant};

use crate::ui::surface::{authored_geometry_affected_node_ids, patch_arranged_tree_geometry};
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiSize, tree::UiDirtyFlags};

use super::{elapsed_micros, UiSurface, UiSurfaceRebuildReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAuthoredGeometryFallbackReason {
    PendingMutationOutsideDelta,
    MissingNodeOrArrangedIndex,
    TopologyGenerationChanged,
    ClipDescendantExpansionFailed,
    RenderCommandNotGeometryPatchable,
    ProjectedHitOrNavigationPatchFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAuthoredGeometryPublication {
    Unchanged,
    Local(UiSurfaceRebuildReport),
    FullFallback {
        reason: UiAuthoredGeometryFallbackReason,
        report: UiSurfaceRebuildReport,
    },
}

impl UiSurface {
    /// Publishes exact geometry changes already authored in node layout caches.
    ///
    /// This path never recomputes layout constraints. Callers must supply the topology generation
    /// observed while authoring the frames and the complete set of directly changed node IDs.
    pub fn publish_authored_geometry(
        &mut self,
        root_size: UiSize,
        changed_node_ids: &BTreeSet<UiNodeId>,
        observed_topology_generation: u64,
    ) -> UiAuthoredGeometryPublication {
        if changed_node_ids.is_empty() {
            self.last_layout_root_size = Some(root_size);
            return UiAuthoredGeometryPublication::Unchanged;
        }
        let root_size_changed = self.last_layout_root_size != Some(root_size);
        if self.tree.layout_order_generation() != observed_topology_generation {
            return self.authored_geometry_full_fallback(
                root_size,
                UiAuthoredGeometryFallbackReason::TopologyGenerationChanged,
            );
        }
        let pending_invalidation_node_ids = self.invalidation.pending_changed_node_ids();
        if !self
            .tree
            .pending_mutation_node_ids()
            .is_subset(changed_node_ids)
            || !self.dirty_node_ids.is_subset(changed_node_ids)
            || !pending_invalidation_node_ids.is_subset(changed_node_ids)
        {
            return self.authored_geometry_full_fallback(
                root_size,
                UiAuthoredGeometryFallbackReason::PendingMutationOutsideDelta,
            );
        }
        if changed_node_ids.iter().any(|node_id| {
            self.tree.node(*node_id).is_none()
                || self
                    .arranged_node_indices
                    .get(node_id)
                    .and_then(|index| self.arranged_tree.nodes.get(*index))
                    .is_none_or(|node| node.node_id != *node_id)
        }) {
            return self.authored_geometry_full_fallback(
                root_size,
                UiAuthoredGeometryFallbackReason::MissingNodeOrArrangedIndex,
            );
        }

        let Some(affected_node_ids) =
            authored_geometry_affected_node_ids(&self.tree, changed_node_ids)
        else {
            return self.authored_geometry_full_fallback(
                root_size,
                UiAuthoredGeometryFallbackReason::ClipDescendantExpansionFailed,
            );
        };

        let arranged_start = Instant::now();
        if patch_arranged_tree_geometry(
            &self.tree,
            &mut self.arranged_tree,
            changed_node_ids,
            &affected_node_ids,
            &self.arranged_node_indices,
            &self.arranged_slot_indices,
        )
        .is_none()
        {
            return self.authored_geometry_full_fallback(
                root_size,
                UiAuthoredGeometryFallbackReason::ClipDescendantExpansionFailed,
            );
        }
        let arranged_elapsed_micros = elapsed_micros(arranged_start);

        let hit_start = Instant::now();
        let hit_grid_rebuilt = if self
            .hit_test
            .patch_arranged_geometry(
                &self.arranged_tree,
                &affected_node_ids,
                &self.arranged_node_indices,
            )
            .is_err()
        {
            self.hit_test
                .rebuild_arranged_indexed(&self.arranged_tree, &self.arranged_node_indices);
            crate::profile_counter!("runtime", "ui.authored_geometry.hit_grid_regrid_count", 1);
            true
        } else {
            false
        };
        let hit_grid_elapsed_micros = elapsed_micros(hit_start);

        let render_start = Instant::now();
        let render_geometry_patch = self.render_cache.patch_geometry(
            &mut self.render_extract,
            &self.arranged_tree,
            &self.arranged_node_indices,
            &affected_node_ids,
        );
        let render_stats = match render_geometry_patch {
            Ok(stats) => stats,
            Err(()) => {
                self.text_measure_cache.begin_frame();
                let render_node_patch = self.patch_render_nodes(&affected_node_ids);
                self.text_measure_cache.finish_frame();
                match render_node_patch {
                    Ok(stats) => stats,
                    Err(()) => {
                        return self.authored_geometry_full_fallback(
                            root_size,
                            UiAuthoredGeometryFallbackReason::RenderCommandNotGeometryPatchable,
                        );
                    }
                }
            }
        };
        let render_elapsed_micros = elapsed_micros(render_start);

        let projected_hit_changed = if hit_grid_rebuilt {
            self.synchronize_projected_hit_test(&affected_node_ids, true)
        } else {
            match self.patch_projected_hit_test_strict(&affected_node_ids) {
                Ok(changed) => changed,
                Err(()) => {
                    return self.authored_geometry_full_fallback(
                        root_size,
                        UiAuthoredGeometryFallbackReason::ProjectedHitOrNavigationPatchFailed,
                    );
                }
            }
        };
        if projected_hit_changed && self.navigation_index_patch_projected_geometry().is_err() {
            return self.authored_geometry_full_fallback(
                root_size,
                UiAuthoredGeometryFallbackReason::ProjectedHitOrNavigationPatchFailed,
            );
        }
        if self
            .navigation_index_patch_changed_geometry(&affected_node_ids, &BTreeSet::new())
            .is_err()
        {
            return self.authored_geometry_full_fallback(
                root_size,
                UiAuthoredGeometryFallbackReason::ProjectedHitOrNavigationPatchFailed,
            );
        }

        self.last_layout_geometry_changed_node_ids = affected_node_ids.clone();
        self.last_layout_root_size = Some(root_size);
        let hit_grid_outer_node_visit_count = if hit_grid_rebuilt {
            self.arranged_tree.draw_order.len()
        } else {
            affected_node_ids.len()
        };
        let report = UiSurfaceRebuildReport {
            dirty_flags: UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                ..UiDirtyFlags::default()
            },
            dirty_node_count: changed_node_ids.len(),
            arranged_rebuilt: true,
            hit_grid_rebuilt: true,
            render_rebuilt: true,
            arranged_outer_node_visit_count: affected_node_ids.len(),
            hit_grid_outer_node_visit_count,
            render_outer_node_visit_count: affected_node_ids.len(),
            layout_geometry_changed_node_count: affected_node_ids.len(),
            render_command_reused_count: render_stats.reused_command_count,
            render_command_rebuilt_count: render_stats.rebuilt_command_count,
            render_damage_rect_count: render_stats.damage_rect_count,
            arranged_elapsed_micros,
            hit_grid_elapsed_micros,
            render_elapsed_micros,
            ..UiSurfaceRebuildReport::default().with_counts(self.rebuild_counts())
        };
        self.last_rebuild_report = report;
        let _ = self
            .invalidation
            .commit_pending()
            .expect("surface-owned invalidation transaction must use the current generation");
        self.clear_dirty_flags();
        self.reset_pending_pool_report();
        self.mark_surface_frame_rebuild_dirty(true, true, true, Some(&affected_node_ids));
        self.publish_surface_frame_after_rebuild();
        crate::profile_counter!(
            "runtime",
            "ui.authored_geometry.local_patch_node_count",
            affected_node_ids.len()
        );
        if root_size_changed {
            crate::profile_counter!(
                "runtime",
                "ui.authored_geometry.root_size_local_publication_count",
                1
            );
        }
        UiAuthoredGeometryPublication::Local(report)
    }

    fn authored_geometry_full_fallback(
        &mut self,
        root_size: UiSize,
        reason: UiAuthoredGeometryFallbackReason,
    ) -> UiAuthoredGeometryPublication {
        crate::profile_counter!("runtime", "ui.authored_geometry.full_fallback_count", 1);
        self.rebuild_authored_frames(root_size);
        UiAuthoredGeometryPublication::FullFallback {
            reason,
            report: self.last_rebuild_report,
        }
    }
}
