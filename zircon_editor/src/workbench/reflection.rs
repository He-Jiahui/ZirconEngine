//! Editor-side projection into editor_ui activity descriptors and reflection models.

use serde_json::Value;
use zircon_editor_ui::{
    ActivityDrawerSlotPreference, ActivityViewDescriptor, ActivityWindowDescriptor, DockCommand,
    EditorActivityHost, EditorActivityKind, EditorActivityReflection, EditorDrawerReflectionModel,
    EditorFloatingWindowReflectionModel, EditorHostPageReflectionModel,
    EditorMenuItemReflectionModel, EditorUiBinding, EditorUiBindingPayload, EditorUiControlService,
    EditorWorkbenchReflectionModel, ViewportCommand,
};
use zircon_ui::{
    UiActionDescriptor, UiEventKind, UiNodePath, UiParameterDescriptor, UiTreeId, UiValueType,
};

use crate::layout::ActivityDrawerSlot;
use crate::snapshot::{
    DocumentWorkspaceSnapshot, EditorChromeSnapshot, FloatingWindowSnapshot, MainPageSnapshot,
    ViewContentKind, ViewTabSnapshot,
};
use crate::view::{DockPolicy, PreferredHost, ViewDescriptor, ViewKind};
use crate::WorkbenchViewModel;
pub fn activity_descriptors_from_views(
    descriptors: &[ViewDescriptor],
) -> (Vec<ActivityViewDescriptor>, Vec<ActivityWindowDescriptor>) {
    let mut activity_views = Vec::new();
    let mut activity_windows = Vec::new();

    for descriptor in descriptors {
        match descriptor.kind {
            ViewKind::ActivityView => {
                let mut activity = ActivityViewDescriptor::new(
                    descriptor.descriptor_id.0.clone(),
                    descriptor.default_title.clone(),
                    descriptor.icon_key.clone(),
                )
                .with_multi_instance(descriptor.multi_instance)
                .with_supports_document_host(!matches!(
                    descriptor.dock_policy,
                    DockPolicy::DrawerOnly
                ))
                .with_supports_floating_window(!matches!(
                    descriptor.dock_policy,
                    DockPolicy::DrawerOnly
                ))
                .with_reflection_root(UiNodePath::new(format!(
                    "editor/views/{}",
                    descriptor.descriptor_id.0
                )));
                if let Some(slot) = descriptor.preferred_drawer_slot {
                    activity = activity.with_default_drawer(drawer_slot_preference(slot));
                }
                activity_views.push(activity);
            }
            ViewKind::ActivityWindow => {
                let activity = ActivityWindowDescriptor::new(
                    descriptor.descriptor_id.0.clone(),
                    descriptor.default_title.clone(),
                    descriptor.icon_key.clone(),
                )
                .with_multi_instance(descriptor.multi_instance)
                .with_supports_document_tab(!matches!(
                    descriptor.preferred_host,
                    PreferredHost::ExclusiveMainPage
                ))
                .with_supports_exclusive_page(matches!(
                    descriptor.preferred_host,
                    PreferredHost::ExclusiveMainPage | PreferredHost::DocumentCenter
                ))
                .with_supports_floating_window(true)
                .with_reflection_root(UiNodePath::new(format!(
                    "editor/windows/{}",
                    descriptor.descriptor_id.0
                )));
                activity_windows.push(activity);
            }
        }
    }

    (activity_views, activity_windows)
}

pub fn build_workbench_reflection_model(
    chrome: &EditorChromeSnapshot,
    view_model: &WorkbenchViewModel,
) -> EditorWorkbenchReflectionModel {
    let mut model = EditorWorkbenchReflectionModel::new(UiTreeId::new("editor.workbench"));
    model.status_line = chrome.status_line.clone();
    model.menu_items = view_model
        .menu_bar
        .menus
        .iter()
        .flat_map(|menu| {
            let menu_id = menu_id(&menu.label);
            menu.items
                .iter()
                .map(move |item| EditorMenuItemReflectionModel {
                    menu_id: menu_id.clone(),
                    control_id: item.binding.path().control_id.clone(),
                    label: item.label.clone(),
                    enabled: item.enabled,
                    binding: item.binding.clone(),
                    route_id: None,
                })
        })
        .collect();

    model.pages = chrome
        .workbench
        .main_pages
        .iter()
        .map(|page| match page {
            MainPageSnapshot::Workbench {
                id,
                title,
                workspace,
            } => EditorHostPageReflectionModel {
                page_id: id.0.clone(),
                title: title.clone(),
                active: id == &chrome.workbench.active_main_page,
                exclusive: false,
                activities: collect_workspace_activities(
                    workspace,
                    EditorActivityHost::DocumentPage(id.0.clone()),
                ),
            },
            MainPageSnapshot::Exclusive { id, title, view } => EditorHostPageReflectionModel {
                page_id: id.0.clone(),
                title: title.clone(),
                active: id == &chrome.workbench.active_main_page,
                exclusive: true,
                activities: vec![activity_from_tab(
                    view,
                    EditorActivityHost::ExclusivePage(id.0.clone()),
                )],
            },
        })
        .collect();

    model.drawers = chrome
        .workbench
        .drawers
        .iter()
        .map(|(slot, drawer)| EditorDrawerReflectionModel {
            drawer_id: drawer_slot_name(*slot).to_string(),
            title: format!("{:?}", slot),
            visible: drawer.visible,
            activities: drawer
                .tabs
                .iter()
                .map(|tab| {
                    activity_from_tab(
                        tab,
                        EditorActivityHost::Drawer(drawer_slot_name(*slot).to_string()),
                    )
                })
                .collect(),
        })
        .collect();

    model.floating_windows = chrome
        .workbench
        .floating_windows
        .iter()
        .map(floating_window_model)
        .collect();

    model
}

