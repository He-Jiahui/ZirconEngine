use crate::ui::retained_host::callback_dispatch::dispatch_builtin_floating_window_focus_for_source;
use crate::ui::retained_host::UiHostWindow;
use crate::ui::workbench::autolayout::{ShellFrame, ShellRegionId};
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::snapshot::{MainPageSnapshot, ViewContentKind, WorkbenchSnapshot};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use super::super::{workbench_snapshot_access, RetainedEditorHost};
use super::active_workspace_tab;

pub(crate) fn resolve_callback_source_window_id(ui: &UiHostWindow) -> Option<MainPageId> {
    let host_shell = ui.get_host_presentation().host_shell;
    if !host_shell.native_floating_window_mode {
        return None;
    }

    let window_id = host_shell.native_floating_window_id.to_string();
    if window_id.trim().is_empty() {
        None
    } else {
        Some(MainPageId::new(window_id))
    }
}

fn is_valid_size(size: UiSize) -> bool {
    size.width > 0.0 && size.height > 0.0
}

fn frame_size(frame: ShellFrame) -> Option<UiSize> {
    let size = UiSize::new(frame.width.max(0.0), frame.height.max(0.0));
    is_valid_size(size).then_some(size)
}

fn ui_frame_size(frame: UiFrame) -> Option<UiSize> {
    let size = UiSize::new(frame.width.max(0.0), frame.height.max(0.0));
    is_valid_size(size).then_some(size)
}

fn drawer_slot_region(slot: ActivityDrawerSlot) -> ShellRegionId {
    match slot {
        ActivityDrawerSlot::LeftTop | ActivityDrawerSlot::LeftBottom => ShellRegionId::Left,
        ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom => ShellRegionId::Right,
        ActivityDrawerSlot::Bottom
        | ActivityDrawerSlot::BottomLeft
        | ActivityDrawerSlot::BottomRight => ShellRegionId::Bottom,
    }
}

fn active_drawer_region_for_kind(
    workbench: &WorkbenchSnapshot,
    kind: ViewContentKind,
) -> Option<ShellRegionId> {
    workbench
        .drawers
        .values()
        .find(|drawer| {
            drawer.visible
                && drawer.mode != ActivityDrawerMode::Collapsed
                && drawer
                    .active_tab
                    .as_ref()
                    .and_then(|active| drawer.tabs.iter().find(|tab| &tab.instance_id == active))
                    .or_else(|| drawer.tabs.first())
                    .is_some_and(|tab| tab.content_kind == kind)
        })
        .map(|drawer| drawer_slot_region(drawer.slot))
}

fn active_main_page_matches_kind(workbench: &WorkbenchSnapshot, kind: ViewContentKind) -> bool {
    let Some(page) = workbench.main_pages.iter().find(|page| match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
            id == &workbench.active_main_page
        }
    }) else {
        return false;
    };

    match page {
        MainPageSnapshot::Workbench { workspace, .. } => {
            active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
        }
        MainPageSnapshot::Exclusive { view, .. } => view.content_kind == kind,
    }
}

fn active_workbench_main_page_matches_kind(
    workbench: &WorkbenchSnapshot,
    kind: ViewContentKind,
) -> bool {
    let Some(MainPageSnapshot::Workbench { workspace, .. }) =
        workbench.main_pages.iter().find(|page| match page {
            MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
                id == &workbench.active_main_page
            }
        })
    else {
        return false;
    };

    active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
}

