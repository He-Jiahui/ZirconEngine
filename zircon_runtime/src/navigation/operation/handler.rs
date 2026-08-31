use crate::core::framework::navigation::{
    NavMeshBakeRequest, NavigationClearBakeRequest, NavigationGeneratedBakeSnapshot,
};
use crate::operation::{
    RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationHandlerError,
    RuntimeOperationPrepared,
};
use crate::scene::{
    SceneNavigationRuntime, SceneNavigationRuntimeHandle, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME,
};

#[derive(Clone, Copy)]
pub(super) enum NavigationOperationKind {
    BakeScene,
    BakeSurface,
    ClearSurface,
    RestoreSnapshot,
}

pub(super) struct NavigationOperationHandler {
    kind: NavigationOperationKind,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct NavigationSnapshotChange {
    before: NavigationGeneratedBakeSnapshot,
    after: NavigationGeneratedBakeSnapshot,
}

impl NavigationOperationHandler {
    pub(super) fn new(kind: NavigationOperationKind) -> Self {
        Self { kind }
    }

    fn resolve_runtime(
        context: &RuntimeOperationContext<'_>,
    ) -> Result<std::sync::Arc<SceneNavigationRuntimeHandle>, RuntimeOperationHandlerError> {
        context
            .core()
            .resolve_driver::<SceneNavigationRuntimeHandle>(SCENE_NAVIGATION_RUNTIME_DRIVER_NAME)
            .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))
    }

    fn snapshot_clear(
        context: RuntimeOperationContext<'_>,
        request: NavigationClearBakeRequest,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        let runtime = Self::resolve_runtime(&context)?;
        let before = runtime.generated_bake_snapshot(request.surface_entity);
        let target = before.surface_entity.or(request.surface_entity);
        encode_snapshot_change(NavigationSnapshotChange {
            before,
            after: NavigationGeneratedBakeSnapshot::empty(target),
        })
    }

    fn snapshot_restore(
        context: RuntimeOperationContext<'_>,
        snapshot: NavigationGeneratedBakeSnapshot,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        let runtime = Self::resolve_runtime(&context)?;
        let before = runtime.generated_bake_snapshot(snapshot.surface_entity);
        encode_snapshot_change(NavigationSnapshotChange {
            before,
            after: snapshot,
        })
    }

    fn prepare_snapshot_change(
        snapshot: serde_json::Value,
        operation: &str,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        let change: NavigationSnapshotChange = decode_payload(snapshot, operation)?;
        let (command, result) = encode_prepared_snapshot_values(change)?;
        Ok(RuntimeOperationPrepared::new(command, result))
    }

    fn apply_snapshot_change(
        context: RuntimeOperationContext<'_>,
        command: serde_json::Value,
        operation: &str,
    ) -> Result<(), RuntimeOperationHandlerError> {
        let change: NavigationSnapshotChange = decode_payload(command, operation)?;
        let runtime = Self::resolve_runtime(&context)?;
        let current = runtime.generated_bake_snapshot(change.after.surface_entity);
        if current != change.before {
            return Err(RuntimeOperationHandlerError::new(
                "navigation generated bake state changed after operation snapshot",
            ));
        }
        runtime
            .replace_generated_bake_snapshot(change.after)
            .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))
    }
}

impl RuntimeOperationHandler for NavigationOperationHandler {
    fn snapshot(
        &self,
        context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        match self.kind {
            NavigationOperationKind::BakeScene => {
                let mut request: NavMeshBakeRequest =
                    decode_payload(payload, "navigation scene bake")?;
                request.surface_entity = None;
                encode_payload(request, "navigation scene bake")
            }
            NavigationOperationKind::BakeSurface => {
                let request: NavMeshBakeRequest =
                    decode_payload(payload, "navigation surface bake")?;
                if request.surface_entity.is_none() {
                    return Err(RuntimeOperationHandlerError::new(
                        "navigation surface bake requires surface_entity",
                    ));
                }
                encode_payload(request, "navigation surface bake")
            }
            NavigationOperationKind::ClearSurface => Self::snapshot_clear(
                context,
                decode_payload(payload, "navigation surface clear")?,
            ),
            NavigationOperationKind::RestoreSnapshot => Self::snapshot_restore(
                context,
                decode_payload(payload, "navigation bake snapshot restore")?,
            ),
        }
    }

    fn prepare(
        &self,
        snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        match self.kind {
            NavigationOperationKind::BakeScene | NavigationOperationKind::BakeSurface => {
                let _request: NavMeshBakeRequest = decode_payload(snapshot, "navigation bake")?;
                Err(RuntimeOperationHandlerError::new(
                    "navigation bake requires a pure prepare backend",
                ))
            }
            NavigationOperationKind::ClearSurface => {
                Self::prepare_snapshot_change(snapshot, "navigation surface clear")
            }
            NavigationOperationKind::RestoreSnapshot => {
                Self::prepare_snapshot_change(snapshot, "navigation bake snapshot restore")
            }
        }
    }

    fn apply(
        &self,
        context: RuntimeOperationContext<'_>,
        command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        match self.kind {
            NavigationOperationKind::BakeScene | NavigationOperationKind::BakeSurface => {
                let _command: NavMeshBakeRequest = decode_payload(command, "navigation bake")?;
                Err(RuntimeOperationHandlerError::new(
                    "navigation bake cannot reach owner apply without a prepared command",
                ))
            }
            NavigationOperationKind::ClearSurface => {
                Self::apply_snapshot_change(context, command, "navigation surface clear")
            }
            NavigationOperationKind::RestoreSnapshot => {
                Self::apply_snapshot_change(context, command, "navigation bake snapshot restore")
            }
        }
    }
}

