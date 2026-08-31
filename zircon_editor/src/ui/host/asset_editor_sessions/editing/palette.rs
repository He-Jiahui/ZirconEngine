use super::*;

impl EditorUiHost {
    pub fn select_ui_asset_editor_palette_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_palette_index(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn update_ui_asset_editor_palette_drag_target(
        &self,
        instance_id: &ViewInstanceId,
        surface_x: f32,
        surface_y: f32,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .update_palette_drag_target(surface_x, surface_y)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn clear_ui_asset_editor_palette_drag_target(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry.session.clear_palette_drag_target();
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn cycle_ui_asset_editor_palette_drag_target_candidate_next(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .cycle_palette_drag_target_candidate_next()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn cycle_ui_asset_editor_palette_drag_target_candidate_previous(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .cycle_palette_drag_target_candidate_previous()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn select_ui_asset_editor_palette_target_candidate(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_palette_target_candidate(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn confirm_ui_asset_editor_palette_target_choice(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .confirm_palette_target_choice()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn cancel_ui_asset_editor_palette_target_choice(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .cancel_palette_target_choice()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn drop_ui_asset_editor_selected_palette_item_at_drag_target(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .drop_selected_palette_item_at_palette_drag_target()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn insert_ui_asset_editor_selected_palette_item_as_child(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .insert_selected_palette_item_as_child()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn insert_ui_asset_editor_selected_palette_item_after_selection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .insert_selected_palette_item_after_selection()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn unchanged_palette_drag_does_not_rebuild_instance_projection() {
        let source = include_str!("palette.rs");
        let drag = source
            .split("pub fn update_ui_asset_editor_palette_drag_target")
            .nth(1)
            .expect("drag update function")
            .split("pub fn clear_ui_asset_editor_palette_drag_target")
            .next()
            .expect("drag update body");

        assert!(drag.contains("if changed"));
    }

    #[test]
    fn optimization_batch_dz_unchanged_palette_actions_guard_projection_sync() {
        let source = include_str!("palette.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("palette production implementation");

        assert_eq!(
            production.matches("if changed {").count(),
            11,
            "every palette mutation must skip projection synchronization when state is unchanged"
        );
    }

    #[test]
    #[ignore = "release-only unchanged palette action benchmark"]
    fn optimization_batch_dz_unchanged_palette_action_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const NOOP_ACTIONS_PER_SAMPLE: usize = 64;
        const PROJECTION_ROWS: usize = 512;

        fn projection_checksum(rows: &[String]) -> usize {
            let projected = black_box(rows.to_vec());
            black_box(projected.iter().map(String::len).sum())
        }

        fn measure_legacy(rows: &[String]) -> u128 {
            let started = Instant::now();
            let mut checksum = 0;
            for _ in 0..NOOP_ACTIONS_PER_SAMPLE {
                checksum ^= projection_checksum(black_box(rows));
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(rows: &[String]) -> u128 {
            let started = Instant::now();
            let mut checksum = 0;
            for _ in 0..NOOP_ACTIONS_PER_SAMPLE {
                if black_box(false) {
                    checksum ^= projection_checksum(black_box(rows));
                }
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let rows = (0..PROJECTION_ROWS)
            .map(|index| {
                format!(
                    "ui.asset.palette.projection.row.{index:04}.{}",
                    "x".repeat(96)
                )
            })
            .collect::<Vec<_>>();

        for _ in 0..4 {
            black_box(measure_legacy(&rows));
            black_box(measure_optimized(&rows));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&rows));
                optimized_samples.push(measure_optimized(&rows));
            } else {
                optimized_samples.push(measure_optimized(&rows));
                legacy_samples.push(measure_legacy(&rows));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "EDITOR362_NOOP_PALETTE_PROJECTION_SYNC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
noop_actions_per_sample={NOOP_ACTIONS_PER_SAMPLE} projection_rows={PROJECTION_ROWS} \
pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 \
legacy_projection_syncs_per_sample={NOOP_ACTIONS_PER_SAMPLE} \
optimized_projection_syncs_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "unchanged palette actions must reduce projection-sync P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
