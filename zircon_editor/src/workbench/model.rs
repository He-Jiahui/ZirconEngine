//! Backend-neutral workbench view models shared by preview and desktop hosts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_editor_ui::EditorUiBinding;

use crate::layout::{ActivityDrawerSlot, MainPageId};
use crate::snapshot::{
    DocumentWorkspaceSnapshot, EditorChromeSnapshot, MainPageSnapshot, ViewContentKind,
    ViewTabSnapshot,
};
use crate::view::{ViewDescriptorId, ViewInstanceId};
use crate::workbench::event::menu_action_binding;
use zircon_scene::NodeKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuAction {
    OpenProject,
    SaveProject,
    SaveLayout,
    ResetLayout,
    Undo,
    Redo,
    CreateNode(NodeKind),
    DeleteSelected,
    OpenView(ViewDescriptorId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItemModel {
    pub label: String,
    pub action: MenuAction,
    pub binding: EditorUiBinding,
    pub shortcut: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuModel {
    pub label: String,
    pub items: Vec<MenuItemModel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuBarModel {
    pub menus: Vec<MenuModel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreadcrumbModel {
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaneActionModel {
    pub label: String,
    pub binding: Option<EditorUiBinding>,
    pub prominent: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaneEmptyStateModel {
    pub title: String,
    pub body: String,
    pub primary_action: Option<PaneActionModel>,
    pub secondary_action: Option<PaneActionModel>,
    pub secondary_hint: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaneTabModel {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub icon_key: String,
    pub content_kind: ViewContentKind,
    pub active: bool,
    pub closeable: bool,
    pub empty_state: Option<PaneEmptyStateModel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolWindowStackModel {
    pub slot: ActivityDrawerSlot,
    pub mode: crate::ActivityDrawerMode,
    pub visible: bool,
    pub tabs: Vec<PaneTabModel>,
    pub active_tab: Option<ViewInstanceId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentTabModel {
    pub page_id: MainPageId,
    pub workspace_path: Vec<usize>,
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub icon_key: String,
    pub content_kind: ViewContentKind,
    pub active: bool,
    pub closeable: bool,
    pub empty_state: Option<PaneEmptyStateModel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostPageTabModel {
    pub id: MainPageId,
    pub title: String,
    pub dirty: bool,
    pub closeable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainHostStripModel {
    Workbench,
    ExclusiveWindow {
        instance_id: crate::view::ViewInstanceId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MainHostStripViewModel {
    pub mode: MainHostStripModel,
    pub pages: Vec<HostPageTabModel>,
    pub active_page: MainPageId,
    pub breadcrumbs: Vec<BreadcrumbModel>,
}

#[derive(Clone, Debug)]
pub struct DrawerRingModel {
    pub visible: bool,
    pub drawers: BTreeMap<ActivityDrawerSlot, crate::ActivityDrawerSnapshot>,
}

#[derive(Clone, Debug)]
pub enum DocumentWorkspaceModel {
    Workbench {
        page_id: MainPageId,
        title: String,
        workspace: DocumentWorkspaceSnapshot,
    },
    Exclusive {
        page_id: MainPageId,
        title: String,
        view: ViewTabSnapshot,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusBarModel {
    pub primary_text: String,
    pub secondary_text: Option<String>,
    pub viewport_label: String,
}

#[derive(Clone, Debug)]
pub struct WorkbenchViewModel {
    pub menu_bar: MenuBarModel,
    pub host_strip: MainHostStripViewModel,
    pub drawer_ring: DrawerRingModel,
    pub tool_windows: BTreeMap<ActivityDrawerSlot, ToolWindowStackModel>,
    pub document_tabs: Vec<DocumentTabModel>,
    pub document: DocumentWorkspaceModel,
    pub status_bar: StatusBarModel,
}

impl WorkbenchViewModel {
    pub fn build(chrome: &EditorChromeSnapshot) -> Self {
        let active_page = chrome
            .workbench
            .main_pages
            .iter()
            .find(|page| page_id(page) == &chrome.workbench.active_main_page)
            .cloned()
            .unwrap_or_else(|| {
                chrome
                    .workbench
                    .main_pages
                    .first()
                    .cloned()
                    .unwrap_or_else(|| MainPageSnapshot::Workbench {
                        id: MainPageId::workbench(),
                        title: "Workbench".to_string(),
                        workspace: DocumentWorkspaceSnapshot::Tabs {
                            tabs: Vec::new(),
                            active_tab: None,
                        },
                    })
            });

        let host_strip = MainHostStripViewModel {
            mode: match &active_page {
                MainPageSnapshot::Workbench { .. } => MainHostStripModel::Workbench,
                MainPageSnapshot::Exclusive { view, .. } => MainHostStripModel::ExclusiveWindow {
                    instance_id: view.instance_id.clone(),
                },
            },
            pages: chrome
                .workbench
                .main_pages
                .iter()
                .map(|page| HostPageTabModel {
                    id: page_id(page).clone(),
                    title: page_title(page).to_string(),
                    dirty: page_dirty(page),
                    closeable: matches!(page, MainPageSnapshot::Exclusive { .. }),
                })
                .collect(),
            active_page: page_id(&active_page).clone(),
            breadcrumbs: breadcrumbs_for_page(&active_page, chrome),
        };
        let drawer_visible = matches!(active_page, MainPageSnapshot::Workbench { .. });
        let tool_windows = chrome
            .workbench
            .drawers
            .iter()
            .map(|(slot, drawer)| {
                (
                    *slot,
                    ToolWindowStackModel {
                        slot: *slot,
                        mode: drawer.mode,
                        visible: drawer.visible,
                        active_tab: drawer.active_tab.clone(),
                        tabs: drawer
                            .tabs
                            .iter()
                            .map(|tab| {
                                pane_tab_model(
                                    tab,
                                    drawer.active_tab.as_ref() == Some(&tab.instance_id),
                                    chrome,
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect();
        let document_tabs = document_tabs_for_page(&active_page, chrome);

        Self {
            menu_bar: default_menu_bar(chrome),
            host_strip,
            drawer_ring: DrawerRingModel {
                visible: drawer_visible,
                drawers: chrome.workbench.drawers.clone(),
            },
            tool_windows,
            document_tabs,
            document: match active_page {
                MainPageSnapshot::Workbench {
                    id,
                    title,
                    workspace,
                } => DocumentWorkspaceModel::Workbench {
                    page_id: id,
                    title,
                    workspace,
                },
                MainPageSnapshot::Exclusive { id, title, view } => {
                    DocumentWorkspaceModel::Exclusive {
                        page_id: id,
                        title,
                        view,
                    }
                }
            },
            status_bar: StatusBarModel {
                primary_text: chrome.status_line.clone(),
                secondary_text: chrome
                    .inspector
                    .as_ref()
                    .map(|inspector| format!("Selection {}", inspector.name)),
                viewport_label: format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            },
        }
    }
}

fn default_menu_bar(chrome: &EditorChromeSnapshot) -> MenuBarModel {
    MenuBarModel {
        menus: vec![
            MenuModel {
                label: "File".to_string(),
                items: vec![
                    MenuItemModel {
                        label: "Open Project".to_string(),
                        action: MenuAction::OpenProject,
                        binding: menu_action_binding(&MenuAction::OpenProject),
                        shortcut: Some("Ctrl+O".to_string()),
                        enabled: true,
                    },
                    MenuItemModel {
                        label: "Save Project".to_string(),
                        action: MenuAction::SaveProject,
                        binding: menu_action_binding(&MenuAction::SaveProject),
                        shortcut: Some("Ctrl+S".to_string()),
                        enabled: chrome.project_open,
                    },
                    MenuItemModel {
                        label: "Save Layout".to_string(),
                        action: MenuAction::SaveLayout,
                        binding: menu_action_binding(&MenuAction::SaveLayout),
                        shortcut: None,
                        enabled: true,
                    },
                    MenuItemModel {
                        label: "Reset Layout".to_string(),
                        action: MenuAction::ResetLayout,
                        binding: menu_action_binding(&MenuAction::ResetLayout),
                        shortcut: None,
                        enabled: true,
                    },
                ],
            },
            MenuModel {
                label: "Edit".to_string(),
                items: vec![
                    MenuItemModel {
                        label: "Undo".to_string(),
                        action: MenuAction::Undo,
                        binding: menu_action_binding(&MenuAction::Undo),
                        shortcut: Some("Ctrl+Z".to_string()),
                        enabled: chrome.can_undo,
                    },
                    MenuItemModel {
                        label: "Redo".to_string(),
                        action: MenuAction::Redo,
                        binding: menu_action_binding(&MenuAction::Redo),
                        shortcut: Some("Ctrl+Shift+Z".to_string()),
                        enabled: chrome.can_redo,
                    },
                ],
            },
            MenuModel {
                label: "Selection".to_string(),
                items: vec![
                    MenuItemModel {
                        label: "Create Cube".to_string(),
                        action: MenuAction::CreateNode(NodeKind::Cube),
                        binding: menu_action_binding(&MenuAction::CreateNode(NodeKind::Cube)),
                        shortcut: None,
                        enabled: true,
                    },
                    MenuItemModel {
                        label: "Create Camera".to_string(),
                        action: MenuAction::CreateNode(NodeKind::Camera),
                        binding: menu_action_binding(&MenuAction::CreateNode(NodeKind::Camera)),
                        shortcut: None,
                        enabled: true,
                    },
                    MenuItemModel {
                        label: "Create Directional Light".to_string(),
                        action: MenuAction::CreateNode(NodeKind::DirectionalLight),
                        binding: menu_action_binding(&MenuAction::CreateNode(
                            NodeKind::DirectionalLight,
                        )),
                        shortcut: None,
                        enabled: true,
                    },
                    MenuItemModel {
                        label: "Delete Selection".to_string(),
                        action: MenuAction::DeleteSelected,
                        binding: menu_action_binding(&MenuAction::DeleteSelected),
                        shortcut: Some("Delete".to_string()),
                        enabled: chrome.inspector.is_some(),
                    },
                ],
            },
            MenuModel {
                label: "View".to_string(),
                items: builtin_view_menu_items(),
            },
            MenuModel {
                label: "Window".to_string(),
                items: vec![MenuItemModel {
                    label: "Reset Layout".to_string(),
                    action: MenuAction::ResetLayout,
                    binding: menu_action_binding(&MenuAction::ResetLayout),
                    shortcut: None,
                    enabled: true,
                }],
            },
            MenuModel {
                label: "Help".to_string(),
                items: vec![MenuItemModel {
                    label: "Workbench Guide".to_string(),
                    action: MenuAction::OpenView(ViewDescriptorId::new("editor.asset_browser")),
                    binding: menu_action_binding(&MenuAction::OpenView(ViewDescriptorId::new(
                        "editor.asset_browser",
                    ))),
                    shortcut: None,
                    enabled: true,
                }],
            },
        ],
    }
}

fn builtin_view_menu_items() -> Vec<MenuItemModel> {
    [
        ("Project", "editor.project"),
        ("Hierarchy", "editor.hierarchy"),
        ("Inspector", "editor.inspector"),
        ("Scene", "editor.scene"),
        ("Game", "editor.game"),
        ("Assets", "editor.assets"),
        ("Console", "editor.console"),
        ("Prefab Editor", "editor.prefab"),
        ("Asset Browser", "editor.asset_browser"),
    ]
    .into_iter()
    .map(|(label, descriptor_id)| MenuItemModel {
        label: label.to_string(),
        action: MenuAction::OpenView(ViewDescriptorId::new(descriptor_id)),
        binding: menu_action_binding(&MenuAction::OpenView(ViewDescriptorId::new(descriptor_id))),
        shortcut: None,
        enabled: true,
    })
    .collect()
}

fn breadcrumbs_for_page(
    page: &MainPageSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Vec<BreadcrumbModel> {
    match page {
        MainPageSnapshot::Workbench {
            title, workspace, ..
        } => {
            let mut breadcrumbs = vec![BreadcrumbModel {
                label: title.clone(),
            }];
            if let Some(active_view) = active_view_in_workspace(workspace) {
                breadcrumbs.push(BreadcrumbModel {
                    label: active_view.title.clone(),
                });
            }
            breadcrumbs
        }
        MainPageSnapshot::Exclusive { title, view, .. } => {
            let mut breadcrumbs = vec![BreadcrumbModel {
                label: title.clone(),
            }];
            if view.content_kind == ViewContentKind::Welcome {
                breadcrumbs.push(BreadcrumbModel {
                    label: chrome.welcome.title.clone(),
                });
            } else if let Some(path) = view
                .serializable_payload
                .get("path")
                .and_then(|value| value.as_str())
            {
                breadcrumbs.push(BreadcrumbModel {
                    label: path.to_string(),
                });
            } else {
                breadcrumbs.push(BreadcrumbModel {
                    label: view.title.clone(),
                });
            }
            breadcrumbs
        }
    }
}

fn active_view_in_workspace(workspace: &DocumentWorkspaceSnapshot) -> Option<&ViewTabSnapshot> {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            active_view_in_workspace(first).or_else(|| active_view_in_workspace(second))
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => active_tab
            .as_ref()
            .and_then(|active_id| tabs.iter().find(|tab| &tab.instance_id == active_id))
            .or_else(|| tabs.first()),
    }
}

fn page_id(page: &MainPageSnapshot) -> &MainPageId {
    match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => id,
    }
}

fn page_title(page: &MainPageSnapshot) -> &str {
    match page {
        MainPageSnapshot::Workbench { title, .. } | MainPageSnapshot::Exclusive { title, .. } => {
            title
        }
    }
}

fn page_dirty(page: &MainPageSnapshot) -> bool {
    match page {
        MainPageSnapshot::Workbench { workspace, .. } => active_view_in_workspace(workspace)
            .map(|view| view.dirty)
            .unwrap_or(false),
        MainPageSnapshot::Exclusive { view, .. } => view.dirty,
    }
}

fn pane_tab_model(
    tab: &ViewTabSnapshot,
    active: bool,
    chrome: &EditorChromeSnapshot,
) -> PaneTabModel {
    PaneTabModel {
        instance_id: tab.instance_id.clone(),
        descriptor_id: tab.descriptor_id.clone(),
        title: tab.title.clone(),
        icon_key: tab.icon_key.clone(),
        content_kind: tab.content_kind,
        active,
        closeable: is_closeable_content_kind(tab.content_kind),
        empty_state: empty_state_for_tab(tab, chrome),
    }
}

fn document_tabs_for_page(
    page: &MainPageSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Vec<DocumentTabModel> {
    match page {
        MainPageSnapshot::Workbench { id, workspace, .. } => {
            let mut tabs = Vec::new();
            collect_document_tabs(workspace, id, &mut Vec::new(), chrome, &mut tabs);
            tabs
        }
        MainPageSnapshot::Exclusive { id, view, .. } => vec![DocumentTabModel {
            page_id: id.clone(),
            workspace_path: Vec::new(),
            instance_id: view.instance_id.clone(),
            descriptor_id: view.descriptor_id.clone(),
            title: view.title.clone(),
            icon_key: view.icon_key.clone(),
            content_kind: view.content_kind,
            active: true,
            closeable: is_closeable_content_kind(view.content_kind),
            empty_state: empty_state_for_tab(view, chrome),
        }],
    }
}

fn collect_document_tabs(
    workspace: &DocumentWorkspaceSnapshot,
    page_id: &MainPageId,
    path: &mut Vec<usize>,
    chrome: &EditorChromeSnapshot,
    output: &mut Vec<DocumentTabModel>,
) {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            path.push(0);
            collect_document_tabs(first, page_id, path, chrome, output);
            path.pop();
            path.push(1);
            collect_document_tabs(second, page_id, path, chrome, output);
            path.pop();
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => {
            for tab in tabs {
                output.push(DocumentTabModel {
                    page_id: page_id.clone(),
                    workspace_path: path.clone(),
                    instance_id: tab.instance_id.clone(),
                    descriptor_id: tab.descriptor_id.clone(),
                    title: tab.title.clone(),
                    icon_key: tab.icon_key.clone(),
                    content_kind: tab.content_kind,
                    active: active_tab.as_ref() == Some(&tab.instance_id),
                    closeable: is_closeable_content_kind(tab.content_kind),
                    empty_state: empty_state_for_tab(tab, chrome),
                });
            }
        }
    }
}

fn empty_state_for_tab(
    tab: &ViewTabSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Option<PaneEmptyStateModel> {
    match tab.content_kind {
        ViewContentKind::Welcome => None,
        ViewContentKind::Project | ViewContentKind::Assets if !chrome.project_open => {
            Some(PaneEmptyStateModel {
                title: "No project open".to_string(),
                body: "Open a project to browse files, assets, and content roots.".to_string(),
                primary_action: Some(open_project_action()),
                secondary_action: None,
                secondary_hint: Some(
                    "Recent Projects is available from the File menu.".to_string(),
                ),
            })
        }
        ViewContentKind::Hierarchy if !chrome.project_open => Some(PaneEmptyStateModel {
            title: "No scene loaded".to_string(),
            body: "Open a project to inspect the active scene hierarchy.".to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Hierarchy if chrome.scene_entries.is_empty() => {
            Some(PaneEmptyStateModel {
                title: "No nodes in scene".to_string(),
                body: "Create or open a scene to populate the hierarchy.".to_string(),
                primary_action: None,
                secondary_action: None,
                secondary_hint: None,
            })
        }
        ViewContentKind::Scene if !chrome.project_open => Some(PaneEmptyStateModel {
            title: "No project open".to_string(),
            body: "Open a project to enter the editor workspace.".to_string(),
            primary_action: Some(open_project_action()),
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Scene if chrome.scene_entries.is_empty() => Some(PaneEmptyStateModel {
            title: "No active scene".to_string(),
            body: "Open Scene or Create Scene to begin editing.".to_string(),
            primary_action: Some(PaneActionModel {
                label: "Open Scene".to_string(),
                binding: None,
                prominent: true,
            }),
            secondary_action: Some(PaneActionModel {
                label: "Create Scene".to_string(),
                binding: None,
                prominent: false,
            }),
            secondary_hint: None,
        }),
        ViewContentKind::Inspector if chrome.inspector.is_none() => Some(PaneEmptyStateModel {
            title: "Nothing selected".to_string(),
            body: "Select an item in Hierarchy or Scene to inspect it.".to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Console if chrome.status_line.trim().is_empty() => {
            Some(PaneEmptyStateModel {
                title: "No output yet".to_string(),
                body: "Recent task output will appear here.".to_string(),
                primary_action: None,
                secondary_action: None,
                secondary_hint: None,
            })
        }
        ViewContentKind::Console => Some(PaneEmptyStateModel {
            title: "Last task status".to_string(),
            body: chrome.status_line.clone(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Placeholder => Some(PaneEmptyStateModel {
            title: "View unavailable".to_string(),
            body: "This pane was restored from layout state but its descriptor is missing."
                .to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        _ => None,
    }
}

fn open_project_action() -> PaneActionModel {
    PaneActionModel {
        label: "Open Project".to_string(),
        binding: Some(menu_action_binding(&MenuAction::OpenProject)),
        prominent: true,
    }
}

fn is_closeable_content_kind(kind: ViewContentKind) -> bool {
    matches!(
        kind,
        ViewContentKind::PrefabEditor
            | ViewContentKind::AssetBrowser
            | ViewContentKind::Placeholder
    )
}
