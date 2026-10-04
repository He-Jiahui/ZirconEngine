use super::MeshPassCommandBuffers;

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct MeshDrawCommandListProfile {
    grow_count: usize,
    peak_capacity: usize,
    merge_move_count: usize,
    merge_command_visit_count: usize,
    sort_count: usize,
    sort_command_visit_count: usize,
}

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct MeshPassCommandMaterializationAccumulator {
    command_arena_grow_count: usize,
    command_arena_peak_capacity: usize,
    command_build_count: usize,
    partition_move_count: usize,
    partition_command_visit_count: usize,
    merge_move_count: usize,
    merge_command_visit_count: usize,
    finalize_count: usize,
    cache_hit_payload_arc_clone_count: usize,
}

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct MeshPassCommandMaterializationProfile {
    pub(super) command_arena_grow_count: usize,
    pub(super) command_arena_peak_capacity: usize,
    pub(super) phase_bucket_grow_count: usize,
    pub(super) phase_bucket_capacity: usize,
    pub(super) command_build_count: usize,
    pub(super) partition_move_count: usize,
    pub(super) merge_move_count: usize,
    pub(super) finalize_count: usize,
    pub(super) sort_count: usize,
    pub(super) partition_command_visit_count: usize,
    pub(super) merge_command_visit_count: usize,
    pub(super) sort_command_visit_count: usize,
    pub(super) active_bucket_count: usize,
    pub(super) cache_hit_payload_arc_clone_count: usize,
    pub(super) command_generation_reuse_count: usize,
    pub(super) view_generation_reuse_count: usize,
    pub(super) depth_prepass_length: usize,
    pub(super) shadow_length: usize,
    pub(super) opaque_length: usize,
    pub(super) alpha_mask_length: usize,
    pub(super) advanced_pbr_opaque_length: usize,
    pub(super) transmission_length: usize,
    pub(super) transparent_length: usize,
    pub(super) half_resolution_transparent_length: usize,
    pub(super) velocity_length: usize,
    pub(super) taa_reactive_mask_length: usize,
}

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
impl MeshDrawCommandListProfile {
    pub(super) fn from_existing_capacity(capacity: usize) -> Self {
        Self {
            grow_count: usize::from(u8::from(capacity > 0)),
            peak_capacity: capacity,
            ..Self::default()
        }
    }

    pub(super) fn record_capacity_transition(&mut self, previous: usize, current: usize) {
        if current > previous {
            self.grow_count += 1;
        }
        self.peak_capacity = self.peak_capacity.max(current);
    }

    pub(super) fn record_arena_cost(&mut self, grow_count: usize, peak_capacity: usize) {
        self.grow_count += grow_count;
        self.peak_capacity = self.peak_capacity.max(peak_capacity);
    }

    pub(super) fn arena_cost(&self) -> (usize, usize) {
        (self.grow_count, self.peak_capacity)
    }

    pub(super) fn record_merge(&mut self, moved: usize, visited: usize) {
        self.merge_move_count += moved;
        self.merge_command_visit_count += visited;
    }

    pub(super) fn record_sort(&mut self, command_count: usize) {
        self.sort_count += 1;
        self.sort_command_visit_count += command_count;
    }

    pub(super) fn accumulate(&mut self, other: Self) {
        self.grow_count += other.grow_count;
        self.peak_capacity = self.peak_capacity.max(other.peak_capacity);
        self.merge_move_count += other.merge_move_count;
        self.merge_command_visit_count += other.merge_command_visit_count;
        self.sort_count += other.sort_count;
        self.sort_command_visit_count += other.sort_command_visit_count;
    }
}

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
impl MeshPassCommandMaterializationAccumulator {
    pub(super) fn from_build_arena(
        arena: MeshDrawCommandListProfile,
        command_count: usize,
        cache_hit_payload_arc_clone_count: usize,
    ) -> Self {
        Self {
            command_arena_grow_count: arena.grow_count,
            command_arena_peak_capacity: arena.peak_capacity,
            command_build_count: command_count,
            partition_move_count: 0,
            partition_command_visit_count: 0,
            merge_move_count: arena.merge_move_count,
            merge_command_visit_count: arena.merge_command_visit_count,
            finalize_count: 1,
            cache_hit_payload_arc_clone_count,
        }
    }

