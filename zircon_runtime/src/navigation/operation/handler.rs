use crate::core::framework::navigation::{
    NavMeshBakeRequest, NavigationClearBakeRequest, NavigationGeneratedBakeChange,
    NavigationGeneratedBakeSnapshot,
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
        let result = encode_change(NavigationGeneratedBakeChange {
            before: change.before.clone(),
            after: change.after.clone(),
            report: None,
        })?;
        let command = encode_snapshot_change(change)?;
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

fn encode_change(
    change: NavigationGeneratedBakeChange,
) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
    serde_json::to_value(change).map_err(|error| {
        RuntimeOperationHandlerError::new(format!(
            "encode navigation generated bake change: {error}"
        ))
    })
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
