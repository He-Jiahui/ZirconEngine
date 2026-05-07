use crate::core::editor_event::{
    ActivityDrawerMode as CoreActivityDrawerMode, ActivityDrawerSlot as CoreActivityDrawerSlot,
    LayoutCommand as CoreLayoutCommand, MainPageId as CoreMainPageId,
    TabInsertionAnchor as CoreTabInsertionAnchor, TabInsertionSide as CoreTabInsertionSide,
    ViewHost as CoreViewHost, ViewInstanceId as CoreViewInstanceId,
    WorkspaceTarget as CoreWorkspaceTarget,
};
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, LayoutCommand, MainPageId, TabInsertionAnchor,
    TabInsertionSide, WorkspaceTarget,
};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

pub(crate) fn core_layout_command_from_ui(command: LayoutCommand) -> CoreLayoutCommand {
    match command {
        LayoutCommand::OpenView {
            instance_id,
            target,
        } => CoreLayoutCommand::OpenView {
            instance_id: core_view_instance_id(instance_id),
            target: core_view_host(target),
        },
        LayoutCommand::CloseView { instance_id } => CoreLayoutCommand::CloseView {
            instance_id: core_view_instance_id(instance_id),
        },
        LayoutCommand::FocusView { instance_id } => CoreLayoutCommand::FocusView {
            instance_id: core_view_instance_id(instance_id),
        },
        LayoutCommand::MoveView {
            instance_id,
            target,
        } => CoreLayoutCommand::MoveView {
            instance_id: core_view_instance_id(instance_id),
            target: core_view_host(target),
        },
        LayoutCommand::AttachView {
            instance_id,
            target,
            anchor,
        } => CoreLayoutCommand::AttachView {
            instance_id: core_view_instance_id(instance_id),
            target: core_view_host(target),
            anchor: anchor.map(core_tab_insertion_anchor),
        },
        LayoutCommand::DetachViewToWindow {
            instance_id,
            new_window,
        } => CoreLayoutCommand::DetachViewToWindow {
            instance_id: core_view_instance_id(instance_id),
            new_window: core_main_page_id(new_window),
        },
        LayoutCommand::CreateSplit {
            workspace,
            path,
            axis,
            placement,
            new_instance,
        } => CoreLayoutCommand::CreateSplit {
            workspace: core_workspace_target(workspace),
            path,
            axis: match axis {
                crate::ui::workbench::layout::SplitAxis::Horizontal => {
                    crate::core::editor_event::SplitAxis::Horizontal
                }
                crate::ui::workbench::layout::SplitAxis::Vertical => {
                    crate::core::editor_event::SplitAxis::Vertical
                }
            },
            placement: match placement {
                crate::ui::workbench::layout::SplitPlacement::Before => {
                    crate::core::editor_event::SplitPlacement::Before
                }
                crate::ui::workbench::layout::SplitPlacement::After => {
                    crate::core::editor_event::SplitPlacement::After
                }
            },
            new_instance: core_view_instance_id(new_instance),
        },
        LayoutCommand::ResizeSplit {
            workspace,
            path,
            ratio,
        } => CoreLayoutCommand::ResizeSplit {
            workspace: core_workspace_target(workspace),
            path,
            ratio,
        },
        LayoutCommand::SetDrawerMode { slot, mode } => CoreLayoutCommand::SetDrawerMode {
            slot: core_activity_drawer_slot(slot),
            mode: core_activity_drawer_mode(mode),
        },
        LayoutCommand::SetDrawerExtent { slot, extent } => CoreLayoutCommand::SetDrawerExtent {
            slot: core_activity_drawer_slot(slot),
            extent,
        },
        LayoutCommand::ActivateDrawerTab { slot, instance_id } => {
            CoreLayoutCommand::ActivateDrawerTab {
                slot: core_activity_drawer_slot(slot),
                instance_id: core_view_instance_id(instance_id),
            }
        }
        LayoutCommand::ActivateMainPage { page_id } => CoreLayoutCommand::ActivateMainPage {
            page_id: core_main_page_id(page_id),
        },
        LayoutCommand::SavePreset { name } => CoreLayoutCommand::SavePreset { name },
        LayoutCommand::LoadPreset { name } => CoreLayoutCommand::LoadPreset { name },
        LayoutCommand::ResetToDefault => CoreLayoutCommand::ResetToDefault,
    }
}