pub fn register_workbench_reflection_routes(
    service: &mut EditorUiControlService,
    mut model: EditorWorkbenchReflectionModel,
) -> EditorWorkbenchReflectionModel {
    for item in &mut model.menu_items {
        if item.route_id.is_some() {
            continue;
        }
        item.route_id = Some(register_menu_route(service, item.binding.clone()));
    }

    for page in &mut model.pages {
        for activity in &mut page.activities {
            register_activity_routes(service, activity);
        }
    }
    for drawer in &mut model.drawers {
        for activity in &mut drawer.activities {
            register_activity_routes(service, activity);
        }
    }
    for window in &mut model.floating_windows {
        for activity in &mut window.activities {
            register_activity_routes(service, activity);
        }
    }

    model
}

fn collect_workspace_activities(
    workspace: &DocumentWorkspaceSnapshot,
    host: EditorActivityHost,
) -> Vec<EditorActivityReflection> {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            let mut activities = collect_workspace_activities(first, host.clone());
            activities.extend(collect_workspace_activities(second, host));
            activities
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, .. } => tabs
            .iter()
            .map(|tab| activity_from_tab(tab, host.clone()))
            .collect(),
    }
}

fn floating_window_model(window: &FloatingWindowSnapshot) -> EditorFloatingWindowReflectionModel {
    EditorFloatingWindowReflectionModel {
        window_id: window.window_id.0.clone(),
        title: window.title.clone(),
        activities: collect_workspace_activities(
            &window.workspace,
            EditorActivityHost::FloatingWindow(window.window_id.0.clone()),
        ),
    }
}

fn activity_from_tab(tab: &ViewTabSnapshot, host: EditorActivityHost) -> EditorActivityReflection {
    let mut properties = std::collections::BTreeMap::from([
        ("icon_key".to_string(), Value::String(tab.icon_key.clone())),
        (
            "content_kind".to_string(),
            Value::String(content_kind_name(tab.content_kind).to_string()),
        ),
        ("placeholder".to_string(), Value::Bool(tab.placeholder)),
    ]);
    if let Value::Object(object) = &tab.serializable_payload {
        for (key, value) in object {
            properties.insert(format!("payload.{key}"), value.clone());
        }
    }

    EditorActivityReflection {
        instance_id: tab.instance_id.0.clone(),
        descriptor_id: tab.descriptor_id.0.clone(),
        title: tab.title.clone(),
        kind: match tab.kind {
            ViewKind::ActivityView => EditorActivityKind::ActivityView,
            ViewKind::ActivityWindow => EditorActivityKind::ActivityWindow,
        },
        host,
        visible: true,
        enabled: !tab.placeholder,
        dirty: tab.dirty,
        properties,
        actions: activity_actions_for_tab(tab),
    }
}

