use std::any::Any;
use std::sync::Arc;

use crate::core::framework::navigation::{
    NavMeshBakeRequest, NavigationClearBakeRequest, NavigationGeneratedBakeChange,
    NavigationGeneratedBakeSnapshot,
};
use crate::operation::{
    RuntimeOperationApply, RuntimeOperationContext, RuntimeOperationHandler,
    RuntimeOperationHandlerError, RuntimeOperationPrepared, RuntimeOperationSnapshot,
};
use crate::scene::{
    SceneNavigationRuntime, SceneNavigationRuntimeHandle, WorldPublicationError,
    WorldPublicationSource, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME,
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

struct NavigationSnapshotOwnerState {
    source: WorldPublicationSource,
    runtime: Arc<SceneNavigationRuntimeHandle>,
    before: NavigationGeneratedBakeSnapshot,
    after: NavigationGeneratedBakeSnapshot,
    generated_mutation_epoch: u64,
}

struct NavigationBakeOwnerState {
    source: WorldPublicationSource,
    runtime: Arc<SceneNavigationRuntimeHandle>,
    before: NavigationGeneratedBakeSnapshot,
    backend_snapshot: Box<dyn Any + Send>,
}

struct NavigationBakePreparedState {
    source: WorldPublicationSource,
    runtime: Arc<SceneNavigationRuntimeHandle>,
    before: NavigationGeneratedBakeSnapshot,
    backend_prepared: Box<dyn Any + Send>,
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

    fn operation_request(
        &self,
        payload: serde_json::Value,
    ) -> Result<NavMeshBakeRequest, RuntimeOperationHandlerError> {
        let mut request: NavMeshBakeRequest = decode_payload(
            payload,
            match self.kind {
                NavigationOperationKind::BakeScene => "navigation scene bake",
                NavigationOperationKind::BakeSurface => "navigation surface bake",
                _ => "navigation bake",
            },
        )?;
        if matches!(self.kind, NavigationOperationKind::BakeScene) {
            request.surface_entity = None;
        }
        if matches!(self.kind, NavigationOperationKind::BakeSurface)
            && request.surface_entity.is_none()
        {
            return Err(RuntimeOperationHandlerError::new(
                "navigation surface bake requires surface_entity",
            ));
        }
        Ok(request)
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
        Self::reject_null_restore_target(snapshot.surface_entity)?;
        let runtime = Self::resolve_runtime(&context)?;
        let before = runtime.generated_bake_snapshot(snapshot.surface_entity);
        Self::reject_noncanonical_restore_target(snapshot.surface_entity, &before)?;
        encode_snapshot_change(NavigationSnapshotChange {
            before,
            after: snapshot,
        })
    }

    fn reject_noncanonical_restore_target(
        requested_surface: Option<u64>,
        before: &NavigationGeneratedBakeSnapshot,
    ) -> Result<(), RuntimeOperationHandlerError> {
        if requested_surface.is_none() {
            return Err(RuntimeOperationHandlerError::new(
                "navigation snapshot restore requires a non-null surface_entity",
            ));
        }
        if before.surface_entity != requested_surface {
            return Err(RuntimeOperationHandlerError::new(
                "navigation snapshot restore target is not canonical",
            ));
        }
        Ok(())
    }

    fn reject_null_restore_target(
        requested_surface: Option<u64>,
    ) -> Result<(), RuntimeOperationHandlerError> {
        if requested_surface.is_none() {
            return Err(RuntimeOperationHandlerError::new(
                "navigation snapshot restore requires a non-null surface_entity",
            ));
        }
        Ok(())
    }

    fn snapshot_change_owner(
        context: &RuntimeOperationContext<'_>,
        request: NavigationClearBakeRequest,
        restore: Option<NavigationGeneratedBakeSnapshot>,
    ) -> Result<RuntimeOperationSnapshot, RuntimeOperationHandlerError> {
        if let Some(snapshot) = restore.as_ref() {
            Self::reject_null_restore_target(snapshot.surface_entity)?;
        }
        let runtime = Self::resolve_runtime(context)?;
        let source = context.level().capture();
        let before = runtime.generated_bake_snapshot(
            restore
                .as_ref()
                .map(|snapshot| snapshot.surface_entity)
                .unwrap_or(request.surface_entity),
        );
        if let Some(snapshot) = restore.as_ref() {
            Self::reject_noncanonical_restore_target(snapshot.surface_entity, &before)?;
        }
        let generated_mutation_epoch = runtime.generated_bake_mutation_epoch(before.surface_entity);
        let after = restore.unwrap_or_else(|| {
            NavigationGeneratedBakeSnapshot::empty(before.surface_entity.or(request.surface_entity))
        });
        let payload = encode_snapshot_change(NavigationSnapshotChange {
            before: before.clone(),
            after: after.clone(),
        })?;
        let owner_bytes = source_retained_bytes(&source)
            .saturating_add(estimate_snapshot_bytes(&before))
            .saturating_add(estimate_snapshot_bytes(&after));
        Ok(RuntimeOperationSnapshot::with_owner_state(
            payload,
            Box::new(NavigationSnapshotOwnerState {
                source,
                runtime,
                before,
                after,
                generated_mutation_epoch,
            }),
            owner_bytes,
        ))
    }

    fn prepare_snapshot_owner(
        owner: NavigationSnapshotOwnerState,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        let (command, result) = encode_prepared_snapshot_values(NavigationSnapshotChange {
            before: owner.before.clone(),
            after: owner.after.clone(),
        })?;
        let owner_bytes = source_retained_bytes(&owner.source)
            .saturating_add(estimate_snapshot_bytes(&owner.before))
            .saturating_add(estimate_snapshot_bytes(&owner.after));
        Ok(RuntimeOperationPrepared::with_owner_state(
            command,
            result,
            Box::new(owner),
            owner_bytes,
        ))
    }

    fn prepare_snapshot_change(
        snapshot: serde_json::Value,
        operation: &str,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        let change: NavigationSnapshotChange = decode_payload(snapshot, operation)?;
        let (command, result) = encode_prepared_snapshot_values(change)?;
        Ok(RuntimeOperationPrepared::new(command, result))
    }

    fn apply_snapshot_owner(
        owner: NavigationSnapshotOwnerState,
    ) -> Result<(), RuntimeOperationHandlerError> {
        let NavigationSnapshotOwnerState {
            source,
            runtime,
            before,
            after,
            generated_mutation_epoch,
        } = owner;
        source
            .publish(|_| {
                if runtime.generated_bake_snapshot(before.surface_entity) != before {
                    return Err(RuntimeOperationHandlerError::new(
                        "navigation generated bake state changed before owner apply",
                    ));
                }
                if runtime.generated_bake_mutation_epoch(before.surface_entity)
                    != generated_mutation_epoch
                {
                    return Err(RuntimeOperationHandlerError::new(
                        "navigation generated bake mutation epoch changed before owner apply",
                    ));
                }
                runtime
                    .replace_generated_bake_snapshot(after)
                    .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))
            })
            .map_err(|error: WorldPublicationError| {
                RuntimeOperationHandlerError::new(error.to_string())
            })?
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

    fn snapshot_owned(
        &self,
        context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<RuntimeOperationSnapshot, RuntimeOperationHandlerError> {
        match self.kind {
            NavigationOperationKind::BakeScene | NavigationOperationKind::BakeSurface => {
                let request = self.operation_request(payload)?;
                let runtime = Self::resolve_runtime(&context)?;
                let source = context.level().capture();
                let (backend_snapshot, owner_bytes, before) = runtime
                    .capture_bake_operation(&source, request.clone())
                    .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))?;
                let source_bytes = source_retained_bytes(&source);
                Ok(RuntimeOperationSnapshot::with_owner_state(
                    encode_payload(request, "navigation bake")?,
                    Box::new(NavigationBakeOwnerState {
                        source,
                        runtime,
                        before,
                        backend_snapshot,
                    }),
                    owner_bytes.saturating_add(source_bytes),
                ))
            }
            NavigationOperationKind::ClearSurface => Self::snapshot_change_owner(
                &context,
                decode_payload(payload, "navigation surface clear")?,
                None,
            ),
            NavigationOperationKind::RestoreSnapshot => Self::snapshot_change_owner(
                &context,
                NavigationClearBakeRequest::default(),
                Some(decode_payload(payload, "navigation bake snapshot restore")?),
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

    fn prepare_owned(
        &self,
        snapshot: RuntimeOperationSnapshot,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        let (_, owner, _) = snapshot.into_parts();
        let owner = owner.ok_or_else(|| {
            RuntimeOperationHandlerError::new("navigation operation lost its owner snapshot")
        })?;
        match self.kind {
            NavigationOperationKind::BakeScene | NavigationOperationKind::BakeSurface => {
                let owner = owner.downcast::<NavigationBakeOwnerState>().map_err(|_| {
                    RuntimeOperationHandlerError::new(
                        "navigation bake owner snapshot type mismatch",
                    )
                })?;
                let NavigationBakeOwnerState {
                    source,
                    runtime,
                    before,
                    backend_snapshot,
                } = *owner;
                let (backend_prepared, report, after, owner_bytes) = runtime
                    .prepare_bake_operation(backend_snapshot)
                    .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))?;
                let (command, _) = encode_prepared_snapshot_values(NavigationSnapshotChange {
                    before: before.clone(),
                    after: after.clone(),
                })?;
                let result = encode_payload(
                    NavigationGeneratedBakeChange {
                        before: before.clone(),
                        after: after.clone(),
                        report: Some(report),
                    },
                    "navigation bake result",
                )?;
                let owner_bytes = owner_bytes
                    .saturating_add(source_retained_bytes(&source))
                    .saturating_add(estimate_snapshot_bytes(&before))
                    .saturating_add(estimate_snapshot_bytes(&after));
                Ok(RuntimeOperationPrepared::with_owner_state(
                    command,
                    result,
                    Box::new(NavigationBakePreparedState {
                        source,
                        runtime,
                        before,
                        backend_prepared,
                    }),
                    owner_bytes,
                ))
            }
            NavigationOperationKind::ClearSurface | NavigationOperationKind::RestoreSnapshot => {
                let owner = owner
                    .downcast::<NavigationSnapshotOwnerState>()
                    .map_err(|_| {
                        RuntimeOperationHandlerError::new("navigation snapshot owner type mismatch")
                    })?;
                Self::prepare_snapshot_owner(*owner)
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
            NavigationOperationKind::ClearSurface | NavigationOperationKind::RestoreSnapshot => {
                let _ = (context, command);
                Err(RuntimeOperationHandlerError::new(
                    "navigation snapshot requires an owner publication source",
                ))
            }
        }
    }

    fn apply_owned(
        &self,
        _context: RuntimeOperationContext<'_>,
        apply: RuntimeOperationApply,
    ) -> Result<(), RuntimeOperationHandlerError> {
        let (_, owner) = apply.into_parts();
        let owner = owner.ok_or_else(|| {
            RuntimeOperationHandlerError::new("navigation operation lost its prepared owner state")
        })?;
        match self.kind {
            NavigationOperationKind::BakeScene | NavigationOperationKind::BakeSurface => {
                let owner = owner
                    .downcast::<NavigationBakePreparedState>()
                    .map_err(|_| {
                        RuntimeOperationHandlerError::new(
                            "navigation prepared bake owner type mismatch",
                        )
                    })?;
                let NavigationBakePreparedState {
                    source,
                    runtime,
                    before,
                    backend_prepared,
                } = *owner;
                runtime
                    .apply_bake_operation(&source, &before, backend_prepared)
                    .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))
            }
            NavigationOperationKind::ClearSurface | NavigationOperationKind::RestoreSnapshot => {
                let owner = owner
                    .downcast::<NavigationSnapshotOwnerState>()
                    .map_err(|_| {
                        RuntimeOperationHandlerError::new(
                            "navigation prepared snapshot owner type mismatch",
                        )
                    })?;
                Self::apply_snapshot_owner(*owner)
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

fn estimate_snapshot_bytes(snapshot: &NavigationGeneratedBakeSnapshot) -> usize {
    serde_json::to_vec(snapshot).map_or(0, |encoded| encoded.len())
}

fn source_retained_bytes(source: &WorldPublicationSource) -> usize {
    source.snapshot().node_records().len().saturating_mul(256)
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
#[path = "tests/handler.rs"]
mod tests;
