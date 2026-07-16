use crate::core::framework::navigation::{
    NavMeshBakeRequest, NavigationClearBakeRequest, NavigationGeneratedBakeChange,
    NavigationGeneratedBakeSnapshot,
};
use crate::operation::{
    RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationHandlerError,
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

    fn bake(
        context: RuntimeOperationContext<'_>,
        mut request: NavMeshBakeRequest,
        selected_surface_required: bool,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        if selected_surface_required && request.surface_entity.is_none() {
            return Err(RuntimeOperationHandlerError::new(
                "navigation surface bake requires surface_entity",
            ));
        }
        if !selected_surface_required {
            request.surface_entity = None;
        }
        let runtime = Self::resolve_runtime(&context)?;
        let requested_surface = request.surface_entity;
        let mut before = runtime.generated_bake_snapshot(requested_surface);
        let report = runtime
            .bake_surface(context.world(), request)
            .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))?;
        let after = runtime.generated_bake_snapshot(requested_surface);
        if before.asset.is_none() && before.surface_entity != after.surface_entity {
            before.surface_entity = after.surface_entity;
        }
        encode_change(NavigationGeneratedBakeChange {
            before,
            after,
            report: Some(report),
        })
    }

    fn clear(
        context: RuntimeOperationContext<'_>,
        request: NavigationClearBakeRequest,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        let runtime = Self::resolve_runtime(&context)?;
        let before = runtime.generated_bake_snapshot(request.surface_entity);
        let target = before.surface_entity.or(request.surface_entity);
        runtime
            .replace_generated_bake_snapshot(NavigationGeneratedBakeSnapshot::empty(target))
            .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))?;
        let after = runtime.generated_bake_snapshot(target);
        encode_change(NavigationGeneratedBakeChange {
            before,
            after,
            report: None,
        })
    }

    fn restore(
        context: RuntimeOperationContext<'_>,
        snapshot: NavigationGeneratedBakeSnapshot,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        let runtime = Self::resolve_runtime(&context)?;
        let before = runtime.generated_bake_snapshot(snapshot.surface_entity);
        let target = snapshot.surface_entity;
        runtime
            .replace_generated_bake_snapshot(snapshot)
            .map_err(|error| RuntimeOperationHandlerError::new(error.to_string()))?;
        let after = runtime.generated_bake_snapshot(target);
        encode_change(NavigationGeneratedBakeChange {
            before,
            after,
            report: None,
        })
    }
}

impl RuntimeOperationHandler for NavigationOperationHandler {
    fn execute(
        &self,
        context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        match self.kind {
            NavigationOperationKind::BakeScene => Self::bake(
                context,
                decode_payload(payload, "navigation scene bake")?,
                false,
            ),
            NavigationOperationKind::BakeSurface => Self::bake(
                context,
                decode_payload(payload, "navigation surface bake")?,
                true,
            ),
            NavigationOperationKind::ClearSurface => Self::clear(
                context,
                decode_payload(payload, "navigation surface clear")?,
            ),
            NavigationOperationKind::RestoreSnapshot => Self::restore(
                context,
                decode_payload(payload, "navigation bake snapshot restore")?,
            ),
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