fn activity_actions_for_tab(tab: &ViewTabSnapshot) -> Vec<UiActionDescriptor> {
    if tab.placeholder {
        return Vec::new();
    }

    let mut actions = vec![
        UiActionDescriptor::new("focus_view", UiEventKind::Click, "DockCommand.FocusView")
            .with_parameter(UiParameterDescriptor::new(
                "instance_id",
                UiValueType::String,
            )),
        UiActionDescriptor::new(
            "detach_to_window",
            UiEventKind::Click,
            "DockCommand.DetachViewToWindow",
        )
        .with_parameter(UiParameterDescriptor::new(
            "instance_id",
            UiValueType::String,
        ))
        .with_parameter(UiParameterDescriptor::new("window_id", UiValueType::String)),
    ];

    match tab.content_kind {
        ViewContentKind::Inspector => {
            actions.push(
                UiActionDescriptor::new("apply_batch", UiEventKind::Click, "InspectorFieldBatch")
                    .with_parameter(UiParameterDescriptor::new(
                        "subject_path",
                        UiValueType::String,
                    ))
                    .with_parameter(UiParameterDescriptor::new("changes", UiValueType::Array)),
            );
        }
        ViewContentKind::Scene | ViewContentKind::Game => {
            actions.push(
                UiActionDescriptor::new(
                    "pointer_move",
                    UiEventKind::Hover,
                    "ViewportCommand.PointerMoved",
                )
                .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
                .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
            );
            actions.push(
                UiActionDescriptor::new(
                    "left_press",
                    UiEventKind::Press,
                    "ViewportCommand.LeftPressed",
                )
                .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
                .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
            );
            actions.push(UiActionDescriptor::new(
                "left_release",
                UiEventKind::Release,
                "ViewportCommand.LeftReleased",
            ));
            actions.push(
                UiActionDescriptor::new(
                    "right_press",
                    UiEventKind::Press,
                    "ViewportCommand.RightPressed",
                )
                .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
                .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
            );
            actions.push(UiActionDescriptor::new(
                "right_release",
                UiEventKind::Release,
                "ViewportCommand.RightReleased",
            ));
            actions.push(
                UiActionDescriptor::new(
                    "middle_press",
                    UiEventKind::Press,
                    "ViewportCommand.MiddlePressed",
                )
                .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
                .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
            );
            actions.push(UiActionDescriptor::new(
                "middle_release",
                UiEventKind::Release,
                "ViewportCommand.MiddleReleased",
            ));
            actions.push(
                UiActionDescriptor::new("scroll", UiEventKind::Scroll, "ViewportCommand.Scrolled")
                    .with_parameter(UiParameterDescriptor::new("delta", UiValueType::Float)),
            );
            actions.push(
                UiActionDescriptor::new("resize", UiEventKind::Resize, "ViewportCommand.Resized")
                    .with_parameter(UiParameterDescriptor::new("width", UiValueType::Unsigned))
                    .with_parameter(UiParameterDescriptor::new("height", UiValueType::Unsigned)),
            );
        }
        _ => {}
    }

    actions
}

fn register_menu_route(
    service: &mut EditorUiControlService,
    binding: EditorUiBinding,
) -> zircon_ui::UiRouteId {
    register_stub_route(service, binding)
}

fn register_activity_routes(
    service: &mut EditorUiControlService,
    activity: &mut EditorActivityReflection,
) {
    let activity_meta = activity.clone();
    for action in &mut activity.actions {
        if action.route_id.is_some() {
            action.callable_from_remote = true;
            continue;
        }

        let route_id = match action.action_id.as_str() {
            "focus_view" | "detach_to_window" => register_docking_route(
                service,
                &activity_meta,
                action.action_id.as_str(),
                action.event_kind,
            ),
            "apply_batch" => register_inspector_route(service, &activity_meta, action.event_kind),
            "pointer_move" | "left_press" | "left_release" | "right_press" | "right_release"
            | "middle_press" | "middle_release" | "scroll" | "resize" => register_viewport_route(
                service,
                &activity_meta,
                action.action_id.as_str(),
                action.event_kind,
            ),
            _ => None,
        };

        if let Some(route_id) = route_id {
            action.route_id = Some(route_id);
            action.callable_from_remote = true;
        }
    }
}

fn register_docking_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<zircon_ui::UiRouteId> {
    let view_id = binding_view_id(activity);
    let control_id = match action_id {
        "focus_view" => "FocusViewButton",
        "detach_to_window" => "DetachViewButton",
        _ => return None,
    };
    let path = zircon_ui::UiEventPath::new(view_id, control_id, event_kind);
    let default_command = default_dock_command(activity, action_id)?;
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        EditorUiBindingPayload::dock_command(default_command.clone()),
    );
    Some(register_stub_route(service, registration_binding))
}

fn register_inspector_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    event_kind: UiEventKind,
) -> Option<zircon_ui::UiRouteId> {
    let path =
        zircon_ui::UiEventPath::new(binding_view_id(activity), "ApplyBatchButton", event_kind);
    let default_subject = "entity://selected".to_string();
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        EditorUiBindingPayload::inspector_field_batch(default_subject.clone(), Vec::new()),
    );
    Some(register_stub_route(service, registration_binding))
}

fn register_viewport_route(
    service: &mut EditorUiControlService,
    activity: &EditorActivityReflection,
    action_id: &str,
    event_kind: UiEventKind,
) -> Option<zircon_ui::UiRouteId> {
    let default_command = default_viewport_command(action_id)?;
    let path =
        zircon_ui::UiEventPath::new(binding_view_id(activity), "ViewportSurface", event_kind);
    let registration_binding = EditorUiBinding::new(
        path.view_id.clone(),
        path.control_id.clone(),
        path.event_kind,
        EditorUiBindingPayload::viewport_command(default_command.clone()),
    );
    Some(register_stub_route(service, registration_binding))
}