    pub(super) fn record_discarded_attempt(
        &mut self,
        command_count: usize,
        cache_hit_payload_arc_clone_count: usize,
    ) {
        self.command_build_count += command_count;
        self.cache_hit_payload_arc_clone_count += cache_hit_payload_arc_clone_count;
    }

    pub(super) fn record_partition_move(&mut self) {
        self.partition_move_count += 1;
    }

    pub(super) fn record_partition_visit(&mut self) {
        self.partition_command_visit_count += 1;
    }

    pub(super) fn accumulate(&mut self, other: Self) {
        self.command_arena_grow_count += other.command_arena_grow_count;
        self.command_arena_peak_capacity = self
            .command_arena_peak_capacity
            .max(other.command_arena_peak_capacity);
        self.command_build_count += other.command_build_count;
        self.partition_move_count += other.partition_move_count;
        self.partition_command_visit_count += other.partition_command_visit_count;
        self.merge_move_count += other.merge_move_count;
        self.merge_command_visit_count += other.merge_command_visit_count;
        self.finalize_count += other.finalize_count;
        self.cache_hit_payload_arc_clone_count += other.cache_hit_payload_arc_clone_count;
    }
}

impl MeshPassCommandBuffers {
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    pub(super) fn materialization_profile(&self) -> MeshPassCommandMaterializationProfile {
        let lists = [
            &self.depth_prepass,
            &self.shadow,
            &self.opaque,
            &self.alpha_mask,
            &self.advanced_pbr_opaque,
            &self.transmission,
            &self.transparent,
            &self.half_resolution_transparent,
            &self.velocity,
            &self.taa_reactive_mask,
        ];
        let phase_bucket_grow_count = lists.iter().map(|list| list.profile.grow_count).sum();
        let phase_bucket_capacity = lists.iter().map(|list| list.commands.capacity()).sum();
        let phase_merge_move_count: usize =
            lists.iter().map(|list| list.profile.merge_move_count).sum();
        let phase_merge_command_visit_count: usize = lists
            .iter()
            .map(|list| list.profile.merge_command_visit_count)
            .sum();
        let sort_count = lists.iter().map(|list| list.profile.sort_count).sum();
        let sort_command_visit_count = lists
            .iter()
            .map(|list| list.profile.sort_command_visit_count)
            .sum();
        let active_bucket_count = lists
            .iter()
            .filter(|list| !list.commands.is_empty())
            .count();

        MeshPassCommandMaterializationProfile {
            command_arena_grow_count: self.materialization_stats.command_arena_grow_count,
            command_arena_peak_capacity: self.materialization_stats.command_arena_peak_capacity,
            phase_bucket_grow_count,
            phase_bucket_capacity,
            command_build_count: self.materialization_stats.command_build_count,
            partition_move_count: self.materialization_stats.partition_move_count,
            merge_move_count: self
                .materialization_stats
                .merge_move_count
                .saturating_add(phase_merge_move_count),
            finalize_count: self.materialization_stats.finalize_count,
            sort_count,
            partition_command_visit_count: self.materialization_stats.partition_command_visit_count,
            merge_command_visit_count: self
                .materialization_stats
                .merge_command_visit_count
                .saturating_add(phase_merge_command_visit_count),
            sort_command_visit_count,
            active_bucket_count,
            // Includes accepted hits and cache payloads cloned by an attempt that later fell
            // back to the residual path.
            cache_hit_payload_arc_clone_count: self
                .materialization_stats
                .cache_hit_payload_arc_clone_count,
            command_generation_reuse_count: 0,
            view_generation_reuse_count: 0,
            depth_prepass_length: self.depth_prepass.commands.len(),
            shadow_length: self.shadow.commands.len(),
            opaque_length: self.opaque.commands.len(),
            alpha_mask_length: self.alpha_mask.commands.len(),
            advanced_pbr_opaque_length: self.advanced_pbr_opaque.commands.len(),
            transmission_length: self.transmission.commands.len(),
            transparent_length: self.transparent.commands.len(),
            half_resolution_transparent_length: self.half_resolution_transparent.commands.len(),
            velocity_length: self.velocity.commands.len(),
            taa_reactive_mask_length: self.taa_reactive_mask.commands.len(),
        }
    }

