use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::editor_extension::{
    EditorUiTemplateDescriptor, EditorUiTemplatePaneDataSnapshot, EditorUiTemplatePaneDataSource,
};
use crate::core::extension::{ContributionBatch, ContributionSource, ContributionTicket};
use crate::ui::host::{EditorHostEventController, EditorManager};
use crate::ui::workbench::state::EditorState;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace, PaneTemplateSpec,
    ViewDescriptor, ViewDescriptorId, ViewInstanceId, ViewKind,
};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::component::UiValue;

const TEMPLATE_ID: &str = "editor.test.targeted_pane";
const VIEW_ID: &str = "editor.test.targeted_pane_view";

struct SnapshotSource<F>(F);

impl<F> EditorUiTemplatePaneDataSource for SnapshotSource<F>
where
    F: Fn() -> EditorUiTemplatePaneDataSnapshot + Send + Sync,
{
    fn snapshot(&self) -> EditorUiTemplatePaneDataSnapshot {
        (self.0)()
    }
}

fn controller() -> (CoreRuntime, Arc<EditorHostEventController>) {
    let core = CoreRuntime::new();
    let manager = Arc::new(EditorManager::new(&core.handle()).unwrap());
    let state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 800),
        Arc::clone(manager.context()),
    );
    let controller = EditorHostEventController::new(
        state,
        crate::tests::support::register_test_editor_manager(&core, manager),
    )
    .unwrap();
    (core, Arc::new(controller))
}

fn open_view(controller: &EditorHostEventController, capabilities: &[&str]) -> ViewInstanceId {
    let manager = controller.enter_editor_manager().unwrap();
    let mut registry = manager.host.lock_view_registry();
    registry.set_available_capabilities(capabilities.iter().copied());
    registry
        .register_view(
            ViewDescriptor::new(
                ViewDescriptorId::new(VIEW_ID),
                ViewKind::ActivityView,
                "Pane",
            )
            .with_required_capabilities(capabilities.iter().copied())
            .with_pane_template(PaneTemplateSpec::new(PaneBodySpec::new(
                TEMPLATE_ID,
                PanePayloadKind::TemplateV2,
                PaneRouteNamespace::Template,
                PaneInteractionMode::TemplateOnly,
            ))),
        )
        .unwrap();
    registry
        .open_descriptor(ViewDescriptorId::new(VIEW_ID))
        .unwrap()
        .instance_id
}

fn register_source(
    controller: &EditorHostEventController,
    callback: impl Fn() -> EditorUiTemplatePaneDataSnapshot + Send + Sync + 'static,
) -> ContributionTicket {
    let mut batch = ContributionBatch::default();
    batch
        .register_ui_template(EditorUiTemplateDescriptor::new(
            TEMPLATE_ID,
            "res://ui/editor/test-targeted-pane.zui",
        ))
        .unwrap();
    batch
        .register_ui_template_pane_data_source(TEMPLATE_ID, Arc::new(SnapshotSource(callback)))
        .unwrap();
    controller
        .shell()
        .lock()
        .contributions
        .contribute(ContributionSource::Builtin, batch)
        .unwrap()
}

fn data() -> EditorUiTemplatePaneDataSnapshot {
    EditorUiTemplatePaneDataSnapshot::new(std::collections::BTreeMap::from([(
        "source".to_string(),
        UiValue::String("current source".to_string()),
    )]))
}

#[test]
fn targeted_template_pane_requires_a_live_view_bound_to_the_requested_template() {
    let (_core, controller) = controller();
    let view = open_view(&controller, &[]);
    let calls = Arc::new(AtomicUsize::new(0));
    let source_calls = Arc::clone(&calls);
    register_source(&controller, move || {
        source_calls.fetch_add(1, Ordering::Relaxed);
        data()
    });

    for (template, instance) in [
        ("editor.test.other_template", view.clone()),
        (TEMPLATE_ID, ViewInstanceId::new("missing-view")),
    ] {
        assert_eq!(
            controller
                .ui_template_pane_data_snapshot(template, &instance)
                .unwrap(),
            None
        );
    }
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        controller
            .ui_template_pane_data_snapshot(TEMPLATE_ID, &view)
            .unwrap(),
        Some(data())
    );
    controller
        .enter_editor_manager()
        .unwrap()
        .host
        .lock_view_registry()
        .remove_instance(&view)
        .unwrap();
    assert_eq!(
        controller
            .ui_template_pane_data_snapshot(TEMPLATE_ID, &view)
            .unwrap(),
        None
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[test]
fn targeted_template_pane_does_not_call_a_source_after_view_capability_is_disabled() {
    let (_core, controller) = controller();
    let view = open_view(&controller, &["editor.test.pane_capability"]);
    register_source(&controller, || {
        panic!("disabled view must not query its source")
    });
    controller
        .enter_editor_manager()
        .unwrap()
        .host
        .lock_view_registry()
        .set_available_capabilities(std::iter::empty::<&str>());

    assert_eq!(
        controller
            .ui_template_pane_data_snapshot(TEMPLATE_ID, &view)
            .unwrap(),
        None
    );
}

#[test]
fn targeted_template_pane_discards_data_when_the_callback_closes_its_view() {
    let (_core, controller) = controller();
    let view = open_view(&controller, &[]);
    let callback_view = view.clone();
    let weak_controller = Arc::downgrade(&controller);
    register_source(&controller, move || {
        weak_controller
            .upgrade()
            .unwrap()
            .enter_editor_manager()
            .unwrap()
            .host
            .lock_view_registry()
            .remove_instance(&callback_view)
            .unwrap();
        data()
    });

    assert_eq!(
        controller
            .ui_template_pane_data_snapshot(TEMPLATE_ID, &view)
            .unwrap(),
        None
    );
}

#[test]
fn targeted_template_pane_discards_data_when_the_callback_revokes_its_source() {
    let (_core, controller) = controller();
    let view = open_view(&controller, &[]);
    let ticket = Arc::new(Mutex::new(None));
    let callback_ticket = Arc::clone(&ticket);
    let weak_controller = Arc::downgrade(&controller);
    let registered = register_source(&controller, move || {
        let controller = weak_controller.upgrade().unwrap();
        let ticket = callback_ticket.lock().unwrap().take().unwrap();
        assert!(controller
            .shell()
            .lock()
            .contributions
            .revoke(ticket)
            .revoked());
        data()
    });
    *ticket.lock().unwrap() = Some(registered);

    assert_eq!(
        controller
            .ui_template_pane_data_snapshot(TEMPLATE_ID, &view)
            .unwrap(),
        None
    );
}
