use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::geometry::inset;
use super::super::render_commands::HostPaintCommand;
use super::super::theme::PALETTE;
use std::f32::consts::{PI, TAU};

const MUI_X_CHART_INSET: f32 = 8.0;
const MUI_X_CHART_MAX_RASTER_EXTENT: f32 = 192.0;
const MUI_X_CHART_LINE_WIDTH: f32 = 2.4;
const MUI_X_SPARKLINE_WIDTH: f32 = 2.0;

#[derive(Clone, Copy)]
pub(super) enum ChartKind {
    Aggregate,
    Line,
    Bar,
    Pie,
    Sparkline,
    Gauge,
}

pub(super) fn chart_kind(component_role: &str, role: &str) -> Option<ChartKind> {
    if super::matches_any_role(component_role, role, &["mui-x-line-chart", "LineChart"]) {
        Some(ChartKind::Line)
    } else if super::matches_any_role(component_role, role, &["mui-x-bar-chart", "BarChart"]) {
        Some(ChartKind::Bar)
    } else if super::matches_any_role(component_role, role, &["mui-x-pie-chart", "PieChart"]) {
        Some(ChartKind::Pie)
    } else if super::matches_any_role(component_role, role, &["mui-x-sparkline", "SparkLineChart"])
    {
        Some(ChartKind::Sparkline)
    } else if super::matches_any_role(component_role, role, &["mui-x-gauge", "Gauge"]) {
        Some(ChartKind::Gauge)
    } else if super::matches_any_role(component_role, role, &["mui-x-charts", "Charts"]) {
        Some(ChartKind::Aggregate)
    } else {
        None
    }
}

pub(super) fn push_chart(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ChartKind,
) {
    let radius = super::node_radius(node).max(4.0);
    super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::node_background(node).unwrap_or_else(|| super::chart_surface_color(node)),
        0.0,
        radius,
        opacity,
    );

    let plot = inset(rect, MUI_X_CHART_INSET);
    super::push_quad(
        commands,
        plot.clone(),
        clip,
        order + 1,
        PALETTE.surface,
        0.0,
        3.0,
        opacity,
    );

    match kind {
        ChartKind::Aggregate | ChartKind::Bar => {
            push_bar_chart(commands, &plot, clip, order, opacity)
        }
        ChartKind::Line | ChartKind::Pie | ChartKind::Sparkline | ChartKind::Gauge => {
            push_chart_raster(commands, node, &plot, clip, order + 2, opacity, kind)
        }
    }
}

fn push_bar_chart(
    commands: &mut Vec<HostPaintCommand>,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 2,
        0.18,
        0.72,
        PALETTE.accent,
        opacity,
    );
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 3,
        0.42,
        0.48,
        PALETTE.success,
        opacity,
    );
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 4,
        0.66,
        0.62,
        PALETTE.warning,
        opacity,
    );
}

fn push_chart_raster(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ChartKind,
) {
    let Some((width, height)) = chart_raster_dimensions(plot) else {
        return;
    };
    let mut raster = ChartRaster::transparent(width, height);
    match kind {
        ChartKind::Line => draw_line_chart_raster(&mut raster),
        ChartKind::Pie => draw_pie_chart_raster(&mut raster, node),
        ChartKind::Sparkline => draw_sparkline_raster(&mut raster),
        ChartKind::Gauge => draw_gauge_raster(&mut raster, chart_value(node)),
        ChartKind::Aggregate | ChartKind::Bar => return,
    }
    commands.push(HostPaintCommand::image_pixels(
        plot.clone(),
        Some(clip.clone()),
        order,
        format!("mui-x-chart:{}:{}x{}", chart_kind_name(kind), width, height),
        width,
        height,
        raster.rgba,
        None,
        opacity,
    ));
}

fn push_chart_bar(
    commands: &mut Vec<HostPaintCommand>,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    x_factor: f32,
    height_factor: f32,
    color: [u8; 4],
    opacity: f32,
) {
    let width = (plot.width * 0.13).max(1.0);
    let height = (plot.height * height_factor).max(1.0);
    super::push_quad(
        commands,
        FrameRect {
            x: plot.x + plot.width * x_factor,
            y: plot.y + plot.height - height,
            width,
            height,
        },
        clip,
        order,
        color,
        0.0,
        2.0,
        opacity,
    );
}

