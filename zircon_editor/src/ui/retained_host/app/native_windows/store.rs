use std::collections::{BTreeMap, BTreeSet};

use crate::ui::retained_host::primitives::{CloseRequestResponse, PlatformError};
use crate::ui::workbench::layout::MainPageId;

use super::super::UiHostWindow;
use super::target::NativeFloatingWindowTarget;

#[derive(Default)]
pub(crate) struct NativeWindowPresenterStore {
    windows: BTreeMap<MainPageId, UiHostWindow>,
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
