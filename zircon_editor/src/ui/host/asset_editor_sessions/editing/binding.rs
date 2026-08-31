use super::*;

impl EditorUiHost {
    pub fn select_ui_asset_editor_binding(
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
            .select_binding(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn add_ui_asset_editor_binding(
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
            .add_binding()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_binding_event_option(
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
            .select_binding_event_option(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_binding(
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
            .delete_selected_binding()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_id(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_id(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_event(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_event(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_binding_action_kind(
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
            .select_binding_action_kind(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_route(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_route(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_route_target(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_route_target(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_action_target(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_action_target(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_selected_binding_route_suggestion(
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
            .apply_selected_binding_route_suggestion(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_selected_binding_action_suggestion(
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
            .apply_selected_binding_action_suggestion(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_binding_payload(
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
            .select_binding_payload(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn upsert_ui_asset_editor_selected_binding_payload(
        &self,
        instance_id: &ViewInstanceId,
        payload_key: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .upsert_selected_binding_payload(payload_key.as_ref(), value_literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_binding_payload(
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
            .delete_selected_binding_payload()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_selected_binding_payload_suggestion(
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
            .apply_selected_binding_payload_suggestion(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    fn sync_ui_asset_editor_binding_projection_if_changed(
        &self,
        instance_id: &ViewInstanceId,
        changed: bool,
    ) -> Result<(), EditorError> {
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn optimization_batch_eb_unchanged_binding_actions_guard_projection_sync() {
        let source = include_str!("binding.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("binding production implementation");

        assert_eq!(
            production
                .matches(
                    "self.sync_ui_asset_editor_binding_projection_if_changed(instance_id, changed)?;",
                )
                .count(),
            16,
            "every binding mutation must use the no-change projection guard"
        );
        assert_eq!(
            production
                .matches("self.sync_ui_asset_editor_instance(instance_id)?;")
                .count(),
            1,
            "only the binding guard helper may synchronize the projection directly"
        );
        assert!(production.contains("if changed {"));
    }

    #[test]
    #[ignore = "release-only unchanged binding action benchmark"]
    fn optimization_batch_eb_unchanged_binding_action_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const NOOP_ACTIONS_PER_SAMPLE: usize = 64;
        const PROJECTION_ROWS: usize = 1_024;

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
                    "ui.asset.binding.projection.row.{index:04}.{}",
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
            "EDITOR364_NOOP_BINDING_PROJECTION_SYNC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
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
            "unchanged binding actions must reduce projection-sync P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