    #[inline]
    pub(crate) fn record_materialization_profile(&self) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            let profile = self.materialization_profile();
            crate::core::diagnostics::profiling::record_counter_batch(
                "render",
                &[
                    (
                        "mesh_commands.command_arena_grow_count",
                        profile.command_arena_grow_count as f64,
                    ),
                    (
                        "mesh_commands.command_arena_peak_capacity",
                        profile.command_arena_peak_capacity as f64,
                    ),
                    (
                        "mesh_commands.phase_bucket_grow_count",
                        profile.phase_bucket_grow_count as f64,
                    ),
                    (
                        "mesh_commands.phase_bucket_capacity",
                        profile.phase_bucket_capacity as f64,
                    ),
                    (
                        "mesh_commands.command_build_count",
                        profile.command_build_count as f64,
                    ),
                    (
                        "mesh_commands.partition_move_count",
                        profile.partition_move_count as f64,
                    ),
                    (
                        "mesh_commands.merge_move_count",
                        profile.merge_move_count as f64,
                    ),
                    (
                        "mesh_commands.finalize_count",
                        profile.finalize_count as f64,
                    ),
                    ("mesh_commands.sort_count", profile.sort_count as f64),
                    (
                        "mesh_commands.partition_command_visit_count",
                        profile.partition_command_visit_count as f64,
                    ),
                    (
                        "mesh_commands.merge_command_visit_count",
                        profile.merge_command_visit_count as f64,
                    ),
                    (
                        "mesh_commands.sort_command_visit_count",
                        profile.sort_command_visit_count as f64,
                    ),
                    (
                        "mesh_commands.active_bucket_count",
                        profile.active_bucket_count as f64,
                    ),
                    (
                        "mesh_commands.cache_hit_payload_arc_clone_count",
                        profile.cache_hit_payload_arc_clone_count as f64,
                    ),
                    (
                        "mesh_commands.command_generation_reuse_count",
                        profile.command_generation_reuse_count as f64,
                    ),
                    (
                        "mesh_commands.view_generation_reuse_count",
                        profile.view_generation_reuse_count as f64,
                    ),
                    (
                        "mesh_commands.bucket.depth_prepass_length",
                        profile.depth_prepass_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.shadow_length",
                        profile.shadow_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.opaque_length",
                        profile.opaque_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.alpha_mask_length",
                        profile.alpha_mask_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.advanced_pbr_opaque_length",
                        profile.advanced_pbr_opaque_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.transmission_length",
                        profile.transmission_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.transparent_length",
                        profile.transparent_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.half_resolution_transparent_length",
                        profile.half_resolution_transparent_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.velocity_length",
                        profile.velocity_length as f64,
                    ),
                    (
                        "mesh_commands.bucket.taa_reactive_mask_length",
                        profile.taa_reactive_mask_length as f64,
                    ),
                ],
            );
            crate::core::diagnostics::profiling::record_counter_batch(
                "render",
                &[(
                    "mesh_commands.cache_resolver_configuration_invalidation_count",
                    self.cache_stats
                        .cache_invalidated_resolver_configuration_count as f64,
                )],
            );
        }
    }

    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    pub(crate) fn record_discarded_materialization_attempt(
        &mut self,
        command_count: usize,
        cache_hit_payload_arc_clone_count: usize,
    ) {
        self.materialization_stats
            .record_discarded_attempt(command_count, cache_hit_payload_arc_clone_count);
    }
}
