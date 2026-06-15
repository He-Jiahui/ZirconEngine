use std::collections::BTreeMap;

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use crate::ui::workbench::layout::{MainPageId, WorkbenchLayout};
use crate::ui::workbench::view::ViewHost;

type NativeWindowHandle = u64;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeWindowHostState {
    pub window_id: MainPageId,
    pub handle: Option<u64>,
    pub bounds: [f32; 4],
    pub surface_tree_id: UiTreeId,
}

impl NativeWindowHostState {
    #[cfg(test)]
    pub(crate) fn new_for_test(
        window_id: MainPageId,
        handle: Option<u64>,
        bounds: [f32; 4],
    ) -> Self {
        Self {
            surface_tree_id: native_window_surface_tree_id(&window_id),
            window_id,
            handle,
            bounds,
        }
    }
}

#[derive(Clone, Debug)]
struct NativeWindowRecord {
    handle: Option<NativeWindowHandle>,
    bounds: [f32; 4],
    surface: UiSurface,
}

impl NativeWindowRecord {
    fn new(window_id: &MainPageId) -> Self {
        Self {
            handle: None,
            bounds: [0.0; 4],
            surface: UiSurface::new(native_window_surface_tree_id(window_id)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct WindowHostManager {
    windows: BTreeMap<MainPageId, NativeWindowRecord>,
}

impl WindowHostManager {
    pub fn open_native_window(
        &mut self,
        window_id: MainPageId,
        handle: Option<NativeWindowHandle>,
    ) {
        let record = self
            .windows
            .entry(window_id.clone())
            .or_insert_with(|| NativeWindowRecord::new(&window_id));
        if let Some(handle) = handle {
            record.handle = Some(handle);
        }
    }

    pub fn close_native_window(&mut self, window_id: &MainPageId) {
        self.windows.remove(window_id);
    }

    pub fn sync_window_bounds(&mut self, window_id: &MainPageId, bounds: [f32; 4]) {
        self.windows
            .entry(window_id.clone())
            .or_insert_with(|| NativeWindowRecord::new(window_id))
            .bounds = bounds;
    }

    pub fn reattach_window(&mut self, window_id: &MainPageId, _drop_target: &ViewHost) {
        self.close_native_window(window_id);
    }

    pub fn sync_layout_windows(&mut self, layout: &WorkbenchLayout) {
        let tracked_window_ids = self.windows.keys().cloned().collect::<Vec<_>>();
        for window_id in tracked_window_ids {
            if !layout
                .floating_windows
                .iter()
                .any(|window| window.window_id == window_id)
            {
                self.close_native_window(&window_id);
            }
        }

        for window in &layout.floating_windows {
            self.open_native_window(window.window_id.clone(), None);
            self.sync_window_bounds(
                &window.window_id,
                [
                    window.frame.x,
                    window.frame.y,
                    window.frame.width,
                    window.frame.height,
                ],
            );
        }
    }

    pub fn states(&self) -> Vec<NativeWindowHostState> {
        self.windows
            .iter()
            .map(|(window_id, record)| NativeWindowHostState {
                window_id: window_id.clone(),
                handle: record.handle,
                bounds: record.bounds,
                surface_tree_id: record.surface.tree.tree_id.clone(),
            })
            .collect()
    }
}

fn native_window_surface_tree_id(window_id: &MainPageId) -> UiTreeId {
    UiTreeId::new(format!("zircon.editor.native_window.{}", window_id.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::workbench::autolayout::ShellFrame;
    use crate::ui::workbench::layout::{
        DocumentNode, FloatingWindowLayout, TabStackLayout, WorkbenchLayout,
    };
    use crate::ui::workbench::view::ViewInstanceId;

    #[test]
    fn native_window_hosts_allocate_independent_surfaces_per_floating_window() {
        let mut manager = WindowHostManager::default();
        let first = MainPageId::new("window:first");
        let second = MainPageId::new("window:second");

        manager.open_native_window(first.clone(), Some(11));
        manager.open_native_window(second.clone(), Some(22));

        let states = manager.states();
        let first_state = states
            .iter()
            .find(|state| state.window_id == first)
            .expect("first native window state");
        let second_state = states
            .iter()
            .find(|state| state.window_id == second)
            .expect("second native window state");

        assert_eq!(
            first_state.surface_tree_id.0,
            "zircon.editor.native_window.window:first"
        );
        assert_eq!(
            second_state.surface_tree_id.0,
            "zircon.editor.native_window.window:second"
        );
        assert_ne!(first_state.surface_tree_id, second_state.surface_tree_id);
        assert_ne!(
            manager.windows[&first].surface.tree.tree_id,
            manager.windows[&second].surface.tree.tree_id,
            "each floating window must own its own runtime UiSurface instead of sharing the main host surface"
        );
    }

    #[test]
    fn native_window_host_sync_preserves_surface_when_bounds_change() {
        let mut manager = WindowHostManager::default();
        let window_id = MainPageId::new("window:preview");

        manager.sync_layout_windows(&layout_with_floating_window(
            window_id.clone(),
            ShellFrame::new(20.0, 30.0, 320.0, 240.0),
        ));
        let surface_tree_id = manager.windows[&window_id].surface.tree.tree_id.clone();

        manager.sync_layout_windows(&layout_with_floating_window(
            window_id.clone(),
            ShellFrame::new(80.0, 90.0, 640.0, 360.0),
        ));

        assert_eq!(
            manager.windows[&window_id].surface.tree.tree_id,
            surface_tree_id
        );
        assert_eq!(
            manager.states()[0].bounds,
            [80.0, 90.0, 640.0, 360.0],
            "geometry changes should update host bounds without replacing the window-owned surface"
        );
    }

    #[test]
    fn native_window_host_sync_removes_surface_for_stale_floating_window() {
        let mut manager = WindowHostManager::default();
        let window_id = MainPageId::new("window:preview");

        manager.sync_layout_windows(&layout_with_floating_window(
            window_id,
            ShellFrame::new(20.0, 30.0, 320.0, 240.0),
        ));
        assert_eq!(manager.windows.len(), 1);

        manager.sync_layout_windows(&WorkbenchLayout::default());

        assert!(manager.windows.is_empty());
        assert!(manager.states().is_empty());
    }

    fn layout_with_floating_window(window_id: MainPageId, frame: ShellFrame) -> WorkbenchLayout {
        let view_id = ViewInstanceId::new(format!("{}#view", window_id.0));
        WorkbenchLayout {
            floating_windows: vec![FloatingWindowLayout {
                window_id,
                title: "Preview".to_string(),
                workspace: DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![view_id.clone()],
                    active_tab: Some(view_id.clone()),
                }),
                focused_view: Some(view_id),
                frame,
            }],
            ..WorkbenchLayout::default()
        }
    }
}
