use std::cell::RefCell;
use std::sync::Arc;

use crate::ui::retained_host::host_contract::paint_theme::{
    enter_host_paint_theme_scope, HostPaintThemeScope, HostPaintThemeSnapshot,
};
use crate::ui::retained_host::host_contract::surface_hit_test::HostWorkbenchHitIndex;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::{
    FrameRect, HostDockOverflowMenuStateData, HostMenuStateData, HostPageOverflowMenuStateData,
    HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageSet,
    HostWindowPresentationData, TemplatePaneNodeData,
};

/// Immutable handles for one coherent host presentation read.
#[derive(Clone)]
pub(crate) struct HostPresentationGeneration {
    structure: Arc<HostWindowPresentationData>,
    menu_state: Arc<HostMenuStateData>,
    page_overflow_menu_state: Arc<HostPageOverflowMenuStateData>,
    dock_overflow_menu_state: Arc<HostDockOverflowMenuStateData>,
    pane_interaction_state: Arc<HostPaneInteractionStateData>,
    text_input_focus: Arc<HostTextInputFocusData>,
    viewport_images: HostViewportImageSet,
    workbench_hit_index: Arc<HostWorkbenchHitIndex>,
    theme: Arc<HostPaintThemeSnapshot>,
    diagnostics_overlay_text: Arc<SharedString>,
    structure_generation: u64,
    geometry_generation: u64,
    interaction_generation: u64,
    viewport_generation: u64,
    hit_test_generation: u64,
    diagnostics_generation: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HostPresentationGenerationCursor {
    structure: u64,
    geometry: u64,
    interaction: u64,
    viewport: u64,
    hit_test: u64,
    theme: u64,
    diagnostics: u64,
}

impl HostPresentationGenerationCursor {
    pub(crate) const fn new(
        structure: u64,
        interaction: u64,
        viewport: u64,
        hit_test: u64,
        theme: u64,
        diagnostics: u64,
    ) -> Self {
        Self::new_with_geometry(
            structure,
            structure,
            interaction,
            viewport,
            hit_test,
            theme,
            diagnostics,
        )
    }

