use crate::core::editor_event::runtime::editor_event_runtime_inner::EditorEventRuntimeInner;
use crate::core::editor_event::EditorEventRuntime;
use crate::core::editor_extension::EditorExtensionRegistry;
use crate::ui::activity::{ActivityViewDescriptor, ActivityWindowDescriptor};
use crate::ui::control::EditorUiControlService;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::reflection::{
    activity_descriptors_from_views, apply_transient_projection, build_workbench_reflection_model,
    register_workbench_reflection_routes,
};
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewDescriptor;

impl EditorEventRuntime {
    pub(crate) fn refresh_reflection(&self) {
        let mut inner = self.inner.lock().unwrap();
        Self::refresh_reflection_locked(&mut inner);
    }

    pub(crate) fn refresh_reflection_locked(inner: &mut EditorEventRuntimeInner) {
        let descriptors = inner.manager.descriptors();
        let (views, windows) = activity_descriptors_from_views(&descriptors);
        register_activity_descriptors(&mut inner.control_service, views, windows);

        let chrome = Self::build_chrome_locked(inner, descriptors);
        let active_extensions = active_extension_registries(inner);
        let view_model = WorkbenchViewModel::build_with_extensions(&chrome, &active_extensions);
        let model = register_workbench_reflection_routes(
            &mut inner.control_service,
            build_workbench_reflection_model(&chrome, &view_model),
        );
        let mut snapshot = crate::ui::EditorUiReflectionAdapter::build_snapshot(&model);
        apply_transient_projection(&mut snapshot, &inner.transient);
        inner.control_service.publish_snapshot(snapshot);
    }

    pub(crate) fn build_chrome_locked(
        inner: &EditorEventRuntimeInner,
        descriptors: Vec<ViewDescriptor>,
    ) -> EditorChromeSnapshot {
        EditorChromeSnapshot::build(
            inner.state.snapshot(),
            &inner.manager.current_layout(),
            inner.manager.current_view_instances(),
            descriptors,
        )
    }
}

fn active_extension_registries(inner: &EditorEventRuntimeInner) -> Vec<EditorExtensionRegistry> {
    let enabled_capabilities = inner
        .manager
        .capability_snapshot()
        .enabled_capabilities()
        .to_vec();
    inner
        .editor_extensions
        .iter()
        .filter(|registration| registration.is_enabled_by(&enabled_capabilities))
        .map(|registration| registration.registry().clone())
        .collect()
}

fn register_activity_descriptors(
    service: &mut EditorUiControlService,
    views: Vec<ActivityViewDescriptor>,
    windows: Vec<ActivityWindowDescriptor>,
) {
    for descriptor in views {
        if service.activity_view(&descriptor.view_id).is_none() {
            let _ = service.register_activity_view(descriptor);
        }
    }
    for descriptor in windows {
        if service.activity_window(&descriptor.window_id).is_none() {
            let _ = service.register_activity_window(descriptor);
        }
    }
}
