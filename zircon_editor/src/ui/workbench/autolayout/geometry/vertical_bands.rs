use super::super::{
    compact_bottom_defaults, fixed_axis, solve_axis_constraints, AxisConstraint,
    ResolvedAxisConstraint, ShellFrame, ShellSizePx, WorkbenchChromeMetrics,
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PriorityBandHeights {
    top: f32,
    host: f32,
    center: f32,
    bottom: f32,
    status: f32,
}

/// Resolves the workbench's fixed chrome and flexible content as one flex-column sequence.
pub(super) fn resolve_vertical_flex_bands(
    size: ShellSizePx,
    request: VerticalFlexBandRequest,
) -> VerticalFlexBands {
    let separator = finite_non_negative(request.metrics.separator_thickness);
    let minimums = priority_band_minimums(&request);
    if finite_non_negative(size.height) < solver_minimum_required_height(&request, separator) {
        return resolve_undersized_vertical_bands(size, minimums, separator);
    }

    let gap_count = 3.0 + request.bottom_constraint.is_some() as u8 as f32;
    let available_height = (size.height - gap_count * separator).max(0.0);
    let resolved = solve_vertical_band_constraints(available_height, &request);
    let mut top_height = finite_non_negative(resolved[0].resolved);
    let mut host_height = finite_non_negative(resolved[1].resolved);
    let mut status_height = resolved
        .last()
        .map(|band| finite_non_negative(band.resolved))
        .unwrap_or_default();
    let flexible_height = (available_height - top_height - host_height - status_height).max(0.0);
    let mut center_height = finite_non_negative(resolved[2].resolved);
    let mut bottom_height = 0.0;
    if request.bottom_constraint.is_some() {
        bottom_height = finite_non_negative(resolved[3].resolved);
        if let Some(compact_limit) = compact_bottom_height_limit(flexible_height) {
            bottom_height = bottom_height.min(compact_limit);
            center_height = (flexible_height - bottom_height).max(0.0);
        } else {
            center_height = center_height.min(flexible_height);
            bottom_height = bottom_height.min((flexible_height - center_height).max(0.0));
        }
    }
    fit_bands_to_budget(
        available_height,
        &mut [
            &mut top_height,
            &mut host_height,
            &mut center_height,
            &mut bottom_height,
            &mut status_height,
        ],
    );
    let mut stack = VerticalFlexBandStack::new(size.width, size.height, separator);
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

impl PriorityBandHeights {
    fn required_height(self, separator: f32) -> f32 {
        let positive_bands = [self.top, self.host, self.center, self.bottom, self.status]
            .into_iter()
            .filter(|height| *height > 0.0)
            .count();
        let gaps = positive_bands.saturating_sub(1) as f32;
        self.top + self.host + self.center + self.bottom + self.status + gaps * separator
    }
}

fn priority_band_minimums(request: &VerticalFlexBandRequest) -> PriorityBandHeights {
    PriorityBandHeights {
        top: finite_non_negative(request.metrics.top_bar_height),
        host: finite_non_negative(request.metrics.host_bar_height),
        center: finite_non_negative(request.center_constraint.resolved().min),
        bottom: request
            .bottom_constraint
            .is_some()
            .then(|| finite_non_negative(request.metrics.panel_header_height))
            .unwrap_or_default(),
        status: finite_non_negative(request.metrics.status_bar_height),
    }
}

fn solver_minimum_required_height(request: &VerticalFlexBandRequest, separator: f32) -> f32 {
    let mut minimums = priority_band_minimums(request);
    minimums.bottom = request
        .bottom_constraint
        .map(|constraint| finite_non_negative(constraint.resolved().min))
        .unwrap_or_default();
    minimums.required_height(separator)
}

fn resolve_undersized_vertical_bands(
    size: ShellSizePx,
    minimums: PriorityBandHeights,
    separator: f32,
) -> VerticalFlexBands {
    let mut remaining = finite_non_negative(size.height);
    let mut has_positive_band = false;
    let mut allocate = |target: f32| {
        let gap = if has_positive_band { separator } else { 0.0 };
        if target <= 0.0 || remaining < gap + 1.0 {
            return 0.0;
        }
        remaining -= gap;
        let allocated = target.min(remaining);
        remaining -= allocated;
        has_positive_band = true;
        allocated
    };
    let bands = PriorityBandHeights {
        top: allocate(minimums.top),
        host: allocate(minimums.host),
        center: allocate(minimums.center),
        bottom: allocate(minimums.bottom),
        status: allocate(minimums.status),
    };

    let mut stack = VerticalFlexBandStack::new(size.width, size.height, separator);
    stack.push(bands.top);
    stack.push(bands.host);
    let center_band_frame = stack.push(bands.center);
    let bottom_frame = stack.push(bands.bottom);
    let status_bar_frame = stack.push(bands.status);
    VerticalFlexBands {
        center_band_frame,
        bottom_frame,
        status_bar_frame,
    }
}

fn solve_vertical_band_constraints(
    available_height: f32,
    request: &VerticalFlexBandRequest,
) -> Vec<ResolvedAxisConstraint> {
    let top = fixed_axis(request.metrics.top_bar_height);
    let host = fixed_axis(request.metrics.host_bar_height);
    let status = fixed_axis(request.metrics.status_bar_height);
    match request.bottom_constraint {
        Some(bottom) => solve_axis_constraints(
            available_height,
            &[top, host, request.center_constraint, bottom, status],
        ),
        None => solve_axis_constraints(
            available_height,
            &[top, host, request.center_constraint, status],
        ),
    }
}

struct VerticalFlexBandStack {
    width: f32,
    height: f32,
    gap: f32,
    next_y: f32,
    has_band: bool,
}

impl VerticalFlexBandStack {
    fn new(width: f32, height: f32, gap: f32) -> Self {
        Self {
            width: finite_non_negative(width),
            height: finite_non_negative(height),
            gap: finite_non_negative(gap),
            next_y: 0.0,
            has_band: false,
        }
    }

    fn push(&mut self, height: f32) -> ShellFrame {
        let height = finite_non_negative(height);
        if height <= 0.0 {
            return ShellFrame::new(0.0, self.next_y, self.width, 0.0);
        }
        if self.has_band {
            self.next_y = (self.next_y + self.gap).min(self.height);
        }
        let height = height.min((self.height - self.next_y).max(0.0));
        let frame = ShellFrame::new(0.0, self.next_y, self.width, height);
        self.next_y = (self.next_y + height).min(self.height);
        self.has_band = height > 0.0;
        frame
    }
}

fn fit_bands_to_budget(available_height: f32, bands: &mut [&mut f32]) {
    let budget = f64::from(finite_non_negative(available_height));
    let total = bands
        .iter()
        .map(|band| f64::from(finite_non_negative(**band)))
        .sum::<f64>();
    if total <= budget || total <= f64::EPSILON {
        return;
    }
    let scale = budget / total;
    let mut remaining = budget;
    for band in bands {
        let next = (f64::from(finite_non_negative(**band)) * scale).min(remaining);
        **band = next as f32;
        remaining = (remaining - f64::from(**band)).max(0.0);
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
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
#[path = "vertical_bands/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "vertical_bands/tests/allocation_tests.rs"]
mod allocation_tests;

#[cfg(test)]
#[path = "vertical_bands/tests/priority_tests.rs"]
mod priority_tests;