fn chart_raster_dimensions(plot: &FrameRect) -> Option<(u32, u32)> {
    if plot.width <= 0.0 || plot.height <= 0.0 {
        return None;
    }
    Some((
        plot.width.ceil().clamp(1.0, MUI_X_CHART_MAX_RASTER_EXTENT) as u32,
        plot.height.ceil().clamp(1.0, MUI_X_CHART_MAX_RASTER_EXTENT) as u32,
    ))
}

fn chart_kind_name(kind: ChartKind) -> &'static str {
    match kind {
        ChartKind::Aggregate => "aggregate",
        ChartKind::Line => "line",
        ChartKind::Bar => "bar",
        ChartKind::Pie => "pie",
        ChartKind::Sparkline => "sparkline",
        ChartKind::Gauge => "gauge",
    }
}

fn draw_line_chart_raster(raster: &mut ChartRaster) {
    let points = [
        (0.08, 0.78),
        (0.30, 0.38),
        (0.52, 0.52),
        (0.75, 0.24),
        (0.92, 0.44),
    ];
    raster.draw_polyline(&points, MUI_X_CHART_LINE_WIDTH, PALETTE.accent);
    raster.draw_polyline(
        &[(0.10, 0.56), (0.34, 0.62), (0.56, 0.42), (0.80, 0.50)],
        MUI_X_CHART_LINE_WIDTH * 0.72,
        PALETTE.success,
    );
    raster.draw_points(&points, 2.2, PALETTE.accent);
}

fn draw_sparkline_raster(raster: &mut ChartRaster) {
    let points = [
        (0.06, 0.72),
        (0.24, 0.38),
        (0.44, 0.58),
        (0.65, 0.31),
        (0.86, 0.46),
    ];
    raster.draw_polyline(&points, MUI_X_SPARKLINE_WIDTH, PALETTE.accent);
    raster.draw_points(&points, 1.9, PALETTE.accent);
}

fn draw_pie_chart_raster(raster: &mut ChartRaster, node: &TemplatePaneNodeData) {
    let center = raster.center();
    let radius = raster.width.min(raster.height) as f32 * 0.43;
    let hole_radius = if node.selected || node.checked {
        radius * 0.34
    } else {
        0.0
    };
    raster.draw_pie(center, radius, hole_radius);
}

fn draw_gauge_raster(raster: &mut ChartRaster, value: f32) {
    let center = (raster.width as f32 * 0.5, raster.height as f32 - 3.0);
    let radius = (raster.height as f32 - 7.0).max(4.0);
    let thickness = (raster.height as f32 * 0.12).clamp(2.0, 4.0);
    let start = PI;
    let end = TAU;
    raster.draw_arc(center, radius, thickness, start, end, PALETTE.surface_hover);
    raster.draw_arc(
        center,
        radius,
        thickness,
        start,
        start + (end - start) * value.clamp(0.0, 1.0),
        PALETTE.accent,
    );
    raster.draw_disc(center, thickness * 1.35, PALETTE.surface_hover);
}

fn chart_value(node: &TemplatePaneNodeData) -> f32 {
    if node.value_percent > 0.0 {
        node.value_percent
    } else if node.value_number > 1.0 {
        node.value_number / 100.0
    } else {
        node.value_number
    }
}

struct ChartRaster {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl ChartRaster {
    fn transparent(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            rgba: vec![0; width as usize * height as usize * 4],
        }
    }

    fn center(&self) -> (f32, f32) {
        (self.width as f32 * 0.5, self.height as f32 * 0.5)
    }

    fn draw_polyline(&mut self, points: &[(f32, f32)], width: f32, color: [u8; 4]) {
        for pair in points.windows(2) {
            let start = self.normalized_point(pair[0]);
            let end = self.normalized_point(pair[1]);
            self.draw_line(start, end, width, color);
        }
    }

    fn draw_points(&mut self, points: &[(f32, f32)], radius: f32, color: [u8; 4]) {
        for point in points {
            self.draw_disc(self.normalized_point(*point), radius, color);
        }
    }