pub(crate) fn ui_layout_command_from_core(command: &CoreLayoutCommand) -> LayoutCommand {
    match command {
        CoreLayoutCommand::OpenView {
            instance_id,
            target,
        } => LayoutCommand::OpenView {
            instance_id: ui_view_instance_id(instance_id),
            target: ui_view_host(target),
        },
        CoreLayoutCommand::CloseView { instance_id } => LayoutCommand::CloseView {
            instance_id: ui_view_instance_id(instance_id),
        },
        CoreLayoutCommand::FocusView { instance_id } => LayoutCommand::FocusView {
            instance_id: ui_view_instance_id(instance_id),
        },
        CoreLayoutCommand::MoveView {
            instance_id,
            target,
        } => LayoutCommand::MoveView {
            instance_id: ui_view_instance_id(instance_id),
            target: ui_view_host(target),
        },
        CoreLayoutCommand::AttachView {
            instance_id,
            target,
            anchor,
        } => LayoutCommand::AttachView {
            instance_id: ui_view_instance_id(instance_id),
            target: ui_view_host(target),
            anchor: anchor.as_ref().map(ui_tab_insertion_anchor),
        },
        CoreLayoutCommand::DetachViewToWindow {
            instance_id,
            new_window,
        } => LayoutCommand::DetachViewToWindow {
            instance_id: ui_view_instance_id(instance_id),
            new_window: ui_main_page_id(new_window),
        },
        CoreLayoutCommand::CreateSplit {
            workspace,
            path,
            axis,
            placement,
            new_instance,
        } => LayoutCommand::CreateSplit {
            workspace: ui_workspace_target(workspace),
            path: path.clone(),
            axis: match axis {
                crate::core::editor_event::SplitAxis::Horizontal => {
                    crate::ui::workbench::layout::SplitAxis::Horizontal
                }
                crate::core::editor_event::SplitAxis::Vertical => {
                    crate::ui::workbench::layout::SplitAxis::Vertical
                }
            },
            placement: match placement {
                crate::core::editor_event::SplitPlacement::Before => {
                    crate::ui::workbench::layout::SplitPlacement::Before
                }
                crate::core::editor_event::SplitPlacement::After => {
                    crate::ui::workbench::layout::SplitPlacement::After
                }
            },
            new_instance: ui_view_instance_id(new_instance),
        },
        CoreLayoutCommand::ResizeSplit {
            workspace,
            path,
            ratio,
        } => LayoutCommand::ResizeSplit {
            workspace: ui_workspace_target(workspace),
            path: path.clone(),
            ratio: *ratio,
        },
        CoreLayoutCommand::SetDrawerMode { slot, mode } => LayoutCommand::SetDrawerMode {
            slot: ui_activity_drawer_slot(*slot),
            mode: ui_activity_drawer_mode(*mode),
        },
        CoreLayoutCommand::SetDrawerExtent { slot, extent } => LayoutCommand::SetDrawerExtent {
            slot: ui_activity_drawer_slot(*slot),
            extent: *extent,
        },
        CoreLayoutCommand::ActivateDrawerTab { slot, instance_id } => {
            LayoutCommand::ActivateDrawerTab {
                slot: ui_activity_drawer_slot(*slot),
                instance_id: ui_view_instance_id(instance_id),
            }
        }
        CoreLayoutCommand::ActivateMainPage { page_id } => LayoutCommand::ActivateMainPage {
            page_id: ui_main_page_id(page_id),
        },
        CoreLayoutCommand::SavePreset { name } => LayoutCommand::SavePreset { name: name.clone() },
        CoreLayoutCommand::LoadPreset { name } => LayoutCommand::LoadPreset { name: name.clone() },
        CoreLayoutCommand::ResetToDefault => LayoutCommand::ResetToDefault,
    }
}

fn core_activity_drawer_mode(mode: ActivityDrawerMode) -> CoreActivityDrawerMode {
    match mode {
        ActivityDrawerMode::Pinned => CoreActivityDrawerMode::Pinned,
        ActivityDrawerMode::AutoHide => CoreActivityDrawerMode::AutoHide,
        ActivityDrawerMode::Collapsed => CoreActivityDrawerMode::Collapsed,
    }
}

fn ui_activity_drawer_mode(mode: CoreActivityDrawerMode) -> ActivityDrawerMode {
    match mode {
        CoreActivityDrawerMode::Pinned => ActivityDrawerMode::Pinned,
        CoreActivityDrawerMode::AutoHide => ActivityDrawerMode::AutoHide,
        CoreActivityDrawerMode::Collapsed => ActivityDrawerMode::Collapsed,
    }
}

fn core_activity_drawer_slot(slot: ActivityDrawerSlot) -> CoreActivityDrawerSlot {
    match slot {
        ActivityDrawerSlot::LeftTop => CoreActivityDrawerSlot::LeftTop,
        ActivityDrawerSlot::LeftBottom => CoreActivityDrawerSlot::LeftBottom,
        ActivityDrawerSlot::RightTop => CoreActivityDrawerSlot::RightTop,
        ActivityDrawerSlot::RightBottom => CoreActivityDrawerSlot::RightBottom,
        ActivityDrawerSlot::Bottom => CoreActivityDrawerSlot::Bottom,
        ActivityDrawerSlot::BottomLeft => CoreActivityDrawerSlot::BottomLeft,
        ActivityDrawerSlot::BottomRight => CoreActivityDrawerSlot::BottomRight,
    }
}