fn asset_surface_kind(surface_mode: &str) -> Option<ViewContentKind> {
    match surface_mode {
        "activity" => Some(ViewContentKind::Assets),
        "browser" => Some(ViewContentKind::AssetBrowser),
        _ => None,
    }
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn resolve_floating_window_content_frame_for_window(
        &self,
        window_id: &MainPageId,
    ) -> Option<ShellFrame> {
        self.floating_window_projection_bundle
            .content_frame(window_id)
    }

    fn resolve_native_floating_window_content_size_for_window(
        &self,
        window_id: &MainPageId,
    ) -> Option<UiSize> {
        let window = self.native_window_presenters.window(window_id)?;
        let bounds = window
            .get_host_presentation()
            .host_shell
            .native_window_bounds;
        let size = UiSize::new(
            bounds.width.max(0.0),
            (bounds.height
                - self.chrome_metrics.document_header_height
                - self.chrome_metrics.separator_thickness)
                .max(0.0),
        );
        is_valid_size(size).then_some(size)
    }

    pub(in crate::ui::retained_host::app) fn with_callback_source_window<T>(
        &mut self,
        source_window_id: Option<MainPageId>,
        callback: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self.callback_source_window.clone();
        self.callback_source_window = source_window_id;
        let result = callback(self);
        self.callback_source_window = previous;
        result
    }

    pub(in crate::ui::retained_host::app) fn focus_callback_source_window(&mut self) {
        let source_window_id = self.callback_source_window.clone();
        let Some(source_window_id) = source_window_id else {
            self.last_focused_callback_window = None;
            return;
        };

        match dispatch_builtin_floating_window_focus_for_source(
            &self.runtime,
            Some(&source_window_id),
            self.last_focused_callback_window.as_ref(),
        ) {
            Some(Ok(effects)) => {
                self.apply_dispatch_effects(effects);
                self.last_focused_callback_window = Some(source_window_id);
            }
            Some(Err(error)) => self.set_status_line(error),
            None => {
                self.last_focused_callback_window = Some(source_window_id);
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn note_focused_floating_window(
        &mut self,
        window_id: Option<MainPageId>,
    ) {
        self.last_focused_callback_window = window_id;
    }

    pub(in crate::ui::retained_host::app) fn note_focused_floating_window_surface(
        &mut self,
        surface_key: &str,
    ) {
        if surface_key == "main" {
            self.last_focused_callback_window = None;
            return;
        }

        let chrome = self.runtime.chrome_snapshot();
        self.last_focused_callback_window =
            workbench_snapshot_access::floating_window_id_for_surface_key(
                &chrome.workbench,
                surface_key,
            );
    }

    fn resolve_host_frame_backed_size_for_kind(&self, kind: ViewContentKind) -> Option<UiSize> {
        if let Some(window_id) = self.callback_source_window.as_ref() {
            return self
                .resolve_floating_window_content_frame_for_window(window_id)
                .and_then(frame_size)
                .or_else(|| {
                    self.resolve_native_floating_window_content_size_for_window(window_id)
                });
        }

        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        let chrome = self.runtime.chrome_snapshot();
        let workbench = &chrome.workbench;
        if let Some(region) = active_drawer_region_for_kind(workbench, kind) {
            return workbench_layout_frames
                .drawer_content_frame(region)
                .and_then(ui_frame_size);
        }

        if active_workbench_main_page_matches_kind(workbench, kind) {
            if matches!(kind, ViewContentKind::Scene | ViewContentKind::Game) {
                if let Some(size) = workbench_layout_frames
                    .viewport_content_frame
                    .and_then(ui_frame_size)
                {
                    return Some(size);
                }
            }
            if let Some(size) = workbench_layout_frames
                .document_region_frame
                .and_then(ui_frame_size)
            {
                return Some(size);
            }
            return None;
        }

        if active_main_page_matches_kind(workbench, kind) {
            let root_shell_frames = self.template_bridge.root_shell_frames();
            return root_shell_frames
                .pane_surface_frame
                .and_then(ui_frame_size)
                .or_else(|| {
                    root_shell_frames
                        .document_host_frame
                        .and_then(ui_frame_size)
                });
        }

        None
    }

    pub(in crate::ui::retained_host::app) fn resolve_callback_surface_size_for_kind(
        &self,
        width: f32,
        height: f32,
        cached_size: UiSize,
        kind: ViewContentKind,
    ) -> UiSize {
        let callback_size = UiSize::new(width.max(0.0), height.max(0.0));
        if is_valid_size(callback_size) {
            return callback_size;
        }
        if is_valid_size(cached_size) {
            return cached_size;
        }

        self.resolve_host_frame_backed_size_for_kind(kind)
            .unwrap_or(UiSize::new(0.0, 0.0))
    }

    pub(in crate::ui::retained_host::app) fn resolve_callback_surface_size_for_asset_surface(
        &self,
        surface_mode: &str,
        width: f32,
        height: f32,
        cached_size: UiSize,
    ) -> Option<UiSize> {
        asset_surface_kind(surface_mode).map(|kind| {
            self.resolve_callback_surface_size_for_kind(width, height, cached_size, kind)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_callback_source_window_id;
    use crate::ui::retained_host::UiHostWindow;
    use crate::ui::workbench::layout::MainPageId;

    #[test]
    fn resolve_callback_source_window_id_returns_none_for_root_shell() {
        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        assert_eq!(resolve_callback_source_window_id(&ui), None);
    }

    #[test]
    fn resolve_callback_source_window_id_reads_native_child_window_identity() {
        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        let mut host_presentation = ui.get_host_presentation();
        host_presentation.host_shell.native_floating_window_mode = true;
        host_presentation.host_shell.native_floating_window_id = "window:native-preview".into();
        ui.set_host_presentation(host_presentation);

        assert_eq!(
            resolve_callback_source_window_id(&ui),
            Some(MainPageId::new("window:native-preview"))
        );
    }
}
