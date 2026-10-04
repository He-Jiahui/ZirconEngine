#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BalancedSideWidths {
    pub(crate) left: f32,
    pub(crate) right: f32,
}

pub(crate) fn balanced_side_widths_for_budget(
    left: f32,
    right: f32,
    side_budget: f32,
) -> BalancedSideWidths {
    let budget = finite_non_negative(side_budget) as f64;
    let mut widths = BalancedSideWidths {
        left: finite_demand_within_budget(left, budget),
        right: finite_demand_within_budget(right, budget),
    };
    let left = widths.left as f64;
    let right = widths.right as f64;
    if left + right <= budget {
        return widths;
    }

    let (left, right) = if left > right && budget >= right * 2.0 {
        (budget - right, right)
    } else if right > left && budget >= left * 2.0 {
        (left, budget - left)
    } else {
        let left = budget * 0.5;
        (left, budget - left)
    };

    widths.left = left.min(budget) as f32;
    widths.right = right.min((budget - f64::from(widths.left)).max(0.0)) as f32;
    widths
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_demand_within_budget(value: f32, budget: f64) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else if value.is_sign_positive() {
        budget as f32
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/side_width_allocation.rs"]
mod tests;
