use std::collections::{BTreeMap, BTreeSet};

use slint::{CloseRequestResponse, PhysicalPosition, PhysicalSize, PlatformError};

use crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::{FrameRect, UiHostWindow};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativeFloatingWindowTarget {
    pub window_id: MainPageId,
    pub title: String,
    pub bounds: [f32; 4],
}

#[derive(Default)]
pub(crate) struct NativeWindowPresenterStore {
    windows: BTreeMap<MainPageId, UiHostWindow>,
}

pub(crate) fn collect_native_floating_window_targets(
    model: &WorkbenchViewModel,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> Vec<NativeFloatingWindowTarget> {
    model
        .floating_windows
        .iter()
        .filter_map(|window| {
            floating_window_projection_bundle
                .frames(&window.window_id)
                .filter(|frames| frames.native_host_present)
                .map(|frames| {
                    let frame = frames.outer_frame;
                    let bounds = [frame.x, frame.y, frame.width, frame.height];
                    NativeFloatingWindowTarget {
                        window_id: window.window_id.clone(),
                        title: window.title.clone(),
                        bounds,
                    }
                })
        })
        .collect()
}

pub(crate) fn configure_native_floating_window_presentation(
    ui: &UiHostWindow,
    target: &NativeFloatingWindowTarget,
) {
    let mut host_presentation = ui.get_host_presentation();
    host_presentation.host_shell.native_floating_window_mode = true;
    host_presentation.host_shell.native_floating_window_id = target.window_id.0.clone().into();
    host_presentation.host_shell.native_window_title = target.title.clone().into();
    host_presentation.host_shell.native_window_bounds = FrameRect {
        x: target.bounds[0],
        y: target.bounds[1],
        width: target.bounds[2],
        height: target.bounds[3],
    };
    ui.set_host_presentation(host_presentation);
    ui.window().set_position(PhysicalPosition::new(
        target.bounds[0].round() as i32,
        target.bounds[1].round() as i32,
    ));
    ui.window().set_size(PhysicalSize::new(
        target.bounds[2].max(1.0).round() as u32,
        target.bounds[3].max(1.0).round() as u32,
    ));
}

impl NativeWindowPresenterStore {
    pub(crate) fn sync_targets<C, F>(
        &mut self,
        targets: &[NativeFloatingWindowTarget],
        mut on_create: C,
        mut apply: F,
    ) -> Result<(), PlatformError>
    where
        C: FnMut(&UiHostWindow, &NativeFloatingWindowTarget),
        F: FnMut(&UiHostWindow, &NativeFloatingWindowTarget),
    {
        let target_ids = targets
            .iter()
            .map(|target| target.window_id.clone())
            .collect::<BTreeSet<_>>();
        let stale = self
            .windows
            .keys()
            .filter(|window_id| !target_ids.contains(*window_id))
            .cloned()
            .collect::<Vec<_>>();
        for window_id in stale {
            if let Some(window) = self.windows.remove(&window_id) {
                window.hide()?;
            }
        }

        for target in targets {
            if !self.windows.contains_key(&target.window_id) {
                let window = UiHostWindow::new()?;
                window
                    .window()
                    .on_close_requested(|| CloseRequestResponse::KeepWindowShown);
                on_create(&window, target);
                window.show()?;
                self.windows.insert(target.window_id.clone(), window);
            }
            let window = self
                .windows
                .get(&target.window_id)
                .expect("window presenter should exist after creation");
            apply(window, target);
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn window_ids(&self) -> Vec<MainPageId> {
        self.windows.keys().cloned().collect()
    }

    pub(crate) fn window(&self, window_id: &MainPageId) -> Option<UiHostWindow> {
        self.windows.get(window_id).map(UiHostWindow::clone_strong)
    }
}
