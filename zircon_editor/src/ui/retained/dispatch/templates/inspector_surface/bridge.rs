use std::collections::HashMap;

use zircon_runtime_interface::ui::binding::UiEventKind;

use crate::ui::binding::EditorUiBinding;
use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID;
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiHostProjection};

#[cfg(test)]
use super::super::project_builtin_surface;
use super::super::{binding_for_control, project_builtin_surface_with_runtime};
use super::error::BuiltinInspectorSurfaceTemplateBridgeError;

// 检查器控件的事件绑定取自已投影的模板，编辑操作由外层分发器解释。
pub(crate) struct BuiltinInspectorSurfaceTemplateBridge {
    bindings_by_id: HashMap<String, EditorUiBinding>,
    host_projection: RetainedUiHostProjection,
}

impl BuiltinInspectorSurfaceTemplateBridge {
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, BuiltinInspectorSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface(BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID)?;
        Ok(Self {
            bindings_by_id,
            host_projection,
        })
    }

    pub(crate) fn new_with_runtime(
        runtime: &EditorUiHostRuntime,
    ) -> Result<Self, BuiltinInspectorSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface_with_runtime(runtime, BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID)?;
        Ok(Self {
            bindings_by_id,
            host_projection,
        })
    }

    pub(crate) fn binding_for_control(
        &self,
        control_id: &str,
        event_kind: UiEventKind,
    ) -> Option<&EditorUiBinding> {
        binding_for_control(
            &self.bindings_by_id,
            &self.host_projection,
            control_id,
            event_kind,
        )
    }
}
