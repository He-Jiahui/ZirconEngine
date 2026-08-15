use crate::core::CoreError;
use crate::scene::EntityId;
use crate::script::VmError;

use super::SCRIPT_BINDINGS_COMPONENT;

// Sibling gameplay hosts propagate this result through scene-system helpers.
pub(in crate::script::vm) type ScriptSceneSystemResult<T> =
    std::result::Result<T, ScriptSceneSystemError>;

#[derive(Debug, thiserror::Error)]
pub(in crate::script::vm) enum ScriptSceneSystemError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("invalid {component} for entity {entity}: {source}")]
    InvalidBindingComponent {
        component: &'static str,
        entity: EntityId,
        #[source]
        source: serde_json::Error,
    },
    #[error("script binding {binding}.{export_name} failed: {source}")]
    ExportCall {
        binding: String,
        export_name: &'static str,
        #[source]
        source: VmError,
    },
}

impl ScriptSceneSystemError {
    pub(super) fn invalid_binding_component(entity: EntityId, source: serde_json::Error) -> Self {
        Self::InvalidBindingComponent {
            component: SCRIPT_BINDINGS_COMPONENT,
            entity,
            source,
        }
    }

    pub(super) fn export_call(binding: String, export_name: &'static str, source: VmError) -> Self {
        Self::ExportCall {
            binding,
            export_name,
            source,
        }
    }
}
