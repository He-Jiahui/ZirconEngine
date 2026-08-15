use super::super::{
    AxisConstraint, ShellFrame, ShellSizePx, WorkbenchChromeMetrics, compact_bottom_defaults,
    fixed_axis, solve_axis_constraints,
};

/// Inputs for the shell's flex-column bands, expressed entirely in logical layout units.
pub(super) struct VerticalFlexBandRequest {
    center_constraint: AxisConstraint,
    bottom_constraint: Option<AxisConstraint>,
    metrics: WorkbenchChromeMetrics,
}

impl VerticalFlexBandRequest {
    pub(super) fn new(
        center_constraint: AxisConstraint,
        bottom_constraint: Option<AxisConstraint>,
        metrics: WorkbenchChromeMetrics,
    ) -> Self {
        Self {
            center_constraint,
            bottom_constraint,
            metrics,
        }
    }
}

pub(super) struct VerticalFlexBands {
    pub(super) center_band_frame: ShellFrame,
    pub(super) bottom_frame: ShellFrame,
    pub(super) status_bar_frame: ShellFrame,
}

/// Resolves the workbench's fixed chrome and flexible content as one flex-column sequence.
pub(super) fn resolve_vertical_flex_bands(
    size: ShellSizePx,
    request: VerticalFlexBandRequest,
) -> VerticalFlexBands {
    let mut constraints = vec![
        fixed_axis(request.metrics.top_bar_height),
        fixed_axis(request.metrics.host_bar_height),
        request.center_constraint,
    ];
    if let Some(bottom_constraint) = request.bottom_constraint {
        constraints.push(bottom_constraint);
    }
    constraints.push(fixed_axis(request.metrics.status_bar_height));

    let gap_count = constraints.len().saturating_sub(1) as f32;
    let available_height = (size.height - gap_count * request.metrics.separator_thickness).max(0.0);
    let resolved = solve_axis_constraints(available_height, &constraints);
    let top_height = resolved[0].resolved;
    let host_height = resolved[1].resolved;
    let status_height = resolved
        .last()
        .map(|band| band.resolved)
        .unwrap_or_default();
    let flexible_height = (available_height - top_height - host_height - status_height).max(0.0);
    let mut center_height = resolved[2].resolved;
    let mut bottom_height = 0.0;
    if request.bottom_constraint.is_some() {
        bottom_height = resolved[3].resolved;
        if let Some(compact_limit) = compact_bottom_height_limit(flexible_height) {
            bottom_height = bottom_height.min(compact_limit);
            center_height = (flexible_height - bottom_height).max(0.0);
        }
    }

    let mut stack = VerticalFlexBandStack::new(size.width, request.metrics.separator_thickness);
    stack.push(top_height);
    stack.push(host_height);
    let center_band_frame = stack.push(center_height);
    let bottom_frame = request
        .bottom_constraint
        .is_some()
        .then(|| stack.push(bottom_height))
        .unwrap_or_default();
    let status_bar_frame = stack.push(status_height);

    VerticalFlexBands {
        center_band_frame,
        bottom_frame,
        status_bar_frame,
    }
}

struct VerticalFlexBandStack {
    width: f32,
    gap: f32,
    next_y: f32,
    has_band: bool,
}

impl VerticalFlexBandStack {
    fn new(width: f32, gap: f32) -> Self {
        Self {
            width,
            gap,
            next_y: 0.0,
            has_band: false,
        }
    }

    fn push(&mut self, height: f32) -> ShellFrame {
        if self.has_band {
            self.next_y += self.gap;
        }
        let frame = ShellFrame::new(0.0, self.next_y, self.width, height);
        self.next_y += height;
        self.has_band = true;
        frame
    }
}

pub(crate) fn compact_bottom_height_limit(available_height: f32) -> Option<f32> {
    let defaults = compact_bottom_defaults();
    if available_height <= defaults.ultra_available_height {
        return Some(
            (available_height * defaults.ultra_max_available_fraction)
                .min(defaults.ultra_max_height)
                .max(defaults.ultra_min_height),
        );
    }

    (available_height <= defaults.available_height).then(|| {
        (available_height * defaults.max_available_fraction)
            .min(defaults.max_height)
            .max(defaults.min_height)
    })
}

#[cfg(test)]
#[path = "vertical_bands/tests.rs"]
mod tests;