    pub(crate) const fn new_with_geometry(
        structure: u64,
        geometry: u64,
        interaction: u64,
        viewport: u64,
        hit_test: u64,
        theme: u64,
        diagnostics: u64,
    ) -> Self {
        Self {
            structure,
            geometry,
            interaction,
            viewport,
            hit_test,
            theme,
            diagnostics,
        }
    }
}

#[derive(Clone)]
struct HostPresentationPaintOverrides {
    menu_state: Arc<HostMenuStateData>,
    page_overflow_menu_state: Arc<HostPageOverflowMenuStateData>,
    dock_overflow_menu_state: Arc<HostDockOverflowMenuStateData>,
    pane_interaction_state: Arc<HostPaneInteractionStateData>,
    text_input_focus: Arc<HostTextInputFocusData>,
    viewport_images: HostViewportImageSet,
    workbench_hit_index: Arc<HostWorkbenchHitIndex>,
    diagnostics_overlay_text: Arc<SharedString>,
}

thread_local! {
    static ACTIVE_PAINT_OVERRIDES: RefCell<Option<HostPresentationPaintOverrides>> =
        const { RefCell::new(None) };
}

pub(crate) struct HostPresentationPaintScope {
    previous: Option<HostPresentationPaintOverrides>,
    _theme_scope: HostPaintThemeScope,
}

impl Drop for HostPresentationPaintScope {
    fn drop(&mut self) {
        let previous = self.previous.take();
        ACTIVE_PAINT_OVERRIDES.with(|active| *active.borrow_mut() = previous);
    }
}

impl HostPresentationGeneration {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        structure: Arc<HostWindowPresentationData>,
        menu_state: Arc<HostMenuStateData>,
        page_overflow_menu_state: Arc<HostPageOverflowMenuStateData>,
        dock_overflow_menu_state: Arc<HostDockOverflowMenuStateData>,
        pane_interaction_state: Arc<HostPaneInteractionStateData>,
        text_input_focus: Arc<HostTextInputFocusData>,
        viewport_images: HostViewportImageSet,
        workbench_hit_index: Arc<HostWorkbenchHitIndex>,
        theme: Arc<HostPaintThemeSnapshot>,
        diagnostics_overlay_text: Arc<SharedString>,
        structure_generation: u64,
        interaction_generation: u64,
        viewport_generation: u64,
        hit_test_generation: u64,
        diagnostics_generation: u64,
    ) -> Self {
        Self::new_with_geometry(
            structure,
            menu_state,
            page_overflow_menu_state,
            dock_overflow_menu_state,
            pane_interaction_state,
            text_input_focus,
            viewport_images,
            workbench_hit_index,
            theme,
            diagnostics_overlay_text,
            structure_generation,
            structure_generation,
            interaction_generation,
            viewport_generation,
            hit_test_generation,
            diagnostics_generation,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with_geometry(
        structure: Arc<HostWindowPresentationData>,
        menu_state: Arc<HostMenuStateData>,
        page_overflow_menu_state: Arc<HostPageOverflowMenuStateData>,
        dock_overflow_menu_state: Arc<HostDockOverflowMenuStateData>,
        pane_interaction_state: Arc<HostPaneInteractionStateData>,
        text_input_focus: Arc<HostTextInputFocusData>,
        viewport_images: HostViewportImageSet,
        workbench_hit_index: Arc<HostWorkbenchHitIndex>,
        theme: Arc<HostPaintThemeSnapshot>,
        diagnostics_overlay_text: Arc<SharedString>,
        structure_generation: u64,
        geometry_generation: u64,
        interaction_generation: u64,
        viewport_generation: u64,
        hit_test_generation: u64,
        diagnostics_generation: u64,
    ) -> Self {
        Self {
            structure,
            menu_state,
            page_overflow_menu_state,
            dock_overflow_menu_state,
            pane_interaction_state,
            text_input_focus,
            viewport_images,
            workbench_hit_index,
            theme,
            diagnostics_overlay_text,
            structure_generation,
            geometry_generation,
            interaction_generation,
            viewport_generation,
            hit_test_generation,
            diagnostics_generation,
        }
    }

    pub(crate) fn structure(&self) -> &HostWindowPresentationData {
        &self.structure
    }

    pub(crate) fn menu_state(&self) -> &HostMenuStateData {
        &self.menu_state
    }

    pub(crate) fn page_overflow_menu_state(&self) -> &HostPageOverflowMenuStateData {
        &self.page_overflow_menu_state
    }

    pub(crate) fn dock_overflow_menu_state(&self) -> &HostDockOverflowMenuStateData {
        &self.dock_overflow_menu_state
    }

    pub(crate) fn pane_interaction_state(&self) -> &HostPaneInteractionStateData {
        &self.pane_interaction_state
    }

    pub(crate) fn text_input_focus(&self) -> &HostTextInputFocusData {
        &self.text_input_focus
    }

    pub(crate) fn viewport_images(&self) -> &HostViewportImageSet {
        &self.viewport_images
    }

    pub(crate) fn workbench_hit_index(&self) -> &HostWorkbenchHitIndex {
        &self.workbench_hit_index
    }

    pub(crate) fn structure_generation(&self) -> u64 {
        self.structure_generation
    }

    pub(crate) fn geometry_generation(&self) -> u64 {
        self.geometry_generation
    }

    pub(crate) fn interaction_generation(&self) -> u64 {
        self.interaction_generation
    }

    pub(crate) fn viewport_generation(&self) -> u64 {
        self.viewport_generation
    }

    pub(crate) fn hit_test_generation(&self) -> u64 {
        self.hit_test_generation
    }

    pub(crate) fn theme_generation(&self) -> u64 {
        self.theme.generation()
    }

    pub(crate) fn diagnostics_generation(&self) -> u64 {
        self.diagnostics_generation
    }

    pub(crate) fn cursor(&self) -> HostPresentationGenerationCursor {
        HostPresentationGenerationCursor::new_with_geometry(
            self.structure_generation(),
            self.geometry_generation(),
            self.interaction_generation(),
            self.viewport_generation(),
            self.hit_test_generation(),
            self.theme_generation(),
            self.diagnostics_generation(),
        )
    }

    pub(crate) fn shares_structure_with(&self, other: &Self) -> bool {
        self.structure_generation == other.structure_generation
    }

    pub(crate) fn shares_theme_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.theme, &other.theme)
    }

    pub(crate) fn materialize(&self) -> HostWindowPresentationData {
        record_current_ui_perf_counter(UiPerfCounter::PresentationSnapshotReadCount, 1.0);
        let mut presentation = self.structure.as_ref().clone();
        presentation.menu_state = self.menu_state.as_ref().clone();
        presentation.host_page_overflow_menu_state = self.page_overflow_menu_state.as_ref().clone();
        presentation.host_dock_overflow_menu_state = self.dock_overflow_menu_state.as_ref().clone();
        presentation.pane_interaction_state = self.pane_interaction_state.as_ref().clone();
        presentation.text_input_focus = self.text_input_focus.as_ref().clone();
        presentation.viewport_images = self.viewport_images.clone();
        presentation.host_shell.debug_refresh_rate = self.diagnostics_overlay_text.as_ref().clone();
        presentation
    }

