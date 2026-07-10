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
    let mut widths = BalancedSideWidths {
        left: left.max(0.0),
        right: right.max(0.0),
    };
    let mut excess = (widths.left + widths.right - side_budget.max(0.0)).max(0.0);
    if excess <= f32::EPSILON {
        return widths;
    }

    if widths.left > widths.right {
        let reduction = excess.min(widths.left - widths.right);
        widths.left -= reduction;
        excess -= reduction;
    } else if widths.right > widths.left {
        let reduction = excess.min(widths.right - widths.left);
        widths.right -= reduction;
        excess -= reduction;
    }

    if excess <= f32::EPSILON {
        return widths;
    }

    let total = widths.left + widths.right;
    if total <= f32::EPSILON {
        return widths;
    }
    let retained_total = (total - excess).max(0.0);
    widths.left = retained_total * (widths.left / total);
    widths.right = retained_total * (widths.right / total);
    widths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn larger_side_releases_width_before_balanced_panels_shrink_together() {
        assert_eq!(
            balanced_side_widths_for_budget(278.0, 186.0, 378.0),
            BalancedSideWidths {
                left: 192.0,
                right: 186.0
            }
        );
        assert_eq!(
            balanced_side_widths_for_budget(340.0, 220.0, 448.0),
            BalancedSideWidths {
                left: 228.0,
                right: 220.0
            }
        );
    }

    #[test]
    fn authored_side_widths_survive_when_the_document_budget_already_fits() {
        assert_eq!(
            balanced_side_widths_for_budget(278.0, 274.0, 558.0),
            BalancedSideWidths {
                left: 278.0,
                right: 274.0
            }
        );
    }
}