fn decode_payload<T: serde::de::DeserializeOwned>(
    payload: serde_json::Value,
    operation: &str,
) -> Result<T, RuntimeOperationHandlerError> {
    serde_json::from_value(payload).map_err(|error| {
        RuntimeOperationHandlerError::new(format!("invalid {operation} payload: {error}"))
    })
}

fn encode_prepared_snapshot_values(
    change: NavigationSnapshotChange,
) -> Result<(serde_json::Value, serde_json::Value), RuntimeOperationHandlerError> {
    let command = encode_snapshot_change(change)?;
    let mut result = command.clone();
    let Some(result_fields) = result.as_object_mut() else {
        return Err(RuntimeOperationHandlerError::new(
            "encode navigation generated bake change: snapshot was not an object",
        ));
    };
    result_fields.insert("report".to_owned(), serde_json::Value::Null);
    Ok((command, result))
}

fn encode_snapshot_change(
    change: NavigationSnapshotChange,
) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
    serde_json::to_value(change).map_err(|error| {
        RuntimeOperationHandlerError::new(format!(
            "encode navigation generated bake snapshot: {error}"
        ))
    })
}

fn encode_payload<T: serde::Serialize>(
    payload: T,
    operation: &str,
) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
    serde_json::to_value(payload).map_err(|error| {
        RuntimeOperationHandlerError::new(format!("encode {operation} payload: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::navigation::{
        NavMeshAsset, NavigationGeneratedBakeChange, NavigationGeneratedBakeSnapshot,
    };

    use super::{encode_prepared_snapshot_values, NavigationSnapshotChange};

    #[test]
    fn optimization_batch_dx_navigation_snapshot_values_preserve_serialized_contracts() {
        let change = navigation_change_fixture(8);
        let expected_command = serde_json::to_value(&change).expect("legacy command value");
        let expected_result = serde_json::to_value(NavigationGeneratedBakeChange {
            before: change.before.clone(),
            after: change.after.clone(),
            report: None,
        })
        .expect("legacy result value");

        let (command, result) =
            encode_prepared_snapshot_values(change).expect("prepared snapshot values");

        assert_eq!(command, expected_command);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn optimization_batch_dx_navigation_snapshot_values_serialize_once() {
        let production = include_str!("handler.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("navigation operation production source");
        let encoder = production
            .split("fn encode_prepared_snapshot_values")
            .nth(1)
            .expect("single serialization encoder");

        assert!(encoder.contains("let command = encode_snapshot_change(change)?"));
        assert!(encoder.contains("let mut result = command.clone()"));
        assert!(!encoder.contains("change.before.clone()"));
        assert!(!encoder.contains("change.after.clone()"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dx_single_navigation_snapshot_serialization_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const PREPARATIONS_PER_SAMPLE: usize = 16;
        const VERTEX_COUNT: usize = 8_192;

        let prototype = navigation_change_fixture(VERTEX_COUNT);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_snapshot_preparations(
                    &prototype,
                    PREPARATIONS_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_snapshot_preparations(
                    &prototype,
                    PREPARATIONS_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_snapshot_preparations(
                    &prototype,
                    PREPARATIONS_PER_SAMPLE,
                    true,
                ));
                legacy_samples.push(measure_snapshot_preparations(
                    &prototype,
                    PREPARATIONS_PER_SAMPLE,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME432_SINGLE_NAVIGATION_SNAPSHOT_SERIALIZATION_BENCH_V1 preparations_per_sample={PREPARATIONS_PER_SAMPLE} vertex_count={VERTEX_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "single navigation snapshot serialization p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn navigation_change_fixture(vertex_count: usize) -> NavigationSnapshotChange {
        let mut before_asset = NavMeshAsset::default();
        before_asset.agent_type = "navigation-agent-before".repeat(16);
        before_asset.vertices = (0..vertex_count)
            .map(|index| [index as f32, (index % 17) as f32, (index % 31) as f32])
            .collect();
        before_asset.indices = (0..vertex_count as u32).collect();
        let mut after_asset = before_asset.clone();
        after_asset.agent_type = "navigation-agent-after".repeat(16);

        NavigationSnapshotChange {
            before: NavigationGeneratedBakeSnapshot {
                surface_entity: Some(17),
                asset: Some(before_asset),
                output_asset: Some("res://navigation/generated/before.navmesh".repeat(8)),
            },
            after: NavigationGeneratedBakeSnapshot {
                surface_entity: Some(17),
                asset: Some(after_asset),
                output_asset: Some("res://navigation/generated/after.navmesh".repeat(8)),
            },
        }
    }

    fn measure_snapshot_preparations(
        prototype: &NavigationSnapshotChange,
        preparation_count: usize,
        optimized: bool,
    ) -> u128 {
        let inputs = vec![prototype.clone(); preparation_count];
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for change in inputs {
            let values = if optimized {
                encode_prepared_snapshot_values(change).expect("optimized snapshot values")
            } else {
                legacy_prepared_snapshot_values(change)
            };
            checksum = checksum
                .wrapping_add(values.0.as_object().map_or(0, |value| value.len()))
                .wrapping_add(values.1.as_object().map_or(0, |value| value.len()));
            black_box(values);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn legacy_prepared_snapshot_values(
        change: NavigationSnapshotChange,
    ) -> (serde_json::Value, serde_json::Value) {
        let result = serde_json::to_value(NavigationGeneratedBakeChange {
            before: change.before.clone(),
            after: change.after.clone(),
            report: None,
        })
        .expect("legacy result value");
        let command = serde_json::to_value(change).expect("legacy command value");
        (command, result)
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
