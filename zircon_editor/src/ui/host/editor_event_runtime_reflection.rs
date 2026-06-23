use std::collections::BTreeMap;

use crate::core::editor_event::runtime::editor_event_runtime_state::EditorEventRuntimeState;
use crate::core::editor_event::EditorEventRuntime;
use crate::core::editor_event::{EditorEventEffect, ViewInstanceId};
use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};
use crate::core::editor_message::{
    EditorMessage, EditorTopic, EditorViewInvalidationMask, EditorViewRefreshReport,
};
use crate::ui::activity::{ActivityViewDescriptor, ActivityWindowDescriptor};
use crate::ui::control::EditorUiControlService;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::reflection::{
    activity_descriptors_from_views, apply_transient_projection, build_workbench_reflection_model,
    register_workbench_reflection_routes,
};
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewDescriptor;

const WORKBENCH_ROOT_VIEW_INSTANCE_ID: &str = "workbench.root";
const VIEW_INVALIDATED_TOPIC: &str = "view.invalidated";

impl EditorEventRuntime {
    pub(crate) fn refresh_reflection(&self) {
        let mut inner = self.lock_inner();
        Self::refresh_reflection_locked(&mut inner);
    }

    pub(crate) fn refresh_reflection_locked(inner: &mut EditorEventRuntimeState) {
        let descriptors = inner.manager.descriptors();
        let (views, windows) = activity_descriptors_from_views(&descriptors);
        register_activity_descriptors(&mut inner.control_service, views, windows);

        let chrome = Self::build_chrome_locked(inner, descriptors);
        let active_extensions = active_extension_registries(inner);
        let enabled_capabilities = inner
            .manager
            .capability_snapshot()
            .enabled_capabilities()
            .to_vec();
        let view_model = WorkbenchViewModel::build_with_extensions_and_capabilities(
            &chrome,
            &active_extensions,
            &enabled_capabilities,
        );
        let model = register_workbench_reflection_routes(
            &mut inner.control_service,
            build_workbench_reflection_model(&chrome, &view_model),
        );
        let mut snapshot = crate::ui::EditorUiReflectionAdapter::build_snapshot(&model);
        apply_transient_projection(&mut snapshot, &inner.transient);
        inner.control_service.publish_snapshot(snapshot);
    }

    pub fn refresh_view(
        &self,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> EditorViewRefreshReport {
        let mut inner = self.lock_inner();
        Self::refresh_view_locked(&mut inner, view, mask)
    }

    pub fn drain_pending_view_refreshes(&self) -> EditorViewRefreshReport {
        let mut inner = self.lock_inner();
        Self::drain_pending_view_refreshes_locked(&mut inner)
    }

    pub(crate) fn refresh_view_locked(
        inner: &mut EditorEventRuntimeState,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> EditorViewRefreshReport {
        let message = EditorMessage::text(view.0.clone()).with_dirty(view.clone(), mask);
        if let Ok(topic) = EditorTopic::parse(VIEW_INVALIDATED_TOPIC) {
            inner.message_bus.publish(topic, message);
        } else {
            inner.message_bus.mark_view_dirty(view, mask);
        }
        Self::drain_pending_view_refreshes_locked(inner)
    }

    pub(crate) fn refresh_workbench_locked(
        inner: &mut EditorEventRuntimeState,
        mask: EditorViewInvalidationMask,
    ) -> EditorViewRefreshReport {
        Self::refresh_view_locked(
            inner,
            ViewInstanceId::new(WORKBENCH_ROOT_VIEW_INSTANCE_ID),
            mask,
        )
    }

    pub(crate) fn refresh_workbench_for_effects_locked(
        inner: &mut EditorEventRuntimeState,
        effects: &[EditorEventEffect],
    ) -> EditorViewRefreshReport {
        Self::refresh_workbench_locked(inner, invalidation_mask_for_effects(effects))
    }

    pub(crate) fn drain_pending_view_refreshes_locked(
        inner: &mut EditorEventRuntimeState,
    ) -> EditorViewRefreshReport {
        let dirty = inner.message_bus.drain_dirty();
        let used_full_snapshot_fallback = !dirty.is_empty();
        if used_full_snapshot_fallback {
            Self::refresh_reflection_locked(inner);
        }
        EditorViewRefreshReport::new(dirty, used_full_snapshot_fallback)
    }

    pub(crate) fn build_chrome_locked(
        inner: &EditorEventRuntimeState,
        descriptors: Vec<ViewDescriptor>,
    ) -> EditorChromeSnapshot {
        let component_drawers = Self::active_component_drawers_locked(inner);
        EditorChromeSnapshot::build(
            inner
                .state
                .snapshot_with_component_drawers(&component_drawers),
            &inner.manager.current_layout(),
            inner.manager.current_view_instances(),
            descriptors,
        )
    }

    pub(crate) fn active_component_drawers_locked(
        inner: &EditorEventRuntimeState,
    ) -> BTreeMap<String, ComponentDrawerDescriptor> {
        active_extension_registries(inner)
            .into_iter()
            .flat_map(|registry| {
                registry
                    .component_drawers()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .map(|descriptor| (descriptor.component_type().to_string(), descriptor))
            .collect()
    }
}

fn invalidation_mask_for_effects(effects: &[EditorEventEffect]) -> EditorViewInvalidationMask {
    let mut mask = EditorViewInvalidationMask::NONE;
    for effect in effects {
        match effect {
            EditorEventEffect::LayoutChanged => mask.insert(
                EditorViewInvalidationMask::LAYOUT
                    .union(EditorViewInvalidationMask::PRESENTATION_DATA),
            ),
            EditorEventEffect::RenderChanged => mask.insert(
                EditorViewInvalidationMask::RENDER
                    .union(EditorViewInvalidationMask::PRESENTATION_DATA),
            ),
            EditorEventEffect::PresentationChanged | EditorEventEffect::ReflectionChanged => {
                mask.insert(EditorViewInvalidationMask::PRESENTATION_DATA);
            }
            EditorEventEffect::PresentWelcomeRequested
            | EditorEventEffect::ProjectOpenRequested
            | EditorEventEffect::ProjectSaveRequested
            | EditorEventEffect::AssetDetailsRefreshRequested
            | EditorEventEffect::AssetPreviewRefreshRequested
            | EditorEventEffect::ImportModelRequested
            | EditorEventEffect::CommandPaletteOpenRequested => {
                mask.insert(EditorViewInvalidationMask::PRESENTATION_DATA);
            }
        }
    }
    if mask.is_empty() {
        EditorViewInvalidationMask::PRESENTATION_DATA
    } else {
        mask
    }
}

fn active_extension_registries(inner: &EditorEventRuntimeState) -> Vec<EditorExtensionRegistry> {
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