fn register_stub_route(
    service: &mut EditorUiControlService,
    binding: EditorUiBinding,
) -> zircon_ui::UiRouteId {
    service
        .route_id_for_binding(&binding.as_ui_binding())
        .unwrap_or_else(|| service.register_route_stub(binding.as_ui_binding()))
}

fn binding_view_id(activity: &EditorActivityReflection) -> String {
    match activity.descriptor_id.as_str() {
        "editor.project" => "ProjectView".to_string(),
        "editor.hierarchy" => "HierarchyView".to_string(),
        "editor.inspector" => "InspectorView".to_string(),
        "editor.scene" => "SceneView".to_string(),
        "editor.game" => "GameView".to_string(),
        "editor.assets" => "AssetsView".to_string(),
        "editor.console" => "ConsoleView".to_string(),
        "editor.prefab" => "PrefabEditorWindow".to_string(),
        "editor.asset_browser" => "AssetBrowserWindow".to_string(),
        _ => activity.instance_id.clone(),
    }
}

fn default_dock_command(
    activity: &EditorActivityReflection,
    action_id: &str,
) -> Option<DockCommand> {
    match action_id {
        "focus_view" => Some(DockCommand::FocusView {
            instance_id: activity.instance_id.clone(),
        }),
        "detach_to_window" => Some(DockCommand::DetachViewToWindow {
            instance_id: activity.instance_id.clone(),
            window_id: format!("window:{}", activity.instance_id),
        }),
        _ => None,
    }
}

fn default_viewport_command(action_id: &str) -> Option<ViewportCommand> {
    match action_id {
        "pointer_move" => Some(ViewportCommand::PointerMoved { x: 0.0, y: 0.0 }),
        "left_press" => Some(ViewportCommand::LeftPressed { x: 0.0, y: 0.0 }),
        "left_release" => Some(ViewportCommand::LeftReleased),
        "right_press" => Some(ViewportCommand::RightPressed { x: 0.0, y: 0.0 }),
        "right_release" => Some(ViewportCommand::RightReleased),
        "middle_press" => Some(ViewportCommand::MiddlePressed { x: 0.0, y: 0.0 }),
        "middle_release" => Some(ViewportCommand::MiddleReleased),
        "scroll" => Some(ViewportCommand::Scrolled { delta: 0.0 }),
        "resize" => Some(ViewportCommand::Resized {
            width: 1,
            height: 1,
        }),
        _ => None,
    }
}

fn drawer_slot_preference(slot: ActivityDrawerSlot) -> ActivityDrawerSlotPreference {
    match slot {
        ActivityDrawerSlot::LeftTop => ActivityDrawerSlotPreference::LeftTop,
        ActivityDrawerSlot::LeftBottom => ActivityDrawerSlotPreference::LeftBottom,
        ActivityDrawerSlot::RightTop => ActivityDrawerSlotPreference::RightTop,
        ActivityDrawerSlot::RightBottom => ActivityDrawerSlotPreference::RightBottom,
        ActivityDrawerSlot::BottomLeft => ActivityDrawerSlotPreference::BottomLeft,
        ActivityDrawerSlot::BottomRight => ActivityDrawerSlotPreference::BottomRight,
    }
}

fn drawer_slot_name(slot: ActivityDrawerSlot) -> &'static str {
    match slot {
        ActivityDrawerSlot::LeftTop => "left_top",
        ActivityDrawerSlot::LeftBottom => "left_bottom",
        ActivityDrawerSlot::RightTop => "right_top",
        ActivityDrawerSlot::RightBottom => "right_bottom",
        ActivityDrawerSlot::BottomLeft => "bottom_left",
        ActivityDrawerSlot::BottomRight => "bottom_right",
    }
}

fn menu_id(label: &str) -> String {
    label.to_ascii_lowercase().replace(' ', "_")
}

fn content_kind_name(kind: ViewContentKind) -> &'static str {
    match kind {
        ViewContentKind::Welcome => "welcome",
        ViewContentKind::Project => "project",
        ViewContentKind::Hierarchy => "hierarchy",
        ViewContentKind::Inspector => "inspector",
        ViewContentKind::Scene => "scene",
        ViewContentKind::Game => "game",
        ViewContentKind::Assets => "assets",
        ViewContentKind::Console => "console",
        ViewContentKind::PrefabEditor => "prefab_editor",
        ViewContentKind::AssetBrowser => "asset_browser",
        ViewContentKind::Placeholder => "placeholder",
    }
}
