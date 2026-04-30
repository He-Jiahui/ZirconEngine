use crate::ui::{
    EditorActivityHost, EditorDrawerReflectionModel, EditorHostPageReflectionModel,
    EditorMenuItemReflectionModel, EditorWorkbenchReflectionModel,
};
use zircon_runtime::ui::event_ui::UiTreeId;

use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, MainPageSnapshot};

use super::activity_collection::{
    activity_from_tab, collect_workspace_activities, floating_window_model,
};
use super::name_mapping::{drawer_slot_name, menu_id};

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
                    operation_path: item
                        .operation_path
                        .as_ref()
                        .map(|path| path.as_str().to_string()),
                    shortcut: item.shortcut.clone(),
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
                activity_window: _,
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
