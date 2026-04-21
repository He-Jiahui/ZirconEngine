use std::collections::BTreeMap;

use crate::ui::workbench::layout::{MainPageId, WorkbenchLayout};
use crate::ui::workbench::view::ViewHost;

type NativeWindowHandle = u64;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeWindowHostState {
    pub window_id: MainPageId,
    pub handle: Option<u64>,
    pub bounds: [f32; 4],
}

#[derive(Clone, Debug, Default)]
struct NativeWindowRecord {
    handle: Option<NativeWindowHandle>,
    bounds: [f32; 4],
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
        let record = self.windows.entry(window_id).or_default();
        if let Some(handle) = handle {
            record.handle = Some(handle);
        }
    }

    pub fn close_native_window(&mut self, window_id: &MainPageId) {
        self.windows.remove(window_id);
    }

    pub fn sync_window_bounds(&mut self, window_id: &MainPageId, bounds: [f32; 4]) {
        self.windows.entry(window_id.clone()).or_default().bounds = bounds;
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
            })
            .collect()
    }
}