fn ui_activity_drawer_slot(slot: CoreActivityDrawerSlot) -> ActivityDrawerSlot {
    match slot {
        CoreActivityDrawerSlot::LeftTop => ActivityDrawerSlot::LeftTop,
        CoreActivityDrawerSlot::LeftBottom => ActivityDrawerSlot::LeftBottom,
        CoreActivityDrawerSlot::RightTop => ActivityDrawerSlot::RightTop,
        CoreActivityDrawerSlot::RightBottom => ActivityDrawerSlot::RightBottom,
        CoreActivityDrawerSlot::Bottom => ActivityDrawerSlot::Bottom,
        CoreActivityDrawerSlot::BottomLeft => ActivityDrawerSlot::BottomLeft,
        CoreActivityDrawerSlot::BottomRight => ActivityDrawerSlot::BottomRight,
    }
}

fn core_main_page_id(page_id: MainPageId) -> CoreMainPageId {
    CoreMainPageId::new(page_id.0)
}

fn ui_main_page_id(page_id: &CoreMainPageId) -> MainPageId {
    MainPageId::new(page_id.0.clone())
}

fn core_tab_insertion_anchor(anchor: TabInsertionAnchor) -> CoreTabInsertionAnchor {
    CoreTabInsertionAnchor {
        target_id: core_view_instance_id(anchor.target_id),
        side: match anchor.side {
            TabInsertionSide::Before => CoreTabInsertionSide::Before,
            TabInsertionSide::After => CoreTabInsertionSide::After,
        },
    }
}

fn ui_tab_insertion_anchor(anchor: &CoreTabInsertionAnchor) -> TabInsertionAnchor {
    TabInsertionAnchor {
        target_id: ui_view_instance_id(&anchor.target_id),
        side: match anchor.side {
            CoreTabInsertionSide::Before => TabInsertionSide::Before,
            CoreTabInsertionSide::After => TabInsertionSide::After,
        },
    }
}

fn core_view_host(host: ViewHost) -> CoreViewHost {
    match host {
        ViewHost::Drawer(slot) => CoreViewHost::Drawer(core_activity_drawer_slot(slot)),
        ViewHost::Document(page_id, path) => {
            CoreViewHost::Document(core_main_page_id(page_id), path)
        }
        ViewHost::FloatingWindow(page_id, path) => {
            CoreViewHost::FloatingWindow(core_main_page_id(page_id), path)
        }
        ViewHost::ExclusivePage(page_id) => CoreViewHost::ExclusivePage(core_main_page_id(page_id)),
    }
}

fn ui_view_host(host: &CoreViewHost) -> ViewHost {
    match host {
        CoreViewHost::Drawer(slot) => ViewHost::Drawer(ui_activity_drawer_slot(*slot)),
        CoreViewHost::Document(page_id, path) => {
            ViewHost::Document(ui_main_page_id(page_id), path.clone())
        }
        CoreViewHost::FloatingWindow(page_id, path) => {
            ViewHost::FloatingWindow(ui_main_page_id(page_id), path.clone())
        }
        CoreViewHost::ExclusivePage(page_id) => ViewHost::ExclusivePage(ui_main_page_id(page_id)),
    }
}

fn core_view_instance_id(instance_id: ViewInstanceId) -> CoreViewInstanceId {
    CoreViewInstanceId::new(instance_id.0)
}

fn ui_view_instance_id(instance_id: &CoreViewInstanceId) -> ViewInstanceId {
    ViewInstanceId::new(instance_id.0.clone())
}

fn core_workspace_target(target: WorkspaceTarget) -> CoreWorkspaceTarget {
    match target {
        WorkspaceTarget::MainPage(page_id) => {
            CoreWorkspaceTarget::MainPage(core_main_page_id(page_id))
        }
        WorkspaceTarget::FloatingWindow(page_id) => {
            CoreWorkspaceTarget::FloatingWindow(core_main_page_id(page_id))
        }
    }
}

fn ui_workspace_target(target: &CoreWorkspaceTarget) -> WorkspaceTarget {
    match target {
        CoreWorkspaceTarget::MainPage(page_id) => {
            WorkspaceTarget::MainPage(ui_main_page_id(page_id))
        }
        CoreWorkspaceTarget::FloatingWindow(page_id) => {
            WorkspaceTarget::FloatingWindow(ui_main_page_id(page_id))
        }
    }
}
