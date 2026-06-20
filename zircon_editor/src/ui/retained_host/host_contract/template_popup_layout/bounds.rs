use super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn valid_bounds(bounds: &FrameRect) -> bool {
    bounds.x.is_finite()
        && bounds.y.is_finite()
        && bounds.width.is_finite()
        && bounds.height.is_finite()
        && bounds.width > 0.0
        && bounds.height > 0.0
}
