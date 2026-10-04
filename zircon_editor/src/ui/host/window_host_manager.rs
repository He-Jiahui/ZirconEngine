use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

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

#[derive(Clone, Default)]
pub(super) struct WindowHostManager {
    windows: HashMap<MainPageId, NativeWindowRecord>,
}

impl fmt::Debug for WindowHostManager {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let windows = self.windows.iter().collect::<BTreeMap<_, _>>();
        formatter
            .debug_struct("WindowHostManager")
            .field("windows", &windows)
            .finish()
    }
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
        let layout_window_ids = layout
            .floating_windows
            .iter()
            .map(|window| &window.window_id)
            .collect::<HashSet<_>>();
        self.windows
            .retain(|window_id, _| layout_window_ids.contains(window_id));

        for window in &layout.floating_windows {
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
        let mut states = self
            .windows
            .iter()
            .map(|(window_id, record)| NativeWindowHostState {
                window_id: window_id.clone(),
                handle: record.handle,
                bounds: record.bounds,
                surface_tree_id: record.surface.tree.tree_id.clone(),
            })
            .collect::<Vec<_>>();
        states.sort_unstable_by(|left, right| left.window_id.cmp(&right.window_id));
        states
    }
}

fn native_window_surface_tree_id(window_id: &MainPageId) -> UiTreeId {
    UiTreeId::new(format!("zircon.editor.native_window.{}", window_id.0))
}

#[cfg(test)]
#[path = "tests/window_host_manager.rs"]
mod tests;

#[cfg(test)]
#[path = "window_host_manager/tests/hash_reconcile_tests.rs"]
mod hash_reconcile_tests;
