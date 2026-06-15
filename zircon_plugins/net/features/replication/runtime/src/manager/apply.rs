use zircon_runtime::core::framework::net::{
    NetObjectId, SyncDelta, SyncFieldValue, SyncObjectSnapshot,
};

use super::NetReplicationRuntimeManager;

pub const DEFAULT_TRANSFORM_INTERPOLATION_DELAY_MS: u64 = 100;
const INTERPOLATION_SAMPLE_LIMIT: usize = 8;

impl NetReplicationRuntimeManager {
    pub fn apply_delta(&self, delta: SyncDelta) -> Option<SyncObjectSnapshot> {
        self.apply_delta_impl(delta)
    }

    pub fn apply_delta_at(
        &self,
        delta: SyncDelta,
        receive_time_ms: u64,
    ) -> Option<SyncObjectSnapshot> {
        self.apply_delta_at_impl(delta, receive_time_ms)
    }

    pub fn interpolated_f32_field(
        &self,
        object: NetObjectId,
        component_type: &str,
        field_name: &str,
        render_time_ms: u64,
    ) -> Option<f32> {
        self.interpolated_f32_field_with_delay(
            object,
            component_type,
            field_name,
            render_time_ms,
            DEFAULT_TRANSFORM_INTERPOLATION_DELAY_MS,
        )
    }

    pub fn interpolated_f32_field_with_delay(
        &self,
        object: NetObjectId,
        component_type: &str,
        field_name: &str,
        render_time_ms: u64,
        delay_ms: u64,
    ) -> Option<f32> {
        let target_time_ms = render_time_ms.saturating_sub(delay_ms);
        let state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let samples = state.interpolation_samples.get(&(
            object,
            component_type.to_string(),
            field_name.to_string(),
        ))?;
        interpolate_f32_samples(samples, target_time_ms)
    }

    pub(in crate::manager) fn apply_delta_impl(
        &self,
        delta: SyncDelta,
    ) -> Option<SyncObjectSnapshot> {
        self.apply_delta_at_impl(delta, 0)
    }

    pub(in crate::manager) fn apply_delta_at_impl(
        &self,
        delta: SyncDelta,
        receive_time_ms: u64,
    ) -> Option<SyncObjectSnapshot> {
        let mut state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let key = (delta.object, delta.component_type.clone());
        if state
            .sequences
            .get(&key)
            .is_some_and(|sequence| delta.sequence <= *sequence)
        {
            return state.snapshots.get(&key).cloned();
        }

        if delta.is_despawn() {
            state.sequences.insert(key.clone(), delta.sequence);
            state.remove_replication_times(delta.object, &delta.component_type);
            state.remove_interpolation_samples(delta.object, &delta.component_type);
            state.snapshots.remove(&key);
            return None;
        }

        let descriptor = state.descriptors.get(&delta.component_type)?.clone();
        let mut fields = state
            .snapshots
            .get(&key)
            .map(|snapshot| snapshot.fields.clone())
            .unwrap_or_default();
        let changed_fields = delta.changed_fields;
        if should_record_interpolation(&delta.component_type) {
            record_interpolation_samples(
                &mut state,
                delta.object,
                &delta.component_type,
                receive_time_ms,
                &changed_fields,
            );
        }
        merge_delta_fields(&mut fields, changed_fields);

        let snapshot = SyncObjectSnapshot::new(delta.object, &descriptor, fields);
        state.sequences.insert(key.clone(), delta.sequence);
        state.snapshots.insert(key, snapshot.clone());
        Some(snapshot)
    }
}

fn should_record_interpolation(component_type: &str) -> bool {
    component_type.eq_ignore_ascii_case("Transform")
        || component_type
            .split("::")
            .any(|segment| segment.eq_ignore_ascii_case("Transform"))
        || component_type.to_ascii_lowercase().contains("transform")
}

fn record_interpolation_samples(
    state: &mut super::state::NetReplicationRuntimeState,
    object: NetObjectId,
    component_type: &str,
    time_ms: u64,
    changed_fields: &[SyncFieldValue],
) {
    for field in changed_fields {
        if f32_from_bytes(&field.bytes).is_none() {
            continue;
        }
        let samples = state
            .interpolation_samples
            .entry((object, component_type.to_string(), field.name.clone()))
            .or_default();
        samples.push(super::state::NetReplicationInterpolationSample {
            time_ms,
            bytes: field.bytes.clone(),
        });
        samples.sort_by(|left, right| left.time_ms.cmp(&right.time_ms));
        if samples.len() > INTERPOLATION_SAMPLE_LIMIT {
            let overflow = samples.len() - INTERPOLATION_SAMPLE_LIMIT;
            samples.drain(0..overflow);
        }
    }
}

fn merge_delta_fields(fields: &mut Vec<SyncFieldValue>, changed_fields: Vec<SyncFieldValue>) {
    for changed in changed_fields {
        if let Some(existing) = fields.iter_mut().find(|field| field.name == changed.name) {
            *existing = changed;
        } else {
            fields.push(changed);
        }
    }
}

fn interpolate_f32_samples(
    samples: &[super::state::NetReplicationInterpolationSample],
    target_time_ms: u64,
) -> Option<f32> {
    let mut previous = None;
    for sample in samples {
        if sample.time_ms == target_time_ms {
            return f32_from_bytes(&sample.bytes);
        }
        if sample.time_ms < target_time_ms {
            previous = Some(sample);
            continue;
        }

        let next_value = f32_from_bytes(&sample.bytes)?;
        let Some(previous) = previous else {
            return Some(next_value);
        };
        let previous_value = f32_from_bytes(&previous.bytes)?;
        let span = sample.time_ms.saturating_sub(previous.time_ms);
        if span == 0 {
            return Some(next_value);
        }
        let alpha = target_time_ms.saturating_sub(previous.time_ms) as f32 / span as f32;
        return Some(previous_value + (next_value - previous_value) * alpha);
    }

    previous.and_then(|sample| f32_from_bytes(&sample.bytes))
}

fn f32_from_bytes(bytes: &[u8]) -> Option<f32> {
    let bytes = bytes.get(..4)?;
    Some(f32::from_le_bytes(bytes.try_into().ok()?))
}
