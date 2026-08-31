use crate::text::{TextLayoutGeometryBudget, TextLayoutGeometryViolation};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiResolvedTextBox, UiResolvedTextLayout},
};

pub(super) fn validate_resolved_size_geometry(
    size: UiSize,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    budget.admit_axis_extent(size.width)?;
    budget.admit_axis_extent(size.height)?;
    Ok(())
}

pub(super) fn validate_resolved_layout_geometry(
    layout: &UiResolvedTextLayout,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    budget.admit_axis_extent(layout.measured_width)?;
    budget.admit_axis_extent(layout.measured_height)?;
    budget.admit_axis_extent(layout.font_size)?;
    budget.admit_axis_extent(layout.line_height)?;
    for line in &layout.lines {
        validate_frame(line.frame, budget)?;
        validate_frame(line.placement_frame, budget)?;
        budget.admit_axis_extent(line.measured_width)?;
        budget.admit_coordinate(line.baseline)?;
        for advance in &line.glyph_advances {
            budget.admit_axis_extent(*advance)?;
        }
    }
    validate_resolved_text_boxes_geometry(&layout.boxes, budget)
}

pub(super) fn validate_resolved_text_boxes_geometry(
    boxes: &[UiResolvedTextBox],
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    for text_box in boxes {
        validate_frame(text_box.frame, budget)?;
        budget.admit_axis_extent(text_box.border_width)?;
    }
    Ok(())
}

fn validate_frame(
    frame: UiFrame,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    budget.admit_coordinate(frame.x)?;
    budget.admit_coordinate(frame.y)?;
    budget.admit_axis_extent(frame.width)?;
    budget.admit_axis_extent(frame.height)?;
    budget.admit_coordinate(frame.right())?;
    budget.admit_coordinate(frame.bottom())?;
    Ok(())
}
