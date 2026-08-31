use zircon_runtime_interface::math::{Transform, UVec2, Vec3, Vec4};

use crate::scene::viewport::projection::ViewportProjectionContext;
use crate::scene::viewport::{
    GizmoAxis, HandleElementExtract, HandleScreenLine, OverlayAxis, ViewportCameraSnapshot,
};

use super::SceneViewportController;

const HANDLE_LINE_WIDTH_PX: f32 = 2.0;
const HOVERED_HANDLE_LINE_WIDTH_PX: f32 = 3.5;

impl SceneViewportController {
    pub(crate) fn handle_screen_lines_for_transform(
        &self,
        selected: Option<(u64, Transform)>,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> Vec<HandleScreenLine> {
        if viewport.x == 0 || viewport.y == 0 {
            return Vec::new();
        }
        let overlays = self.handle_overlays_for_transform(selected, camera);
        let projection = ViewportProjectionContext::new(camera, viewport);
        let mut lines = Vec::new();
        for element in overlays.iter().flat_map(|overlay| &overlay.elements) {
            append_element_lines(
                &mut lines,
                element,
                camera,
                &projection,
                self.hovered_axis(),
            );
        }
        lines.retain(|line| line.is_finite());
        lines
    }
}

fn append_element_lines(
    lines: &mut Vec<HandleScreenLine>,
    element: &HandleElementExtract,
    camera: &ViewportCameraSnapshot,
    projection: &ViewportProjectionContext,
    hovered_axis: Option<GizmoAxis>,
) {
    match element {
        HandleElementExtract::AxisLine {
            axis,
            start,
            end,
            color,
            ..
        } => {
            let axis = gizmo_axis(*axis);
            push_projected_line(
                lines,
                *start,
                *end,
                *color,
                Some(axis),
                hovered_axis,
                projection,
            );
            append_arrow_head(lines, *start, *end, *color, axis, hovered_axis, projection);
        }
        HandleElementExtract::AxisRing {
            axis,
            center,
            normal,
            radius,
            color,
            ..
        } => {
            let axis = gizmo_axis(*axis);
            let width = line_width(Some(axis), hovered_axis);
            lines.extend(
                crate::scene::viewport::pointer::projected_ring_segments(
                    *center, *normal, *radius, projection,
                )
                .into_iter()
                .map(|(start, end)| HandleScreenLine::new(start, end, *color, width, Some(axis))),
            );
        }
        HandleElementExtract::AxisScale {
            axis,
            start,
            end,
            color,
            handle_size,
            ..
        } => {
            let axis = gizmo_axis(*axis);
            push_projected_line(
                lines,
                *start,
                *end,
                *color,
                Some(axis),
                hovered_axis,
                projection,
            );
            append_cross(
                lines,
                *end,
                *handle_size,
                *color,
                Some(axis),
                hovered_axis,
                camera.transform.right(),
                camera.transform.up(),
                projection,
            );
        }
        HandleElementExtract::CenterAnchor {
            position,
            size,
            color,
        } => append_cross(
            lines,
            *position,
            *size,
            *color,
            None,
            hovered_axis,
            camera.transform.right(),
            camera.transform.up(),
            projection,
        ),
    }
}

fn append_arrow_head(
    lines: &mut Vec<HandleScreenLine>,
    start: Vec3,
    end: Vec3,
    color: Vec4,
    axis: GizmoAxis,
    hovered_axis: Option<GizmoAxis>,
    projection: &ViewportProjectionContext,
) {
    let forward = (end - start).normalize_or_zero();
    if forward.length_squared() <= f32::EPSILON {
        return;
    }
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    let right = if right.length_squared() <= f32::EPSILON {
        forward.cross(Vec3::X).normalize_or_zero()
    } else {
        right
    };
    let head_length = (end - start).length() * 0.18;
    let head_width = head_length * 0.55;
    let base = end - forward * head_length;
    push_projected_line(
        lines,
        end,
        base + right * head_width,
        color,
        Some(axis),
        hovered_axis,
        projection,
    );
    push_projected_line(
        lines,
        end,
        base - right * head_width,
        color,
        Some(axis),
        hovered_axis,
        projection,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_cross(
    lines: &mut Vec<HandleScreenLine>,
    position: Vec3,
    size: f32,
    color: Vec4,
    axis: Option<GizmoAxis>,
    hovered_axis: Option<GizmoAxis>,
    right: Vec3,
    up: Vec3,
    projection: &ViewportProjectionContext,
) {
    push_projected_line(
        lines,
        position - right * size,
        position + right * size,
        color,
        axis,
        hovered_axis,
        projection,
    );
    push_projected_line(
        lines,
        position - up * size,
        position + up * size,
        color,
        axis,
        hovered_axis,
        projection,
    );
}

fn push_projected_line(
    lines: &mut Vec<HandleScreenLine>,
    start: Vec3,
    end: Vec3,
    color: Vec4,
    axis: Option<GizmoAxis>,
    hovered_axis: Option<GizmoAxis>,
    projection: &ViewportProjectionContext,
) {
    let (Some(start), Some(end)) = (
        projection.projected_point(start),
        projection.projected_point(end),
    ) else {
        return;
    };
    lines.push(HandleScreenLine::new(
        start.position,
        end.position,
        color,
        line_width(axis, hovered_axis),
        axis,
    ));
}

fn line_width(axis: Option<GizmoAxis>, hovered_axis: Option<GizmoAxis>) -> f32 {
    if axis.is_some() && axis == hovered_axis {
        HOVERED_HANDLE_LINE_WIDTH_PX
    } else {
        HANDLE_LINE_WIDTH_PX
    }
}

fn gizmo_axis(axis: OverlayAxis) -> GizmoAxis {
    match axis {
        OverlayAxis::X => GizmoAxis::X,
        OverlayAxis::Y => GizmoAxis::Y,
        OverlayAxis::Z => GizmoAxis::Z,
    }
}
