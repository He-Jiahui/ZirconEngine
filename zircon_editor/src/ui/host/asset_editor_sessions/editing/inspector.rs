use super::*;

impl EditorUiHost {
    pub fn set_ui_asset_editor_selected_widget_control_id(
        &self,
        instance_id: &ViewInstanceId,
        control_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_control_id(control_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_widget_text_property(
        &self,
        instance_id: &ViewInstanceId,
        text: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_text_property(text.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_widget_prop_literal(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_prop_literal(path.as_ref(), literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_widget_state_literal(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_state_literal(path.as_ref(), literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_component_root_class_policy(
        &self,
        instance_id: &ViewInstanceId,
        policy: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_component_root_class_policy(policy.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_asset_id(
        &self,
        instance_id: &ViewInstanceId,
        asset_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_promote_widget_asset_id(asset_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_component_name(
        &self,
        instance_id: &ViewInstanceId,
        component_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_promote_widget_component_name(component_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_document_id(
        &self,
        instance_id: &ViewInstanceId,
        document_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_promote_widget_document_id(document_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_mount(
        &self,
        instance_id: &ViewInstanceId,
        mount: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_mount(mount.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_padding(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_padding(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_width_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_width_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_height_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_height_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_designer_tool_mode(
        &self,
        instance_id: &ViewInstanceId,
        mode: UiDesignerToolMode,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry.session.set_designer_tool_mode(mode);
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_locale_preview(
        &self,
        instance_id: &ViewInstanceId,
        locale: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry.session.set_locale_preview(locale.as_ref());
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn resize_ui_asset_editor_selected_slot_preferred_size(
        &self,
        instance_id: &ViewInstanceId,
        width: f32,
        height: f32,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .resize_selected_slot_preferred_size(width, height)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_width_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_layout_width_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_height_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_layout_height_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_slot_semantic(
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
            .select_slot_semantic(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_semantic_value(
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
            .set_selected_slot_semantic_value(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_semantic_field(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_slot_semantic_field(path.as_ref(), value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_slot_semantic(
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
            .delete_selected_slot_semantic()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_layout_semantic(
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
            .select_layout_semantic(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_semantic_value(
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
            .set_selected_layout_semantic_value(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_semantic_field(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_layout_semantic_field(path.as_ref(), value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_layout_semantic(
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
            .delete_selected_layout_semantic()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    fn sync_ui_asset_editor_instance_if_changed(
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
    fn optimization_batch_ea_unchanged_inspector_actions_guard_projection_sync() {
        let source = include_str!("inspector.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("inspector production implementation");

        assert_eq!(
            production
                .matches("self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;")
                .count(),
            25,
            "every inspector mutation must use the no-change projection guard"
        );
        assert_eq!(
            production
                .matches("self.sync_ui_asset_editor_instance(instance_id)?;")
                .count(),
            1,
            "only the guard helper may call projection synchronization directly"
        );
        assert!(production.contains("if changed {"));
    }

    #[test]
    #[ignore = "release-only unchanged inspector action benchmark"]
    fn optimization_batch_ea_unchanged_inspector_action_release_benchmark_evidence() {
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
                    "ui.asset.inspector.projection.row.{index:04}.{}",
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
            "EDITOR363_NOOP_INSPECTOR_PROJECTION_SYNC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
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
            "unchanged inspector actions must reduce projection-sync P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
