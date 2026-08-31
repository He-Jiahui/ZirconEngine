use std::collections::{BTreeMap, BTreeSet};

use crate::ui::retained_host::host_contract::data::{
    HostPresentationGenerationCursor, SceneViewportChromeData, UiAssetEditorPaneData,
};
use crate::ui::retained_host::primitives::{CloseRequestResponse, PlatformError};
use crate::ui::retained_host::ui::patch_ui_asset_presentation;
use crate::ui::workbench::layout::MainPageId;

use super::super::UiHostWindow;
use super::target::NativeFloatingWindowTarget;

#[derive(Default)]
pub(crate) struct NativeWindowPresenterStore {
    windows: BTreeMap<MainPageId, UiHostWindow>,
    applied_generations: BTreeMap<MainPageId, NativeWindowAppliedGeneration>,
    presented_rows: BTreeMap<MainPageId, usize>,
}

struct NativeWindowAppliedGeneration {
    target: NativeFloatingWindowTarget,
    source: HostPresentationGenerationCursor,
}

#[derive(Default)]
pub(crate) struct NativePresentationPatch {
    pub(crate) presenter_ids: BTreeSet<MainPageId>,
    pub(crate) presenter_visit_count: usize,
    pub(crate) floating_window_rows_visited: usize,
    pub(crate) floating_window_rows_cloned: usize,
    pub(crate) damage_region_count: usize,
}

impl NativeWindowPresenterStore {
    pub(crate) fn sync_targets<C, F>(
        &mut self,
        targets: &[NativeFloatingWindowTarget],
        on_create: C,
        apply: F,
    ) -> Result<(), PlatformError>
    where
        C: FnMut(&UiHostWindow, &NativeFloatingWindowTarget),
        F: FnMut(&UiHostWindow, &NativeFloatingWindowTarget),
    {
        self.sync_targets_inner(targets, None, on_create, apply)
    }

    pub(crate) fn sync_targets_with_generation<C, F>(
        &mut self,
        targets: &[NativeFloatingWindowTarget],
        source_generation: HostPresentationGenerationCursor,
        on_create: C,
        apply: F,
    ) -> Result<(), PlatformError>
    where
        C: FnMut(&UiHostWindow, &NativeFloatingWindowTarget),
        F: FnMut(&UiHostWindow, &NativeFloatingWindowTarget),
    {
        self.sync_targets_inner(targets, Some(source_generation), on_create, apply)
    }

    fn sync_targets_inner<C, F>(
        &mut self,
        targets: &[NativeFloatingWindowTarget],
        source_generation: Option<HostPresentationGenerationCursor>,
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
                self.applied_generations.remove(&window_id);
                self.presented_rows.remove(&window_id);
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
            let already_applied = source_generation.is_some_and(|source| {
                self.applied_generations
                    .get(&target.window_id)
                    .is_some_and(|applied| applied.source == source && applied.target == *target)
                    && self.presented_rows.contains_key(&target.window_id)
            });
            if already_applied {
                continue;
            }

            apply(window, target);
            let presented_row = window
                .get_host_presentation_generation()
                .structure()
                .native_floating_surface_data
                .floating_windows
                .iter()
                .position(|presented| presented.window_id.as_str() == target.window_id.0.as_str());
            if let Some(row) = presented_row {
                self.presented_rows.insert(target.window_id.clone(), row);
            } else {
                self.presented_rows.remove(&target.window_id);
            }
            if let Some(source) = source_generation {
                self.applied_generations.insert(
                    target.window_id.clone(),
                    NativeWindowAppliedGeneration {
                        target: target.clone(),
                        source,
                    },
                );
            } else {
                self.applied_generations.remove(&target.window_id);
            }
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

    /// Patches viewport chrome in each independently presented native child window.
    pub(crate) fn patch_scene_viewport_chrome(
        &self,
        viewport: &SceneViewportChromeData,
    ) -> NativePresentationPatch {
        let mut result = NativePresentationPatch {
            presenter_visit_count: self.windows.len(),
            ..NativePresentationPatch::default()
        };
        for (window_id, window) in &self.windows {
            let Some(&row) = self.presented_rows.get(window_id) else {
                continue;
            };
            if window
                .get_host_presentation_generation()
                .structure()
                .native_floating_surface_data
                .native_floating_window_id
                .as_str()
                != window_id.0.as_str()
            {
                continue;
            }
            if !window.patch_native_scene_viewport_chrome(
                row,
                window_id.0.as_str(),
                viewport.clone(),
            ) {
                continue;
            }
            let damage = window
                .native_viewport_chrome_damage_frame()
                .unwrap_or_else(|| window.get_host_window_bootstrap().shell_frame);
            window.request_frame_update_region(damage);
            result.damage_region_count += 1;
            result.presenter_ids.insert(window_id.clone());
        }
        result
    }

    /// Patches the already-presented UI Asset pane in every native child window.
    ///
    /// Native presenters own independent host presentations, so they cannot rely on the
    /// main window's projection patch or damage queue.
    pub(crate) fn patch_ui_asset_presentation(
        &self,
        expected_presenter_ids: &BTreeSet<MainPageId>,
        instance_id: &str,
        ui_asset: &UiAssetEditorPaneData,
    ) -> NativePresentationPatch {
        let mut result = NativePresentationPatch {
            presenter_visit_count: self.windows.len(),
            ..NativePresentationPatch::default()
        };
        for (window_id, window) in &self.windows {
            if !expected_presenter_ids.contains(window_id) {
                continue;
            }
            if window
                .get_host_presentation_generation()
                .structure()
                .native_floating_surface_data
                .native_floating_window_id
                .as_str()
                != window_id.0.as_str()
            {
                continue;
            }
            let patch = patch_ui_asset_presentation(window, instance_id, ui_asset);
            result.floating_window_rows_visited += patch.floating_window_rows_visited;
            result.floating_window_rows_cloned += patch.floating_window_rows_cloned;
            if patch.damage.is_empty() {
                continue;
            }
            result.damage_region_count += patch.damage.len();
            for frame in patch.damage {
                window.request_frame_update_region(frame);
            }
            result.presenter_ids.insert(window_id.clone());
        }
        result
    }
}
