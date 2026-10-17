use super::support::*;
use crate::ui::layouts::windows::workbench_host_window::{
    ModulePluginDetailsViewData, ModulePluginsPaneViewData,
};
use zircon_runtime::asset::{
    project::{ProjectManifest, ProjectPaths},
    AssetUri,
};
use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiPointerComponentEvent, UiPointerComponentEventReason, UiTemplateActionInvocation,
    },
    event_ui::{UiNodeId, UiTreeId},
};

struct ProjectFixture(PathBuf);
impl Drop for ProjectFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture(
    prefix: &str,
) -> (
    ProjectFixture,
    ChildWindowHostHarness,
    ViewInstanceId,
    UiHostWindow,
) {
    // Declared before the host, so failed assertions also drop every host owner
    // before this project is removed. No native library is needed for desired state.
    let project = ProjectFixture(unique_temp_path(prefix));
    let paths = ProjectPaths::from_root(&project.0).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    let mut manifest = ProjectManifest::new(
        "Feature Details",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.save(paths.manifest_path()).unwrap();
    for owner in ["vendor.first", "vendor.second"] {
        let dir = project.0.join("zircon_plugins").join(owner);
        fs::create_dir_all(&dir).unwrap();
        let crate_id = owner.replace('.', "_");
        let mut source = format!(
            r#"
id = "{owner}"
version = "0.1.0"
display_name = "{owner}"
default_packaging = ["native_dynamic"]
[[modules]]
name = "{owner}.runtime"
kind = "runtime"
crate_name = "zircon_plugin_{crate_id}_runtime"
target_modes = ["editor_host"]
capabilities = ["runtime.plugin.{owner}"]
"#
        );
        for index in 0..10 {
            source.push_str(&format!(
                r#"
[[optional_features]]
id = "shared.feature.{index}"
display_name = "Feature {index}"
owner_plugin_id = "{owner}"
capabilities = ["runtime.feature.{owner}.{index}"]
default_packaging = ["native_dynamic"]
enabled_by_default = false
[[optional_features.dependencies]]
plugin_id = "{owner}"
capability = "runtime.plugin.{owner}"
primary = true
[[optional_features.modules]]
name = "{owner}.feature{index}.runtime"
kind = "runtime"
crate_name = "zircon_plugin_{crate_id}_feature{index}_runtime"
target_modes = ["editor_host"]
capabilities = ["runtime.feature.{owner}.{index}"]
"#
            ));
        }
        fs::write(dir.join("plugin.toml"), source).unwrap();
    }
    let harness = ChildWindowHostHarness::new(prefix);
    {
        let mut host = harness.host.borrow_mut();
        host.asset_runtime_access
            .asset_manager()
            .unwrap()
            .open_project(project.0.to_string_lossy().as_ref())
            .unwrap();
        let manager = host.editor_manager.enter().unwrap();
        for owner in ["vendor.first", "vendor.second"] {
            manager
                .set_native_aware_project_plugin_enabled(&project.0, &mut manifest, owner, true)
                .unwrap();
            for index in 0..10 {
                manager
                    .set_native_aware_project_plugin_feature_enabled(
                        &project.0,
                        &mut manifest,
                        owner,
                        &format!("shared.feature.{index}"),
                        false,
                    )
                    .unwrap();
            }
        }
        manifest.save(paths.manifest_path()).unwrap();
        drop(manager);
        host.runtime
            .shell()
            .lock()
            .state
            .set_project_path(project.0.to_string_lossy().into_owned());
        host.runtime.refresh_reflection();
    }
    harness.activate_workbench_page();
    let pane = harness.open_view("editor.module_plugins");
    let child = harness.detach_view_to_child_window(&pane.0, "window:plugin-details");
    (project, harness, pane, child)
}

fn refresh(harness: &ChildWindowHostHarness) {
    let mut host = harness.host.borrow_mut();
    host.mark_presentation_dirty();
    host.refresh_ui();
    host.recompute_if_dirty();
}

fn pane_data(harness: &ChildWindowHostHarness) -> ModulePluginsPaneViewData {
    let host = harness.host.borrow();
    let report = host
        .editor_manager
        .enter()
        .unwrap()
        .published_plugin_status_report();
    let project = host.runtime.editor_snapshot().unwrap().project_path;
    let mut pane = ModulePluginsPaneViewData::default();
    host.project_module_plugin_details(&project, &report, &mut pane);
    pane
}
fn details(harness: &ChildWindowHostHarness) -> Rc<ModulePluginDetailsViewData> {
    pane_data(harness)
        .details
        .expect("the selected package must expose its feature page")
}
fn open_owner(harness: &ChildWindowHostHarness, child: &UiHostWindow, owner: &str) {
    pane_surface_host(child)
        .invoke_surface_control_clicked("ModulePluginSelect".into(), owner.into());
    refresh(harness);
}
fn select(
    harness: &ChildWindowHostHarness,
    child: &UiHostWindow,
    pane: &ViewInstanceId,
    index: i32,
    id: &str,
) {
    pane_surface_host(child).invoke_template_table_row_selected(
        pane.0.as_str().into(),
        "ModulePluginFeatureRows".into(),
        index,
        "string".into(),
        id.into(),
    );
    refresh(harness);
}
fn pointer_apply(harness: &ChildWindowHostHarness, child: &UiHostWindow, intent: &str) {
    let event = UiPointerComponentEvent::new(
        &UiTreeId::new("res://ui/editor/host/module_plugins_body.zui"),
        UiNodeId::new(7),
        "ModulePluginDetailsApply",
        "ModulePluginDetailsApply/Click",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".into(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    )
    .with_template_action(UiTemplateActionInvocation::route(
        "workbench.plugin.details.apply",
        std::collections::BTreeMap::from([("intent".into(), UiValue::String(intent.into()))]),
    ));
    pane_surface_host(child).invoke_pointer_component_event(event);
    refresh(harness);
}
fn enabled_on_disk(project: &ProjectFixture, owner: &str, feature: &str) -> bool {
    let paths = ProjectPaths::from_root(&project.0).unwrap();
    ProjectManifest::load(paths.manifest_path())
        .unwrap()
        .plugins
        .selections
        .iter()
        .find(|selection| selection.id == owner)
        .unwrap()
        .features
        .iter()
        .find(|selection| selection.id == feature)
        .unwrap()
        .enabled
}
fn key(
    harness: &ChildWindowHostHarness,
    child: &UiHostWindow,
    named: NamedKey,
    code: KeyCode,
    modifiers: ModifiersState,
) {
    child.dispatch_native_key_for_test(
        key_event(
            Key::Named(named),
            PhysicalKey::Code(code),
            None,
            ElementState::Pressed,
        ),
        modifiers,
    );
    refresh(harness);
}

#[test]
fn feature_details_apply_only_the_selected_feature_of_the_qualified_owner() {
    let _guard = lock_env();
    let (project, harness, pane, child) = fixture("zircon_feature_details_owner");
    open_owner(&harness, &child, "vendor.first");
    assert_eq!(details(&harness).rows.len(), 8);
    select(&harness, &child, &pane, 1, "shared.feature.1");
    let selected = details(&harness);
    assert_eq!(selected.selected_feature_id, "shared.feature.1");
    assert_eq!(selected.plugin_id, "vendor.first");
    assert!(!selected.intent_id.is_empty());
    pointer_apply(&harness, &child, &selected.intent_id);
    assert!(enabled_on_disk(
        &project,
        "vendor.first",
        "shared.feature.1"
    ));
    assert!(!enabled_on_disk(
        &project,
        "vendor.first",
        "shared.feature.0"
    ));
    assert!(!enabled_on_disk(
        &project,
        "vendor.second",
        "shared.feature.1"
    ));
    // A second delivery must not retoggle the now-enabled feature.
    pointer_apply(&harness, &child, &selected.intent_id);
    assert!(enabled_on_disk(
        &project,
        "vendor.first",
        "shared.feature.1"
    ));
    harness
        .host
        .borrow()
        .asset_runtime_access
        .asset_manager()
        .unwrap()
        .close_project()
        .unwrap();
}

#[test]
fn feature_details_keyboard_pages_and_back_restore_stable_selection() {
    let _guard = lock_env();
    let (project, harness, pane, child) = fixture("zircon_feature_details_keyboard");
    open_owner(&harness, &child, "vendor.second");
    key(
        &harness,
        &child,
        NamedKey::ArrowDown,
        KeyCode::ArrowDown,
        ModifiersState::empty(),
    );
    assert_eq!(details(&harness).selected_feature_id, "shared.feature.0");
    key(
        &harness,
        &child,
        NamedKey::Tab,
        KeyCode::Tab,
        ModifiersState::empty(),
    );
    assert_eq!(pane_data(&harness).focused_control.as_str(), "next");
    key(
        &harness,
        &child,
        NamedKey::Enter,
        KeyCode::Enter,
        ModifiersState::empty(),
    );
    assert_eq!(details(&harness).page, 1);
    assert!(details(&harness).selected_feature_id.is_empty());
    assert_eq!(pane_data(&harness).focused_control.as_str(), "features");
    key(
        &harness,
        &child,
        NamedKey::ArrowDown,
        KeyCode::ArrowDown,
        ModifiersState::empty(),
    );
    assert_eq!(details(&harness).selected_feature_id, "shared.feature.8");
    key(
        &harness,
        &child,
        NamedKey::Tab,
        KeyCode::Tab,
        ModifiersState::SHIFT,
    );
    assert_eq!(pane_data(&harness).focused_control.as_str(), "previous");
    key(
        &harness,
        &child,
        NamedKey::Enter,
        KeyCode::Enter,
        ModifiersState::empty(),
    );
    assert_eq!(details(&harness).page, 0);
    key(
        &harness,
        &child,
        NamedKey::PageDown,
        KeyCode::PageDown,
        ModifiersState::empty(),
    );
    assert_eq!(details(&harness).page, 1);
    key(
        &harness,
        &child,
        NamedKey::PageUp,
        KeyCode::PageUp,
        ModifiersState::empty(),
    );
    assert_eq!(details(&harness).page, 0);
    select(&harness, &child, &pane, 0, "shared.feature.0");
    key(
        &harness,
        &child,
        NamedKey::Tab,
        KeyCode::Tab,
        ModifiersState::empty(),
    );
    key(
        &harness,
        &child,
        NamedKey::Tab,
        KeyCode::Tab,
        ModifiersState::empty(),
    );
    assert_eq!(pane_data(&harness).focused_control.as_str(), "apply");
    key(
        &harness,
        &child,
        NamedKey::Enter,
        KeyCode::Enter,
        ModifiersState::empty(),
    );
    assert!(enabled_on_disk(
        &project,
        "vendor.second",
        "shared.feature.0"
    ));
    assert!(!enabled_on_disk(
        &project,
        "vendor.first",
        "shared.feature.0"
    ));
    key(
        &harness,
        &child,
        NamedKey::Escape,
        KeyCode::Escape,
        ModifiersState::empty(),
    );
    let list = pane_data(&harness);
    assert!(list.details.is_none());
    assert_eq!(list.selected_plugin_id.as_str(), "vendor.second");
    assert_eq!(list.focused_control.as_str(), "package");
    key(
        &harness,
        &child,
        NamedKey::Enter,
        KeyCode::Enter,
        ModifiersState::empty(),
    );
    assert_eq!(details(&harness).plugin_id, "vendor.second");
    harness
        .host
        .borrow()
        .asset_runtime_access
        .asset_manager()
        .unwrap()
        .close_project()
        .unwrap();
}

#[test]
fn feature_details_ignore_stale_actions_and_leave_other_panes_unaffected() {
    let _guard = lock_env();
    let other = ProjectFixture(unique_temp_path("zircon_feature_details_other_project"));
    let other_paths = ProjectPaths::from_root(&other.0).unwrap();
    other_paths
        .ensure_layout(&[RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Other Project",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(other_paths.manifest_path())
    .unwrap();
    let (project, harness, pane, child) = fixture("zircon_feature_details_stale");
    open_owner(&harness, &child, "vendor.first");
    select(&harness, &child, &pane, 1, "shared.feature.1");
    let old = details(&harness);
    pane_surface_host(&child)
        .invoke_surface_control_clicked("ModulePluginDetailsNext".into(), "".into());
    refresh(&harness);
    pointer_apply(&harness, &child, &old.intent_id);
    assert_eq!(details(&harness).page, 1);
    assert!(details(&harness).selected_feature_id.is_empty());
    assert!(!enabled_on_disk(
        &project,
        "vendor.first",
        "shared.feature.1"
    ));
    open_owner(&harness, &child, "vendor.second");
    select(&harness, &child, &pane, 1, "shared.feature.1");
    let current = details(&harness);
    pointer_apply(&harness, &child, &old.intent_id);
    assert!(!enabled_on_disk(
        &project,
        "vendor.second",
        "shared.feature.1"
    ));
    {
        let mut host = harness.host.borrow_mut();
        let assets = host.asset_runtime_access.asset_manager().unwrap();
        assets.close_project().unwrap();
        assets
            .open_project(other.0.to_string_lossy().as_ref())
            .unwrap();
        host.runtime
            .shell()
            .lock()
            .state
            .set_project_path(other.0.to_string_lossy().into_owned());
        host.runtime.refresh_reflection();
    }
    pointer_apply(&harness, &child, &current.intent_id);
    assert!(pane_data(&harness).details.is_none());
    assert!(!enabled_on_disk(
        &project,
        "vendor.second",
        "shared.feature.1"
    ));
    let baseline = harness.journal_len();
    let event = UiPointerComponentEvent::new(
        &UiTreeId::new("editor.other.pane"),
        UiNodeId::new(9),
        "OpenProject",
        "OpenProject/Click",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".into(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    )
    .with_template_action(UiTemplateActionInvocation::route(
        "file.project.open",
        Default::default(),
    ));
    pane_surface_host(&child).invoke_pointer_component_event(event);
    assert!(harness
        .delta_events_since(baseline)
        .iter()
        .any(|event| matches!(event, EditorEvent::WorkbenchMenu(MenuAction::OpenProject))));
    assert!(pane_data(&harness).details.is_none());
    harness
        .host
        .borrow()
        .asset_runtime_access
        .asset_manager()
        .unwrap()
        .close_project()
        .unwrap();
}
