use serde_json::json;
use crate::ui::{
    ActivityViewDescriptor, ActivityWindowDescriptor, EditorUiControlService,
    EditorUiReflectionAdapter,
};
use zircon_ui::{UiNodeDescriptor, UiPropertyDescriptor, UiReflectionSnapshot, UiValueType};

use crate::core::editor_event::EditorTransientUiState;
use crate::{
    activity_descriptors_from_views, build_workbench_reflection_model,
    register_workbench_reflection_routes, EditorChromeSnapshot, ViewDescriptor, WorkbenchViewModel,
};

use super::editor_event_runtime::EditorEventRuntime;
use super::runtime_inner::EditorEventRuntimeInner;

impl EditorEventRuntime {
    pub(super) fn refresh_reflection(&self) {
        let mut inner = self.inner.lock().unwrap();
        Self::refresh_reflection_locked(&mut inner);
    }

    pub(super) fn refresh_reflection_locked(inner: &mut EditorEventRuntimeInner) {
        let descriptors = inner.manager.descriptors();
        let (views, windows) = activity_descriptors_from_views(&descriptors);
        register_activity_descriptors(&mut inner.control_service, views, windows);

        let chrome = Self::build_chrome_locked(inner, descriptors);
        let view_model = WorkbenchViewModel::build(&chrome);
        let model = register_workbench_reflection_routes(
            &mut inner.control_service,
            build_workbench_reflection_model(&chrome, &view_model),
        );
        let mut snapshot = EditorUiReflectionAdapter::build_snapshot(&model);
        apply_transient_projection(&mut snapshot, &inner.transient);
        inner.control_service.publish_snapshot(snapshot);
    }

    pub(super) fn build_chrome_locked(
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

fn apply_transient_projection(
    snapshot: &mut UiReflectionSnapshot,
    transient: &EditorTransientUiState,
) {
    for node in snapshot.nodes.values_mut() {
        let node_path = node.node_path.0.clone();
        let hovered = transient.is_node_hovered(&node_path);
        let focused = transient.is_node_focused(&node_path);
        let pressed = transient.is_node_pressed(&node_path);
        let resizing = drawer_id_from_path(&node_path)
            .is_some_and(|drawer_id| transient.is_drawer_resizing(drawer_id));
        let dragging = node_path
            .rsplit('/')
            .next()
            .is_some_and(|segment| transient.is_view_dragging(segment));

        node.state_flags.pressed = pressed;
        upsert_property(node, "transient.hovered", hovered);
        upsert_property(node, "transient.focused", focused);
        upsert_property(node, "transient.resizing", resizing);
        upsert_property(node, "transient.dragging", dragging);
    }
}

fn upsert_property(node: &mut UiNodeDescriptor, name: &str, value: bool) {
    node.properties.insert(
        name.to_string(),
        UiPropertyDescriptor::new(name, UiValueType::Bool, json!(value)),
    );
}

fn drawer_id_from_path(node_path: &str) -> Option<&str> {
    let prefix = "editor/workbench/drawers/";
    let remainder = node_path.strip_prefix(prefix)?;
    remainder.split('/').next()
}
