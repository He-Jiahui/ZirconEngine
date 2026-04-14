//! Responsive shell auto-layout for the Slint workbench host.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::layout::{ActivityDrawerMode, ActivityDrawerSlot, WorkbenchLayout};
use crate::snapshot::{EditorChromeSnapshot, ViewContentKind};
use crate::view::{ViewDescriptor, ViewDescriptorId};
use crate::workbench::model::{DocumentTabModel, PaneTabModel, WorkbenchViewModel};

const EPSILON: f32 = 0.001;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StretchMode {
    Fixed,
    #[default]
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AxisConstraint {
    pub min: f32,
    pub max: f32,
    pub preferred: f32,
    pub priority: i32,
    pub weight: f32,
    pub stretch_mode: StretchMode,
}

impl Default for AxisConstraint {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: -1.0,
            preferred: 0.0,
            priority: 0,
            weight: 1.0,
            stretch_mode: StretchMode::Stretch,
        }
    }
}

impl AxisConstraint {
    fn resolved(self) -> ResolvedAxisConstraint {
        let min = self.min.max(0.0);
        let max = if self.max < 0.0 {
            None
        } else {
            Some(self.max.max(min))
        };
        let preferred = clamp_axis_value(self.preferred.max(0.0), min, max);
        ResolvedAxisConstraint {
            min,
            max,
            preferred,
            priority: self.priority,
            weight: if self.weight <= 0.0 { 1.0 } else { self.weight },
            stretch_mode: self.stretch_mode,
            resolved: preferred,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedAxisConstraint {
    pub min: f32,
    pub max: Option<f32>,
    pub preferred: f32,
    pub priority: i32,
    pub weight: f32,
    pub stretch_mode: StretchMode,
    pub resolved: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AxisConstraintOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stretch_mode: Option<StretchMode>,
}

impl AxisConstraintOverride {
    pub fn apply_to(self, base: AxisConstraint) -> AxisConstraint {
        AxisConstraint {
            min: self.min.unwrap_or(base.min),
            max: self.max.unwrap_or(base.max),
            preferred: self.preferred.unwrap_or(base.preferred),
            priority: self.priority.unwrap_or(base.priority),
            weight: self.weight.unwrap_or(base.weight),
            stretch_mode: self.stretch_mode.unwrap_or(base.stretch_mode),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min.is_none()
            && self.max.is_none()
            && self.preferred.is_none()
            && self.priority.is_none()
            && self.weight.is_none()
            && self.stretch_mode.is_none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PaneConstraints {
    #[serde(default)]
    pub width: AxisConstraint,
    #[serde(default)]
    pub height: AxisConstraint,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PaneConstraintOverride {
    #[serde(default)]
    pub width: AxisConstraintOverride,
    #[serde(default)]
    pub height: AxisConstraintOverride,
}

impl PaneConstraintOverride {
    pub fn is_empty(&self) -> bool {
        self.width.is_empty() && self.height.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ShellRegionId {
    Left,
    Document,
    Right,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ShellSizePx {
    pub width: f32,
    pub height: f32,
}

impl ShellSizePx {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ShellFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ShellFrame {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkbenchChromeMetrics {
    pub top_bar_height: f32,
    pub host_bar_height: f32,
    pub status_bar_height: f32,
    pub panel_header_height: f32,
    pub document_header_height: f32,
    pub viewport_toolbar_height: f32,
    pub rail_width: f32,
    pub separator_thickness: f32,
    pub splitter_hit_size: f32,
}

impl Default for WorkbenchChromeMetrics {
    fn default() -> Self {
        Self {
            top_bar_height: 25.0,
            host_bar_height: 24.0,
            status_bar_height: 20.0,
            panel_header_height: 25.0,
            document_header_height: 31.0,
            viewport_toolbar_height: 28.0,
            rail_width: 34.0,
            separator_thickness: 1.0,
            splitter_hit_size: 8.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkbenchShellGeometry {
    pub window_min_width: f32,
    pub window_min_height: f32,
    pub center_band_frame: ShellFrame,
    pub status_bar_frame: ShellFrame,
    pub region_frames: BTreeMap<ShellRegionId, ShellFrame>,
    pub splitter_frames: BTreeMap<ShellRegionId, ShellFrame>,
    pub viewport_content_frame: ShellFrame,
}

impl WorkbenchShellGeometry {
    pub fn region_frame(&self, region: ShellRegionId) -> ShellFrame {
        self.region_frames.get(&region).copied().unwrap_or_default()
    }

    pub fn splitter_frame(&self, region: ShellRegionId) -> ShellFrame {
        self.splitter_frames
            .get(&region)
            .copied()
            .unwrap_or_default()
    }
}

pub fn solve_axis_constraints(
    available: f32,
    constraints: &[AxisConstraint],
) -> Vec<ResolvedAxisConstraint> {
    let available = available.max(0.0);
    let mut resolved: Vec<_> = constraints
        .iter()
        .copied()
        .map(AxisConstraint::resolved)
        .collect();
    let mut total: f32 = resolved.iter().map(|axis| axis.resolved).sum();

    if total + EPSILON < available {
        let priorities = priorities_descending(&resolved, |axis| {
            axis.stretch_mode == StretchMode::Stretch
                && axis.max.is_none_or(|max| axis.resolved + EPSILON < max)
        });
        let mut remaining = available - total;
        for priority in priorities {
            if remaining <= EPSILON {
                break;
            }
            remaining = distribute_growth(&mut resolved, priority, remaining);
        }
    } else if total > available + EPSILON {
        let priorities = priorities_ascending(&resolved, |axis| axis.resolved > axis.min + EPSILON);
        let mut deficit = total - available;
        for priority in priorities {
            if deficit <= EPSILON {
                break;
            }
            deficit = distribute_shrink(&mut resolved, priority, deficit);
        }
    }

    total = resolved.iter().map(|axis| axis.resolved).sum();
    if total > available + EPSILON {
        let mut deficit = total - available;
        for axis in &mut resolved {
            if deficit <= EPSILON {
                break;
            }
            let shrink = (axis.resolved - axis.min).max(0.0).min(deficit);
            axis.resolved -= shrink;
            deficit -= shrink;
        }
    }

    resolved
}

pub fn compute_workbench_shell_geometry(
    model: &WorkbenchViewModel,
    _chrome: &EditorChromeSnapshot,
    layout: &WorkbenchLayout,
    descriptors: &[ViewDescriptor],
    shell_size: ShellSizePx,
    metrics: &WorkbenchChromeMetrics,
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> WorkbenchShellGeometry {
    let descriptor_map: HashMap<_, _> = descriptors
        .iter()
        .map(|descriptor| (descriptor.descriptor_id.clone(), descriptor))
        .collect();
    let size = ShellSizePx::new(shell_size.width.max(1.0), shell_size.height.max(1.0));

    let left = build_tool_region_state(
        model,
        layout,
        &descriptor_map,
        ShellRegionId::Left,
        &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        transient_region_preferred,
        metrics,
    );
    let right = build_tool_region_state(
        model,
        layout,
        &descriptor_map,
        ShellRegionId::Right,
        &[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ],
        transient_region_preferred,
        metrics,
    );
    let bottom = build_tool_region_state(
        model,
        layout,
        &descriptor_map,
        ShellRegionId::Bottom,
        &[
            ActivityDrawerSlot::BottomLeft,
            ActivityDrawerSlot::BottomRight,
        ],
        transient_region_preferred,
        metrics,
    );
    let document =
        build_document_region_state(model, layout, &descriptor_map, transient_region_preferred);

    let status_bar_frame = ShellFrame::new(
        0.0,
        size.height - metrics.status_bar_height,
        size.width,
        metrics.status_bar_height,
    );
    let center_y = metrics.top_bar_height
        + metrics.separator_thickness
        + metrics.host_bar_height
        + metrics.separator_thickness;
    let fixed_vertical = metrics.top_bar_height
        + metrics.separator_thickness
        + metrics.host_bar_height
        + metrics.separator_thickness
        + metrics.separator_thickness
        + metrics.status_bar_height
        + if bottom.visible {
            metrics.separator_thickness
        } else {
            0.0
        };
    let center_and_bottom_available = (size.height - fixed_vertical).max(0.0);

    let row_height_constraint =
        aggregate_row_constraints(&[left.constraints, document.constraints, right.constraints]);
    let mut center_height = center_and_bottom_available;
    let mut bottom_height = 0.0;
    if bottom.visible {
        let band_heights = solve_axis_constraints(
            center_and_bottom_available,
            &[row_height_constraint.height, bottom.constraints.height],
        );
        center_height = band_heights[0].resolved;
        bottom_height = band_heights[1].resolved;
    }

    let visible_row_count = [left.visible, true, right.visible]
        .into_iter()
        .filter(|visible| *visible)
        .count();
    let row_separator_count = visible_row_count.saturating_sub(1) as f32;
    let available_row_width =
        (size.width - row_separator_count * metrics.separator_thickness).max(0.0);

    let mut horizontal_constraints = Vec::new();
    let mut horizontal_regions = Vec::new();
    if left.visible {
        horizontal_regions.push(ShellRegionId::Left);
        horizontal_constraints.push(left.constraints.width);
    }
    horizontal_regions.push(ShellRegionId::Document);
    horizontal_constraints.push(document.constraints.width);
    if right.visible {
        horizontal_regions.push(ShellRegionId::Right);
        horizontal_constraints.push(right.constraints.width);
    }
    let solved_widths = solve_axis_constraints(available_row_width, &horizontal_constraints);

    let center_band_frame = ShellFrame::new(0.0, center_y, size.width, center_height);
    let mut region_frames = BTreeMap::new();
    let mut x = 0.0;
    for (region, solved) in horizontal_regions.into_iter().zip(solved_widths.iter()) {
        let frame = ShellFrame::new(x, center_y, solved.resolved, center_height);
        region_frames.insert(region, frame);
        x += solved.resolved + metrics.separator_thickness;
    }

    let left_frame = region_frames
        .get(&ShellRegionId::Left)
        .copied()
        .unwrap_or_default();
    let document_frame = region_frames
        .get(&ShellRegionId::Document)
        .copied()
        .unwrap_or_default();
    let right_frame = region_frames
        .get(&ShellRegionId::Right)
        .copied()
        .unwrap_or_default();

    let bottom_y = center_y
        + center_height
        + if bottom.visible {
            metrics.separator_thickness
        } else {
            0.0
        };
    let bottom_frame = if bottom.visible {
        ShellFrame::new(0.0, bottom_y, size.width, bottom_height)
    } else {
        ShellFrame::default()
    };
    region_frames.insert(ShellRegionId::Bottom, bottom_frame);

    let viewport_toolbar_height = active_document_tab(model)
        .map(|tab| {
            matches!(
                tab.content_kind,
                ViewContentKind::Scene | ViewContentKind::Game
            )
        })
        .unwrap_or(false)
        .then_some(metrics.viewport_toolbar_height)
        .unwrap_or(0.0);
    let viewport_content_frame = ShellFrame::new(
        document_frame.x,
        document_frame.y
            + metrics.document_header_height
            + metrics.separator_thickness
            + viewport_toolbar_height,
        document_frame.width,
        (document_frame.height
            - metrics.document_header_height
            - metrics.separator_thickness
            - viewport_toolbar_height)
            .max(0.0),
    );

    let mut splitter_frames = BTreeMap::new();
    if left.expanded && left_frame.width > metrics.rail_width + EPSILON {
        splitter_frames.insert(
            ShellRegionId::Left,
            ShellFrame::new(
                left_frame.right() - metrics.splitter_hit_size / 2.0,
                center_band_frame.y,
                metrics.splitter_hit_size,
                center_band_frame.height,
            ),
        );
    }
    if right.expanded && right_frame.width > metrics.rail_width + EPSILON {
        splitter_frames.insert(
            ShellRegionId::Right,
            ShellFrame::new(
                right_frame.x - metrics.separator_thickness - metrics.splitter_hit_size / 2.0,
                center_band_frame.y,
                metrics.splitter_hit_size,
                center_band_frame.height,
            ),
        );
    }
    if bottom.expanded && bottom_frame.height > EPSILON {
        splitter_frames.insert(
            ShellRegionId::Bottom,
            ShellFrame::new(
                0.0,
                bottom_frame.y - metrics.separator_thickness - metrics.splitter_hit_size / 2.0,
                size.width,
                metrics.splitter_hit_size,
            ),
        );
    }

    let window_min_width = {
        let mut widths = Vec::new();
        if left.visible {
            widths.push(left.constraints);
        }
        widths.push(document.constraints);
        if right.visible {
            widths.push(right.constraints);
        }
        let separators = widths.len().saturating_sub(1) as f32 * metrics.separator_thickness;
        aggregate_row_constraints(&widths).width.resolved().min + separators
    };
    let window_min_height = {
        let mut min_height = metrics.top_bar_height
            + metrics.separator_thickness
            + metrics.host_bar_height
            + metrics.separator_thickness
            + metrics.status_bar_height
            + metrics.separator_thickness;
        let center_min = row_height_constraint.height.resolved().min;
        if bottom.visible {
            min_height +=
                center_min + bottom.constraints.height.resolved().min + metrics.separator_thickness;
        } else {
            min_height += center_min;
        }
        min_height
    };

    WorkbenchShellGeometry {
        window_min_width,
        window_min_height,
        center_band_frame,
        status_bar_frame,
        region_frames,
        splitter_frames,
        viewport_content_frame,
    }
}

#[derive(Clone, Copy, Debug)]
struct RegionState {
    visible: bool,
    expanded: bool,
    constraints: PaneConstraints,
}

fn build_tool_region_state(
    model: &WorkbenchViewModel,
    layout: &WorkbenchLayout,
    descriptors: &HashMap<ViewDescriptorId, &ViewDescriptor>,
    region: ShellRegionId,
    slots: &[ActivityDrawerSlot],
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
    metrics: &WorkbenchChromeMetrics,
) -> RegionState {
    let drawers_visible = model.drawer_ring.visible;
    let tab = active_tool_tab(model, slots);
    let has_tabs = drawers_visible
        && slots.iter().any(|slot| {
            model
                .tool_windows
                .get(slot)
                .is_some_and(|stack| stack.visible && !stack.tabs.is_empty())
        });
    let expanded = drawers_visible
        && slots.iter().any(|slot| {
            model.tool_windows.get(slot).is_some_and(|stack| {
                stack.visible
                    && !stack.tabs.is_empty()
                    && stack.mode != ActivityDrawerMode::Collapsed
            })
        });
    let extent = transient_region_preferred
        .and_then(|map| map.get(&region).copied())
        .unwrap_or_else(|| {
            slots
                .iter()
                .filter_map(|slot| layout.drawers.get(slot))
                .filter(|drawer| drawer.visible)
                .map(|drawer| drawer.extent)
                .fold(0.0_f32, f32::max)
        });

    if !has_tabs {
        return RegionState {
            visible: false,
            expanded: false,
            constraints: fixed_zero_constraints(),
        };
    }

    if !expanded {
        let collapsed_primary = match region {
            ShellRegionId::Bottom => metrics.panel_header_height,
            ShellRegionId::Left | ShellRegionId::Right => metrics.rail_width,
            ShellRegionId::Document => 0.0,
        };
        return RegionState {
            visible: true,
            expanded: false,
            constraints: match region {
                ShellRegionId::Bottom => PaneConstraints {
                    width: stretch_axis(0.0, 0.0, 50, 1.0),
                    height: fixed_axis(collapsed_primary),
                },
                ShellRegionId::Left | ShellRegionId::Right => PaneConstraints {
                    width: fixed_axis(collapsed_primary),
                    height: stretch_axis(0.0, 0.0, 50, 1.0),
                },
                ShellRegionId::Document => fixed_zero_constraints(),
            },
        };
    }

    let descriptor_constraints = tab
        .and_then(|tab| descriptors.get(&tab.descriptor_id).copied())
        .map(|descriptor| {
            if descriptor.default_constraints == PaneConstraints::default() {
                default_constraints_for_content(
                    tab.map(|tab| tab.content_kind)
                        .unwrap_or(ViewContentKind::Placeholder),
                )
            } else {
                descriptor.default_constraints
            }
        })
        .unwrap_or_else(|| default_region_constraints(region));
    let layout_override = layout.region_overrides.get(&region).copied();
    let view_override = tab.and_then(|tab| layout.view_overrides.get(&tab.instance_id).copied());
    let constraints = set_primary_preferred(
        region,
        merge_constraints(
            default_region_constraints(region),
            layout_override,
            descriptor_constraints,
            view_override,
        ),
        extent.max(0.0),
    );

    RegionState {
        visible: true,
        expanded: true,
        constraints,
    }
}

fn build_document_region_state(
    model: &WorkbenchViewModel,
    layout: &WorkbenchLayout,
    descriptors: &HashMap<ViewDescriptorId, &ViewDescriptor>,
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> RegionState {
    let tab = active_document_tab(model);
    let descriptor_constraints = tab
        .and_then(|tab| descriptors.get(&tab.descriptor_id).copied())
        .map(|descriptor| {
            if descriptor.default_constraints == PaneConstraints::default() {
                default_constraints_for_content(
                    tab.map(|tab| tab.content_kind)
                        .unwrap_or(ViewContentKind::Placeholder),
                )
            } else {
                descriptor.default_constraints
            }
        })
        .unwrap_or_else(|| default_region_constraints(ShellRegionId::Document));
    let layout_override = layout
        .region_overrides
        .get(&ShellRegionId::Document)
        .copied();
    let view_override = tab.and_then(|tab| layout.view_overrides.get(&tab.instance_id).copied());
    let mut constraints = merge_constraints(
        default_region_constraints(ShellRegionId::Document),
        layout_override,
        descriptor_constraints,
        view_override,
    );
    if let Some(preferred) =
        transient_region_preferred.and_then(|map| map.get(&ShellRegionId::Document).copied())
    {
        constraints.width.preferred = preferred;
    }
    RegionState {
        visible: true,
        expanded: true,
        constraints,
    }
}

fn active_tool_tab<'a>(
    model: &'a WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Option<&'a PaneTabModel> {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .find(|stack| {
            stack.visible && stack.mode != ActivityDrawerMode::Collapsed && !stack.tabs.is_empty()
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| stack.visible && !stack.tabs.is_empty())
        })
        .and_then(|stack| {
            stack
                .tabs
                .iter()
                .find(|tab| tab.active)
                .or_else(|| stack.tabs.first())
        })
}

fn active_document_tab(model: &WorkbenchViewModel) -> Option<&DocumentTabModel> {
    model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first())
}

fn merge_constraints(
    region_defaults: PaneConstraints,
    region_override: Option<PaneConstraintOverride>,
    descriptor_defaults: PaneConstraints,
    view_override: Option<PaneConstraintOverride>,
) -> PaneConstraints {
    PaneConstraints {
        width: merge_axis(
            region_defaults.width,
            region_override.map(|override_set| override_set.width),
            descriptor_defaults.width,
            view_override.map(|override_set| override_set.width),
        ),
        height: merge_axis(
            region_defaults.height,
            region_override.map(|override_set| override_set.height),
            descriptor_defaults.height,
            view_override.map(|override_set| override_set.height),
        ),
    }
}

fn merge_axis(
    region_default: AxisConstraint,
    region_override: Option<AxisConstraintOverride>,
    descriptor_default: AxisConstraint,
    view_override: Option<AxisConstraintOverride>,
) -> AxisConstraint {
    let mut axis = descriptor_default;
    if descriptor_default == AxisConstraint::default() {
        axis = region_default;
    }
    if let Some(override_axis) = region_override {
        axis = override_axis.apply_to(axis);
    }
    if let Some(override_axis) = view_override {
        axis = override_axis.apply_to(axis);
    }
    axis
}

fn set_primary_preferred(
    region: ShellRegionId,
    mut constraints: PaneConstraints,
    preferred: f32,
) -> PaneConstraints {
    match region {
        ShellRegionId::Bottom => constraints.height.preferred = preferred,
        ShellRegionId::Left | ShellRegionId::Right | ShellRegionId::Document => {
            constraints.width.preferred = preferred;
        }
    }
    constraints
}

fn aggregate_row_constraints(children: &[PaneConstraints]) -> PaneConstraints {
    if children.is_empty() {
        return fixed_zero_constraints();
    }
    PaneConstraints {
        width: AxisConstraint {
            min: children
                .iter()
                .map(|constraint| constraint.width.resolved().min)
                .sum(),
            max: sum_max(
                children
                    .iter()
                    .map(|constraint| constraint.width.resolved().max),
            ),
            preferred: children
                .iter()
                .map(|constraint| constraint.width.resolved().preferred)
                .sum(),
            priority: children
                .iter()
                .map(|constraint| constraint.width.priority)
                .max()
                .unwrap_or_default(),
            weight: children
                .iter()
                .map(|constraint| constraint.width.weight)
                .sum(),
            stretch_mode: StretchMode::Stretch,
        },
        height: AxisConstraint {
            min: children
                .iter()
                .map(|constraint| constraint.height.resolved().min)
                .fold(0.0_f32, f32::max),
            max: max_max(
                children
                    .iter()
                    .map(|constraint| constraint.height.resolved().max),
            ),
            preferred: children
                .iter()
                .map(|constraint| constraint.height.resolved().preferred)
                .fold(0.0_f32, f32::max),
            priority: children
                .iter()
                .map(|constraint| constraint.height.priority)
                .max()
                .unwrap_or_default(),
            weight: children
                .iter()
                .map(|constraint| constraint.height.weight)
                .sum(),
            stretch_mode: StretchMode::Stretch,
        },
    }
}

fn priorities_descending(
    resolved: &[ResolvedAxisConstraint],
    filter: impl Fn(&ResolvedAxisConstraint) -> bool,
) -> Vec<i32> {
    let mut priorities: Vec<_> = resolved
        .iter()
        .filter(|axis| filter(axis))
        .map(|axis| axis.priority)
        .collect();
    priorities.sort_unstable();
    priorities.dedup();
    priorities.reverse();
    priorities
}

fn priorities_ascending(
    resolved: &[ResolvedAxisConstraint],
    filter: impl Fn(&ResolvedAxisConstraint) -> bool,
) -> Vec<i32> {
    let mut priorities: Vec<_> = resolved
        .iter()
        .filter(|axis| filter(axis))
        .map(|axis| axis.priority)
        .collect();
    priorities.sort_unstable();
    priorities.dedup();
    priorities
}

fn distribute_growth(
    resolved: &mut [ResolvedAxisConstraint],
    priority: i32,
    remaining: f32,
) -> f32 {
    let mut remaining = remaining;
    loop {
        let indices: Vec<_> = resolved
            .iter()
            .enumerate()
            .filter(|(_, axis)| {
                axis.priority == priority
                    && axis.stretch_mode == StretchMode::Stretch
                    && axis.max.is_none_or(|max| axis.resolved + EPSILON < max)
            })
            .map(|(index, _)| index)
            .collect();
        if indices.is_empty() || remaining <= EPSILON {
            return remaining;
        }
        let weight_sum: f32 = indices.iter().map(|index| resolved[*index].weight).sum();
        let active_count = indices.len() as f32;
        let mut consumed = 0.0;
        for index in indices {
            let share = if weight_sum <= EPSILON {
                remaining / active_count
            } else {
                remaining * (resolved[index].weight / weight_sum)
            };
            let capacity = resolved[index]
                .max
                .map(|max| (max - resolved[index].resolved).max(0.0))
                .unwrap_or(share);
            let delta = share.min(capacity);
            resolved[index].resolved += delta;
            consumed += delta;
        }
        if consumed <= EPSILON {
            return remaining;
        }
        remaining -= consumed;
    }
}

fn distribute_shrink(resolved: &mut [ResolvedAxisConstraint], priority: i32, deficit: f32) -> f32 {
    let mut deficit = deficit;
    loop {
        let indices: Vec<_> = resolved
            .iter()
            .enumerate()
            .filter(|(_, axis)| axis.priority == priority && axis.resolved > axis.min + EPSILON)
            .map(|(index, _)| index)
            .collect();
        if indices.is_empty() || deficit <= EPSILON {
            return deficit;
        }
        let weight_sum: f32 = indices.iter().map(|index| resolved[*index].weight).sum();
        let active_count = indices.len() as f32;
        let mut consumed = 0.0;
        for index in indices {
            let share = if weight_sum <= EPSILON {
                deficit / active_count
            } else {
                deficit * (resolved[index].weight / weight_sum)
            };
            let capacity = (resolved[index].resolved - resolved[index].min).max(0.0);
            let delta = share.min(capacity);
            resolved[index].resolved -= delta;
            consumed += delta;
        }
        if consumed <= EPSILON {
            return deficit;
        }
        deficit -= consumed;
    }
}

fn clamp_axis_value(value: f32, min: f32, max: Option<f32>) -> f32 {
    max.map(|max| value.clamp(min, max))
        .unwrap_or_else(|| value.max(min))
}

fn sum_max(values: impl Iterator<Item = Option<f32>>) -> f32 {
    let mut total = 0.0;
    for value in values {
        let Some(value) = value else {
            return -1.0;
        };
        total += value;
    }
    total
}

fn max_max(values: impl Iterator<Item = Option<f32>>) -> f32 {
    let mut max_value: f32 = 0.0;
    for value in values {
        let Some(value) = value else {
            return -1.0;
        };
        max_value = max_value.max(value);
    }
    max_value
}

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn stretch_axis(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

fn fixed_zero_constraints() -> PaneConstraints {
    PaneConstraints {
        width: fixed_axis(0.0),
        height: fixed_axis(0.0),
    }
}

pub fn default_region_constraints(region: ShellRegionId) -> PaneConstraints {
    match region {
        ShellRegionId::Left | ShellRegionId::Right => PaneConstraints {
            width: stretch_axis(220.0, 288.0, 50, 1.0),
            height: stretch_axis(220.0, 480.0, 50, 1.0),
        },
        ShellRegionId::Bottom => PaneConstraints {
            width: stretch_axis(0.0, 0.0, 50, 1.0),
            height: stretch_axis(120.0, 164.0, 50, 1.0),
        },
        ShellRegionId::Document => PaneConstraints {
            width: stretch_axis(480.0, 960.0, 100, 3.0),
            height: stretch_axis(272.0, 640.0, 100, 3.0),
        },
    }
}

pub fn default_constraints_for_content(kind: ViewContentKind) -> PaneConstraints {
    match kind {
        ViewContentKind::Welcome => default_region_constraints(ShellRegionId::Document),
        ViewContentKind::Project | ViewContentKind::Assets | ViewContentKind::Hierarchy => {
            PaneConstraints {
                width: stretch_axis(240.0, 312.0, 50, 1.0),
                height: stretch_axis(220.0, 520.0, 50, 1.0),
            }
        }
        ViewContentKind::Inspector => PaneConstraints {
            width: stretch_axis(240.0, 308.0, 50, 1.0),
            height: stretch_axis(220.0, 520.0, 50, 1.0),
        },
        ViewContentKind::Console => PaneConstraints {
            width: stretch_axis(0.0, 0.0, 50, 1.0),
            height: stretch_axis(120.0, 164.0, 50, 1.0),
        },
        ViewContentKind::Scene | ViewContentKind::Game | ViewContentKind::PrefabEditor => {
            PaneConstraints {
                width: stretch_axis(480.0, 960.0, 100, 3.0),
                height: stretch_axis(272.0, 640.0, 100, 3.0),
            }
        }
        ViewContentKind::AssetBrowser | ViewContentKind::Placeholder => {
            default_region_constraints(ShellRegionId::Document)
        }
    }
}