    pub(crate) fn enter_paint_scope(&self) -> HostPresentationPaintScope {
        let theme_scope = enter_host_paint_theme_scope(Arc::clone(&self.theme));
        let overrides = HostPresentationPaintOverrides {
            menu_state: Arc::clone(&self.menu_state),
            page_overflow_menu_state: Arc::clone(&self.page_overflow_menu_state),
            dock_overflow_menu_state: Arc::clone(&self.dock_overflow_menu_state),
            pane_interaction_state: Arc::clone(&self.pane_interaction_state),
            text_input_focus: Arc::clone(&self.text_input_focus),
            viewport_images: self.viewport_images.clone(),
            workbench_hit_index: Arc::clone(&self.workbench_hit_index),
            diagnostics_overlay_text: Arc::clone(&self.diagnostics_overlay_text),
        };
        let previous = ACTIVE_PAINT_OVERRIDES.with(|active| active.replace(Some(overrides)));
        HostPresentationPaintScope {
            previous,
            _theme_scope: theme_scope,
        }
    }
}

pub(crate) fn paint_menu_state(
    presentation: &HostWindowPresentationData,
) -> Arc<HostMenuStateData> {
    ACTIVE_PAINT_OVERRIDES
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|state| Arc::clone(&state.menu_state))
        })
        .unwrap_or_else(|| Arc::new(presentation.menu_state.clone()))
}

pub(crate) fn paint_page_overflow_menu_state(
    presentation: &HostWindowPresentationData,
) -> Arc<HostPageOverflowMenuStateData> {
    ACTIVE_PAINT_OVERRIDES
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|state| Arc::clone(&state.page_overflow_menu_state))
        })
        .unwrap_or_else(|| Arc::new(presentation.host_page_overflow_menu_state.clone()))
}

pub(crate) fn paint_dock_overflow_menu_state(
    presentation: &HostWindowPresentationData,
) -> Arc<HostDockOverflowMenuStateData> {
    ACTIVE_PAINT_OVERRIDES
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|state| Arc::clone(&state.dock_overflow_menu_state))
        })
        .unwrap_or_else(|| Arc::new(presentation.host_dock_overflow_menu_state.clone()))
}

pub(crate) fn paint_pane_interaction_state(
    presentation: &HostWindowPresentationData,
) -> Arc<HostPaneInteractionStateData> {
    ACTIVE_PAINT_OVERRIDES
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|state| Arc::clone(&state.pane_interaction_state))
        })
        .unwrap_or_else(|| Arc::new(presentation.pane_interaction_state.clone()))
}

pub(crate) fn paint_text_input_focus(
    presentation: &HostWindowPresentationData,
) -> Arc<HostTextInputFocusData> {
    ACTIVE_PAINT_OVERRIDES
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|state| Arc::clone(&state.text_input_focus))
        })
        .unwrap_or_else(|| Arc::new(presentation.text_input_focus.clone()))
}

pub(crate) fn paint_viewport_images(
    presentation: &HostWindowPresentationData,
) -> HostViewportImageSet {
    ACTIVE_PAINT_OVERRIDES
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|state| state.viewport_images.clone())
        })
        .unwrap_or_else(|| presentation.viewport_images.clone())
}

pub(crate) fn paint_debug_refresh_rate(
    presentation: &HostWindowPresentationData,
) -> Arc<SharedString> {
    ACTIVE_PAINT_OVERRIDES
        .with(|active| {
            active
                .borrow()
                .as_ref()
                .map(|state| Arc::clone(&state.diagnostics_overlay_text))
        })
        .unwrap_or_else(|| Arc::new(presentation.host_shell.debug_refresh_rate.clone()))
}

pub(crate) fn paint_workbench_hit_index(
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> Option<Arc<HostWorkbenchHitIndex>> {
    ACTIVE_PAINT_OVERRIDES.with(|active| {
        active.borrow().as_ref().and_then(|state| {
            state
                .workbench_hit_index
                .indexes_paint_nodes(nodes)
                .then(|| Arc::clone(&state.workbench_hit_index))
        })
    })
}

pub(crate) fn visit_paint_workbench_rows(
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    visit: &mut dyn FnMut(usize),
) -> bool {
    let local_clip = FrameRect {
        x: clip.x - origin.x,
        y: clip.y - origin.y,
        width: clip.width,
        height: clip.height,
    };
    let Some(index) = paint_workbench_hit_index(nodes) else {
        return false;
    };
    index.visit_paint_rows_for_nodes(nodes, &local_clip, visit)
}

#[cfg(test)]
#[path = "tests/presentation_generation.rs"]
mod tests;