    fn normalized_point(&self, point: (f32, f32)) -> (f32, f32) {
        (
            point.0.clamp(0.0, 1.0) * (self.width.saturating_sub(1)) as f32,
            point.1.clamp(0.0, 1.0) * (self.height.saturating_sub(1)) as f32,
        )
    }

    fn draw_line(&mut self, start: (f32, f32), end: (f32, f32), width: f32, color: [u8; 4]) {
        let radius = (width * 0.5).max(0.5);
        let min_x = start.0.min(end.0) - radius;
        let max_x = start.0.max(end.0) + radius;
        let min_y = start.1.min(end.1) - radius;
        let max_y = start.1.max(end.1) + radius;
        for y in clamp_pixel_range(min_y, max_y, self.height) {
            for x in clamp_pixel_range(min_x, max_x, self.width) {
                let point = (x as f32 + 0.5, y as f32 + 0.5);
                if distance_to_segment(point, start, end) <= radius {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }

    fn draw_disc(&mut self, center: (f32, f32), radius: f32, color: [u8; 4]) {
        let radius_sq = radius * radius;
        for y in clamp_pixel_range(center.1 - radius, center.1 + radius, self.height) {
            for x in clamp_pixel_range(center.0 - radius, center.0 + radius, self.width) {
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                if dx * dx + dy * dy <= radius_sq {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }

    fn draw_arc(
        &mut self,
        center: (f32, f32),
        radius: f32,
        thickness: f32,
        start_angle: f32,
        end_angle: f32,
        color: [u8; 4],
    ) {
        let inner = (radius - thickness * 0.5).max(0.0);
        let outer = radius + thickness * 0.5;
        for y in clamp_pixel_range(center.1 - outer, center.1 + outer, self.height) {
            for x in clamp_pixel_range(center.0 - outer, center.0 + outer, self.width) {
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                let distance = (dx * dx + dy * dy).sqrt();
                let angle = normalized_angle(dy.atan2(dx));
                if distance >= inner
                    && distance <= outer
                    && angle >= start_angle
                    && angle <= end_angle
                {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }

    fn draw_pie(&mut self, center: (f32, f32), radius: f32, hole_radius: f32) {
        let radius_sq = radius * radius;
        let hole_sq = hole_radius * hole_radius;
        for y in clamp_pixel_range(center.1 - radius, center.1 + radius, self.height) {
            for x in clamp_pixel_range(center.0 - radius, center.0 + radius, self.width) {
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                let distance_sq = dx * dx + dy * dy;
                if distance_sq > radius_sq || distance_sq < hole_sq {
                    continue;
                }
                let progress = (normalized_angle(dy.atan2(dx)) + PI * 0.5).rem_euclid(TAU) / TAU;
                let color = if progress < 0.42 {
                    PALETTE.accent
                } else if progress < 0.76 {
                    PALETTE.success
                } else {
                    PALETTE.warning
                };
                self.set_pixel(x, y, color);
            }
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let offset = ((y as usize * self.width as usize) + x as usize) * 4;
        self.rgba[offset..offset + 4].copy_from_slice(&color);
    }
}

fn clamp_pixel_range(min: f32, max: f32, extent: u32) -> std::ops::Range<u32> {
    let start = min.floor().max(0.0).min(extent as f32) as u32;
    let end = max.ceil().max(0.0).min(extent as f32) as u32;
    start..end
}

fn distance_to_segment(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let segment = (end.0 - start.0, end.1 - start.1);
    let length_sq = segment.0 * segment.0 + segment.1 * segment.1;
    if length_sq <= f32::EPSILON {
        let dx = point.0 - start.0;
        let dy = point.1 - start.1;
        return (dx * dx + dy * dy).sqrt();
    }
    let t = (((point.0 - start.0) * segment.0 + (point.1 - start.1) * segment.1) / length_sq)
        .clamp(0.0, 1.0);
    let projection = (start.0 + segment.0 * t, start.1 + segment.1 * t);
    let dx = point.0 - projection.0;
    let dy = point.1 - projection.1;
    (dx * dx + dy * dy).sqrt()
}

fn normalized_angle(angle: f32) -> f32 {
    angle.rem_euclid(TAU)
}
