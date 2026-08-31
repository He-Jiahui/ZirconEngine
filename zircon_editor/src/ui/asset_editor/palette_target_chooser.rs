use crate::ui::asset_editor::tree::palette_drop::{
    UiAssetPaletteDragResolution, UiAssetPaletteDragTarget,
};

#[cfg(test)]
mod single_candidate_scan_tests;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UiAssetPaletteTargetChooser {
    resolution: UiAssetPaletteDragResolution,
    manual_selection: bool,
    sticky: bool,
}

impl UiAssetPaletteTargetChooser {
    pub fn new(
        resolution: UiAssetPaletteDragResolution,
        manual_selection: bool,
        sticky: bool,
    ) -> Self {
        Self {
            resolution,
            manual_selection,
            sticky,
        }
    }

    pub fn resolution(&self) -> &UiAssetPaletteDragResolution {
        &self.resolution
    }

    pub fn resolution_mut(&mut self) -> &mut UiAssetPaletteDragResolution {
        &mut self.resolution
    }

    pub fn manual_selection(&self) -> bool {
        self.manual_selection
    }

    pub fn set_manual_selection(&mut self, value: bool) {
        self.manual_selection = value;
    }

    pub fn sticky(&self) -> bool {
        self.sticky
    }

    pub fn arm_sticky(&mut self) -> bool {
        if self.sticky
            || self.manual_selection
            || !self.resolution.requires_confirmation
            || self.resolution.candidates.len() <= 1
        {
            return false;
        }
        self.sticky = true;
        true
    }

    pub fn selected_target(&self) -> Option<&UiAssetPaletteDragTarget> {
        self.resolution.selected_target()
    }

    pub fn select_candidate(&mut self, index: usize) -> bool {
        if index >= self.resolution.candidates.len() || self.resolution.selected_index == index {
            return false;
        }
        self.resolution.selected_index = index;
        self.manual_selection = true;
        true
    }
}

pub(super) fn reconcile_palette_target_chooser(
    previous: Option<UiAssetPaletteTargetChooser>,
    mut next_resolution: Option<UiAssetPaletteDragResolution>,
) -> (Option<UiAssetPaletteTargetChooser>, bool) {
    let same_candidates = previous
        .as_ref()
        .zip(next_resolution.as_ref())
        .is_some_and(|(previous, next)| same_candidate_set(previous.resolution(), next));
    if let Some(previous_ref) = previous.as_ref() {
        if previous_ref.sticky() {
            if next_resolution.is_none() {
                return (previous, false);
            }
            if !same_candidates {
                return (previous, false);
            }
        }
    }

    let mut next_manual_selection = false;
    if let (Some(previous_ref), Some(next_resolution_ref)) =
        (previous.as_ref(), next_resolution.as_mut())
    {
        if same_candidates
            && previous_ref.manual_selection()
            && previous_ref.selected_target().is_some()
        {
            next_resolution_ref.selected_index = previous_ref.resolution().selected_index;
            next_manual_selection = true;
        }
    }

    let next = next_resolution.map(|resolution| {
        UiAssetPaletteTargetChooser::new(
            resolution,
            next_manual_selection,
            previous
                .as_ref()
                .map(UiAssetPaletteTargetChooser::sticky)
                .unwrap_or(false),
        )
    });
    let changed = previous.as_ref() != next.as_ref();
    (next, changed)
}

fn same_candidate_set(
    left: &UiAssetPaletteDragResolution,
    right: &UiAssetPaletteDragResolution,
) -> bool {
    left.candidates.len() == right.candidates.len()
        && left
            .candidates
            .iter()
            .zip(right.candidates.iter())
            .all(|(left, right)| {
                left.preview_index == right.preview_index
                    && left.plan.node_id == right.plan.node_id
                    && left.plan.mode == right.plan.mode
                    && left.plan.placement == right.plan.placement
                    && left.key == right.key
                    && left.detail == right.detail
            })
}
