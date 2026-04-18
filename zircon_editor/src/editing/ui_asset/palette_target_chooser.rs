use crate::editing::ui_asset::tree::palette_drop::{
    UiAssetPaletteDragResolution, UiAssetPaletteDragTarget,
};

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
    previous: Option<&UiAssetPaletteTargetChooser>,
    mut next_resolution: Option<UiAssetPaletteDragResolution>,
) -> Option<UiAssetPaletteTargetChooser> {
    if let Some(previous) = previous {
        if previous.sticky() {
            let Some(next_resolution_ref) = next_resolution.as_mut() else {
                return Some(previous.clone());
            };
            if !same_candidate_set(previous.resolution(), next_resolution_ref) {
                return Some(previous.clone());
            }
        }
    }

    let mut next_manual_selection = false;
    if let (Some(previous), Some(next_resolution_ref)) = (previous, next_resolution.as_mut()) {
        if same_candidate_set(previous.resolution(), next_resolution_ref)
            && previous.manual_selection()
        {
            if let Some(previous_target) = previous.selected_target() {
                if let Some(index) = next_resolution_ref.candidates.iter().position(|candidate| {
                    candidate.key == previous_target.key
                        && candidate.detail == previous_target.detail
                }) {
                    next_resolution_ref.selected_index = index;
                    next_manual_selection = true;
                }
            }
        }
    }

    next_resolution.map(|resolution| {
        UiAssetPaletteTargetChooser::new(
            resolution,
            next_manual_selection,
            previous
                .map(UiAssetPaletteTargetChooser::sticky)
                .unwrap_or(false),
        )
    })
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
