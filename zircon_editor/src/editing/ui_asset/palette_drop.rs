use std::collections::BTreeMap;

use toml::Value;
use zircon_ui::{UiAssetDocument, UiComponentDefinition, UiNodeDefinition};

use super::{
    preview_projection::UiAssetPreviewProjection,
    tree_editing::{
        insert_palette_item_with_placement, PaletteInsertMode, UiAssetPaletteEntry,
        UiAssetPaletteInsertionPlacement,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UiAssetPaletteInsertPlan {
    pub node_id: String,
    pub mode: PaletteInsertMode,
    pub label: String,
    pub placement: UiAssetPaletteInsertionPlacement,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UiAssetPaletteDragTarget {
    pub preview_index: Option<usize>,
    pub plan: UiAssetPaletteInsertPlan,
    pub key: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UiAssetPaletteDragResolution {
    pub candidates: Vec<UiAssetPaletteDragTarget>,
    pub selected_index: usize,
}

impl UiAssetPaletteDragResolution {
    pub fn selected_target(&self) -> Option<&UiAssetPaletteDragTarget> {
        self.candidates.get(self.selected_index)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiAssetPaletteSlotTargetOverlay {
    pub label: String,
    pub detail: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub selected: bool,
}

pub(super) fn can_insert_palette_item_for_node(
    document: &UiAssetDocument,
    palette_entry: &UiAssetPaletteEntry,
    node_id: &str,
    mode: PaletteInsertMode,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> bool {
    build_palette_insert_plan(document, palette_entry, node_id, mode, widget_imports, None)
        .is_some()
}

pub(super) fn build_palette_insert_plan(
    document: &UiAssetDocument,
    palette_entry: &UiAssetPaletteEntry,
    node_id: &str,
    mode: PaletteInsertMode,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    hover: Option<UiAssetPaletteHoverContext>,
) -> Option<UiAssetPaletteInsertPlan> {
    if !document.nodes.contains_key(node_id) {
        return None;
    }

    let plan = match mode {
        PaletteInsertMode::After => UiAssetPaletteInsertPlan {
            node_id: node_id.to_string(),
            mode,
            label: "Insert After".to_string(),
            placement: UiAssetPaletteInsertionPlacement::default(),
        },
        PaletteInsertMode::Child => {
            let node = document.nodes.get(node_id)?;
            if let Some(mount) = component_mount_for_node(document, node, widget_imports, hover) {
                UiAssetPaletteInsertPlan {
                    node_id: node_id.to_string(),
                    mode,
                    label: format!("Insert {} Slot", title_case_identifier(&mount)),
                    placement: UiAssetPaletteInsertionPlacement {
                        mount: Some(mount),
                        slot: BTreeMap::new(),
                    },
                }
            } else {
                UiAssetPaletteInsertPlan {
                    node_id: node_id.to_string(),
                    mode,
                    label: native_child_insert_label(node),
                    placement: native_child_placement(node, hover),
                }
            }
        }
    };

    finalize_palette_insert_plan(document, palette_entry, plan)
}

fn finalize_palette_insert_plan(
    document: &UiAssetDocument,
    palette_entry: &UiAssetPaletteEntry,
    plan: UiAssetPaletteInsertPlan,
) -> Option<UiAssetPaletteInsertPlan> {
    let mut candidate = document.clone();
    insert_palette_item_with_placement(
        &mut candidate,
        &plan.node_id,
        palette_entry,
        plan.mode,
        &plan.placement,
    )
    .map(|_| plan)
}

pub(super) fn resolve_palette_drag_target(
    document: &UiAssetDocument,
    palette_entry: &UiAssetPaletteEntry,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    projection: &UiAssetPreviewProjection,
    surface_x: f32,
    surface_y: f32,
) -> Option<UiAssetPaletteDragResolution> {
    for (preview_index, item) in projection.canvas_nodes.iter().enumerate().rev() {
        let item_width = item.width.max(2.0);
        let item_height = item.height.max(2.0);
        let within_x = surface_x >= item.x && surface_x <= item.x + item_width;
        let within_y = surface_y >= item.y && surface_y <= item.y + item_height;
        let hover = UiAssetPaletteHoverContext::new(
            item.x,
            item.y,
            item_width,
            item_height,
            surface_x,
            surface_y,
        );

        if within_x && within_y {
            if let Some(resolution) = build_palette_drag_resolution(
                document,
                palette_entry,
                &item.node_id,
                PaletteInsertMode::Child,
                widget_imports,
                Some(hover),
                Some(preview_index),
            ) {
                return Some(resolution);
            }
        }

        let after_top = item.y + item_height + 4.0;
        let after_bottom = after_top + 14.0;
        if within_x && surface_y >= after_top && surface_y <= after_bottom {
            if let Some(resolution) = build_palette_drag_resolution(
                document,
                palette_entry,
                &item.node_id,
                PaletteInsertMode::After,
                widget_imports,
                None,
                Some(preview_index),
            ) {
                return Some(resolution);
            }
        }
    }

    let root_id = document.root.as_ref()?.node.as_str();
    let surface_hover = UiAssetPaletteHoverContext::new(
        0.0,
        0.0,
        projection.surface_width.max(0.0),
        projection.surface_height.max(0.0),
        surface_x,
        surface_y,
    );
    if surface_hover.contains_point() {
        build_palette_drag_resolution(
            document,
            palette_entry,
            root_id,
            PaletteInsertMode::Child,
            widget_imports,
            Some(surface_hover),
            None,
        )
    } else {
        None
    }
}

fn build_palette_drag_resolution(
    document: &UiAssetDocument,
    palette_entry: &UiAssetPaletteEntry,
    node_id: &str,
    mode: PaletteInsertMode,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    hover: Option<UiAssetPaletteHoverContext>,
    preview_index: Option<usize>,
) -> Option<UiAssetPaletteDragResolution> {
    if !document.nodes.contains_key(node_id) {
        return None;
    }

    let Some(node) = document.nodes.get(node_id) else {
        return None;
    };
    if mode == PaletteInsertMode::Child {
        if let Some(hover) = hover {
            if let Some(resolution) = build_component_palette_drag_resolution(
                document,
                palette_entry,
                node_id,
                node,
                widget_imports,
                hover,
                preview_index,
            ) {
                return Some(resolution);
            }
            if let Some(resolution) = build_native_palette_drag_resolution(
                document,
                palette_entry,
                node_id,
                node,
                hover,
                preview_index,
            ) {
                return Some(resolution);
            }
        }
    }

    let plan = build_palette_insert_plan(
        document,
        palette_entry,
        node_id,
        mode,
        widget_imports,
        hover,
    )?;
    Some(UiAssetPaletteDragResolution {
        candidates: vec![UiAssetPaletteDragTarget {
            preview_index,
            key: plan.label.clone(),
            detail: String::new(),
            plan,
        }],
        selected_index: 0,
    })
}

fn build_component_palette_drag_resolution(
    document: &UiAssetDocument,
    palette_entry: &UiAssetPaletteEntry,
    node_id: &str,
    node: &UiNodeDefinition,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    hover: UiAssetPaletteHoverContext,
    preview_index: Option<usize>,
) -> Option<UiAssetPaletteDragResolution> {
    let targets = component_slot_targets(
        document,
        node,
        widget_imports,
        UiAssetPaletteTargetFrame {
            x: hover.x,
            y: hover.y,
            width: hover.width,
            height: hover.height,
        },
        None,
    )?;
    if targets.is_empty() {
        return None;
    }

    let mut candidates = Vec::new();
    for target in targets {
        let plan = finalize_palette_insert_plan(
            document,
            palette_entry,
            UiAssetPaletteInsertPlan {
                node_id: node_id.to_string(),
                mode: PaletteInsertMode::Child,
                label: format!("Insert {} Slot", title_case_identifier(&target.mount)),
                placement: UiAssetPaletteInsertionPlacement {
                    mount: Some(target.mount.clone()),
                    slot: BTreeMap::new(),
                },
            },
        )?;
        candidates.push(UiAssetPaletteDragTarget {
            preview_index,
            key: target.overlay.label,
            detail: target.overlay.detail,
            plan,
        });
    }

    let selected_mount = component_mount_for_node(document, node, widget_imports, Some(hover));
    let selected_index = selected_mount
        .as_deref()
        .and_then(|mount| {
            candidates.iter().position(|candidate| {
                candidate.plan.placement.mount.as_deref() == Some(mount)
            })
        })
        .unwrap_or(0);

    Some(UiAssetPaletteDragResolution {
        candidates,
        selected_index,
    })
}

fn build_native_palette_drag_resolution(
    document: &UiAssetDocument,
    palette_entry: &UiAssetPaletteEntry,
    node_id: &str,
    node: &UiNodeDefinition,
    hover: UiAssetPaletteHoverContext,
    preview_index: Option<usize>,
) -> Option<UiAssetPaletteDragResolution> {
    let label = native_child_insert_label(node);
    let targets = match native_container_kind(node) {
        Some("Overlay") => overlay_slot_targets(hover),
        Some("GridBox") => grid_slot_targets(node, hover),
        Some("FlowBox") => flow_slot_targets(hover),
        _ => return None,
    };
    if targets.is_empty() {
        return None;
    }

    let default_slot = native_child_placement(node, Some(hover)).slot;
    let mut candidates = Vec::new();
    for target in targets {
        let plan = finalize_palette_insert_plan(
            document,
            palette_entry,
            UiAssetPaletteInsertPlan {
                node_id: node_id.to_string(),
                mode: PaletteInsertMode::Child,
                label: label.clone(),
                placement: UiAssetPaletteInsertionPlacement {
                    mount: None,
                    slot: target.slot.clone(),
                },
            },
        )?;
        candidates.push(UiAssetPaletteDragTarget {
            preview_index,
            key: target.label,
            detail: target.detail,
            plan,
        });
    }

    let selected_index = candidates
        .iter()
        .position(|candidate| candidate.plan.placement.slot == default_slot)
        .unwrap_or(0);
    Some(UiAssetPaletteDragResolution {
        candidates,
        selected_index,
    })
}

pub(super) fn build_palette_drag_slot_target_overlays(
    document: &UiAssetDocument,
    drag_target: &UiAssetPaletteDragTarget,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    projection: &UiAssetPreviewProjection,
) -> Vec<UiAssetPaletteSlotTargetOverlay> {
    if drag_target.plan.mode != PaletteInsertMode::Child {
        return Vec::new();
    }

    let Some(node) = document.nodes.get(&drag_target.plan.node_id) else {
        return Vec::new();
    };
    let Some(frame) = target_frame_for_drag_target(drag_target, projection) else {
        return Vec::new();
    };

    if let Some(overlays) = component_slot_target_overlays(
        document,
        node,
        widget_imports,
        frame,
        drag_target.plan.placement.mount.as_deref(),
    ) {
        return overlays;
    }

    match native_container_kind(node) {
        Some("Overlay") => overlay_slot_target_overlays(frame, &drag_target.plan.placement.slot),
        Some("GridBox") => grid_slot_target_overlays(node, frame, &drag_target.plan.placement.slot),
        Some("FlowBox") => flow_slot_target_overlays(frame, &drag_target.plan.placement.slot),
        _ => Vec::new(),
    }
}

#[derive(Clone, Copy, Debug)]
struct UiAssetPaletteTargetFrame {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct UiAssetPaletteHoverContext {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    surface_x: f32,
    surface_y: f32,
}

impl UiAssetPaletteHoverContext {
    fn new(x: f32, y: f32, width: f32, height: f32, surface_x: f32, surface_y: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            surface_x,
            surface_y,
        }
    }

    fn contains_point(&self) -> bool {
        self.surface_x >= self.x
            && self.surface_x <= self.x + self.width.max(0.0)
            && self.surface_y >= self.y
            && self.surface_y <= self.y + self.height.max(0.0)
    }

    fn normalized_x(&self) -> f32 {
        normalized_axis(self.surface_x, self.x, self.width)
    }

    fn normalized_y(&self) -> f32 {
        normalized_axis(self.surface_y, self.y, self.height)
    }
}

fn target_frame_for_drag_target(
    drag_target: &UiAssetPaletteDragTarget,
    projection: &UiAssetPreviewProjection,
) -> Option<UiAssetPaletteTargetFrame> {
    if let Some(preview_index) = drag_target.preview_index {
        let item = projection.canvas_nodes.get(preview_index)?;
        return Some(UiAssetPaletteTargetFrame {
            x: item.x,
            y: item.y,
            width: item.width.max(1.0),
            height: item.height.max(1.0),
        });
    }

    Some(UiAssetPaletteTargetFrame {
        x: 0.0,
        y: 0.0,
        width: projection.surface_width.max(1.0),
        height: projection.surface_height.max(1.0),
    })
}

fn component_slot_target_overlays(
    document: &UiAssetDocument,
    node: &UiNodeDefinition,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    frame: UiAssetPaletteTargetFrame,
    selected_mount: Option<&str>,
) -> Option<Vec<UiAssetPaletteSlotTargetOverlay>> {
    Some(
        component_slot_targets(document, node, widget_imports, frame, selected_mount)?
            .into_iter()
            .map(|target| target.overlay)
            .collect(),
    )
}

fn ordered_component_slot_names(available: &[String], groups: &[&[&str]]) -> Vec<String> {
    let mut ordered = Vec::new();
    for semantics in groups {
        for slot_name in available {
            if ordered.iter().any(|existing| existing == slot_name) {
                continue;
            }
            if semantics
                .iter()
                .any(|semantic| normalized_slot_name(slot_name).contains(semantic))
            {
                ordered.push(slot_name.clone());
            }
        }
    }
    for slot_name in available {
        if !ordered.iter().any(|existing| existing == slot_name) {
            ordered.push(slot_name.clone());
        }
    }
    ordered
}

fn component_mount_for_node(
    document: &UiAssetDocument,
    node: &UiNodeDefinition,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    hover: Option<UiAssetPaletteHoverContext>,
) -> Option<String> {
    let component = component_definition_for_node(document, node, widget_imports)?;
    let available = available_component_slots(component, &node.children);
    if available.is_empty() {
        return None;
    }
    if available.len() == 1 {
        return available.into_iter().next();
    }

    let hover = hover?;
    if let Some(targets) = component_slot_targets(
        document,
        node,
        widget_imports,
        UiAssetPaletteTargetFrame {
            x: hover.x,
            y: hover.y,
            width: hover.width,
            height: hover.height,
        },
        None,
    ) {
        if let Some(target) = targets
            .into_iter()
            .find(|target| point_within_overlay(hover.surface_x, hover.surface_y, &target.overlay))
        {
            return Some(target.mount);
        }
    }

    let horizontal = contains_slot_semantics(
        &available,
        &["leading", "left", "start", "trailing", "right", "end"],
    );
    let vertical = contains_slot_semantics(
        &available,
        &[
            "header", "top", "body", "content", "center", "main", "footer", "bottom",
        ],
    );

    if horizontal {
        if hover.normalized_x() >= 0.66 {
            if let Some(slot) = find_slot_by_semantics(&available, &["trailing", "right", "end"]) {
                return Some(slot);
            }
        }
        if hover.normalized_x() <= 0.33 {
            if let Some(slot) = find_slot_by_semantics(&available, &["leading", "left", "start"]) {
                return Some(slot);
            }
        }
    }

    if vertical {
        if hover.normalized_y() <= 0.33 {
            if let Some(slot) = find_slot_by_semantics(&available, &["header", "top"]) {
                return Some(slot);
            }
        }
        if hover.normalized_y() >= 0.66 {
            if let Some(slot) =
                find_slot_by_semantics(&available, &["footer", "bottom", "body", "content", "main"])
            {
                return Some(slot);
            }
        }
        if let Some(slot) =
            find_slot_by_semantics(&available, &["content", "body", "center", "main"])
        {
            return Some(slot);
        }
    }

    find_slot_by_semantics(
        &available,
        &[
            "content", "body", "center", "main", "default", "leading", "left", "start", "trailing",
            "right", "end", "header", "top", "footer", "bottom",
        ],
    )
    .or_else(|| available.into_iter().next())
}

#[derive(Clone, Debug)]
struct UiAssetComponentSlotTarget {
    mount: String,
    overlay: UiAssetPaletteSlotTargetOverlay,
}

fn component_slot_targets(
    document: &UiAssetDocument,
    node: &UiNodeDefinition,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    frame: UiAssetPaletteTargetFrame,
    selected_mount: Option<&str>,
) -> Option<Vec<UiAssetComponentSlotTarget>> {
    let component = component_definition_for_node(document, node, widget_imports)?;
    let available = available_component_slots(component, &node.children);
    if available.is_empty() {
        return None;
    }

    let ordered = if contains_slot_semantics(
        &available,
        &[
            "header", "top", "body", "content", "center", "main", "footer", "bottom",
        ],
    ) && !contains_slot_semantics(
        &available,
        &["leading", "left", "start", "trailing", "right", "end"],
    ) {
        ordered_component_slot_names(
            &available,
            &[
                &["header", "top"],
                &["content", "body", "center", "main", "default"],
                &["footer", "bottom"],
            ],
        )
        .into_iter()
        .enumerate()
        .map(|(index, slot_name)| UiAssetComponentSlotTarget {
            mount: slot_name.clone(),
            overlay: UiAssetPaletteSlotTargetOverlay {
                label: format!("{} Slot", title_case_identifier(&slot_name)),
                detail: slot_name.clone(),
                x: frame.x,
                y: frame.y + frame.height * index as f32 / available.len() as f32,
                width: frame.width,
                height: frame.height / available.len() as f32,
                selected: selected_mount == Some(slot_name.as_str()),
            },
        })
        .collect()
    } else {
        ordered_component_slot_names(
            &available,
            &[
                &["leading", "left", "start"],
                &["content", "body", "center", "main", "default"],
                &["trailing", "right", "end"],
            ],
        )
        .into_iter()
        .enumerate()
        .map(|(index, slot_name)| UiAssetComponentSlotTarget {
            mount: slot_name.clone(),
            overlay: UiAssetPaletteSlotTargetOverlay {
                label: format!("{} Slot", title_case_identifier(&slot_name)),
                detail: slot_name.clone(),
                x: frame.x + frame.width * index as f32 / available.len() as f32,
                y: frame.y,
                width: frame.width / available.len() as f32,
                height: frame.height,
                selected: selected_mount == Some(slot_name.as_str()),
            },
        })
        .collect()
    };

    Some(ordered)
}

fn point_within_overlay(
    surface_x: f32,
    surface_y: f32,
    overlay: &UiAssetPaletteSlotTargetOverlay,
) -> bool {
    surface_x >= overlay.x
        && surface_x <= overlay.x + overlay.width.max(0.0)
        && surface_y >= overlay.y
        && surface_y <= overlay.y + overlay.height.max(0.0)
}

fn component_definition_for_node<'a>(
    document: &'a UiAssetDocument,
    node: &UiNodeDefinition,
    widget_imports: &'a BTreeMap<String, UiAssetDocument>,
) -> Option<&'a UiComponentDefinition> {
    if let Some(component_name) = node.component.as_deref() {
        return document.components.get(component_name);
    }
    let reference = node.component_ref.as_deref()?;
    let (asset_id, component_name) = reference.split_once('#')?;
    widget_imports
        .get(reference)
        .or_else(|| widget_imports.get(asset_id))?;
    widget_imports
        .get(reference)
        .or_else(|| widget_imports.get(asset_id))
        .and_then(|document| document.components.get(component_name))
}

fn available_component_slots(
    component: &UiComponentDefinition,
    children: &[zircon_ui::UiChildMount],
) -> Vec<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for child in children {
        let slot_name = child.mount.clone().unwrap_or_default();
        let entry = counts.entry(slot_name).or_insert(0);
        *entry += 1;
    }

    component
        .slots
        .iter()
        .filter_map(|(slot_name, slot)| {
            let occupied = counts.get(slot_name).copied().unwrap_or_default();
            (slot.multiple || occupied == 0).then(|| slot_name.clone())
        })
        .collect()
}

fn contains_slot_semantics(slots: &[String], semantics: &[&str]) -> bool {
    slots.iter().any(|slot| {
        semantics
            .iter()
            .any(|semantic| normalized_slot_name(slot).contains(semantic))
    })
}

fn find_slot_by_semantics(slots: &[String], semantics: &[&str]) -> Option<String> {
    slots
        .iter()
        .find(|slot| {
            semantics
                .iter()
                .any(|semantic| normalized_slot_name(slot).contains(semantic))
        })
        .cloned()
}

fn native_child_insert_label(node: &UiNodeDefinition) -> String {
    match native_container_kind(node) {
        Some("Overlay") => "Insert Overlay Child".to_string(),
        Some("GridBox") => "Insert Grid Child".to_string(),
        Some("FlowBox") => "Insert Flow Child".to_string(),
        Some("ScrollableBox") => "Insert Scroll Child".to_string(),
        Some("HorizontalBox") => "Insert Row Child".to_string(),
        Some("VerticalBox") => "Insert Column Child".to_string(),
        _ => "Insert In".to_string(),
    }
}

fn native_child_placement(
    node: &UiNodeDefinition,
    hover: Option<UiAssetPaletteHoverContext>,
) -> UiAssetPaletteInsertionPlacement {
    let mut placement = UiAssetPaletteInsertionPlacement::default();
    let Some(hover) = hover else {
        return placement;
    };

    match native_container_kind(node) {
        Some("Overlay") => {
            placement.slot = overlay_slot_for_hover(hover);
        }
        Some("GridBox") => {
            placement.slot = grid_slot_for_hover(node, hover);
        }
        Some("FlowBox") => {
            placement.slot = flow_slot_for_hover(hover);
        }
        _ => {}
    }

    placement
}

fn native_container_kind<'a>(node: &'a UiNodeDefinition) -> Option<&'a str> {
    node.layout
        .as_ref()
        .and_then(|layout| layout.get("container"))
        .and_then(Value::as_table)
        .and_then(|container| container.get("kind"))
        .and_then(Value::as_str)
        .or_else(|| node.widget_type.as_deref())
}

#[derive(Clone, Debug)]
struct UiAssetPaletteNativeSlotTarget {
    label: String,
    detail: String,
    slot: BTreeMap<String, Value>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn overlay_slot_target_overlays(
    frame: UiAssetPaletteTargetFrame,
    selected_slot: &BTreeMap<String, Value>,
) -> Vec<UiAssetPaletteSlotTargetOverlay> {
    let selected_anchor = overlay_slot_anchor(selected_slot);
    overlay_slot_targets(UiAssetPaletteHoverContext {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
        surface_x: frame.x + frame.width * 0.5,
        surface_y: frame.y + frame.height * 0.5,
    })
    .into_iter()
    .map(|target| UiAssetPaletteSlotTargetOverlay {
        label: target.label,
        detail: target.detail,
        x: target.x,
        y: target.y,
        width: target.width,
        height: target.height,
        selected: overlay_slot_anchor(&target.slot) == selected_anchor,
    })
    .collect()
}

fn overlay_slot_anchor(slot: &BTreeMap<String, Value>) -> Option<(f32, f32)> {
    Some((
        slot_table_numeric_value(slot.get("layout")?, &["anchor", "x"])?,
        slot_table_numeric_value(slot.get("layout")?, &["anchor", "y"])?,
    ))
}

fn overlay_slot_for_hover(hover: UiAssetPaletteHoverContext) -> BTreeMap<String, Value> {
    let anchor_x = quantized_anchor(hover.normalized_x());
    let anchor_y = quantized_anchor(hover.normalized_y());
    overlay_slot_for_anchor(
        hover.x,
        hover.y,
        hover.width,
        hover.height,
        hover.surface_x,
        hover.surface_y,
        anchor_x,
        anchor_y,
    )
}

fn overlay_slot_for_anchor(
    frame_x: f32,
    frame_y: f32,
    frame_width: f32,
    frame_height: f32,
    surface_x: f32,
    surface_y: f32,
    anchor_x: f32,
    anchor_y: f32,
) -> BTreeMap<String, Value> {
    let position_x = round_position(surface_x - (frame_x + frame_width * anchor_x));
    let position_y = round_position(surface_y - (frame_y + frame_height * anchor_y));
    let mut slot = BTreeMap::new();
    let _ = slot.insert(
        "layout".to_string(),
        table_value(&[
            (
                "anchor",
                table_value(&[
                    ("x", Value::Float(anchor_x as f64)),
                    ("y", Value::Float(anchor_y as f64)),
                ]),
            ),
            (
                "pivot",
                table_value(&[
                    ("x", Value::Float(anchor_x as f64)),
                    ("y", Value::Float(anchor_y as f64)),
                ]),
            ),
            (
                "position",
                table_value(&[
                    ("x", numeric_value(position_x)),
                    ("y", numeric_value(position_y)),
                ]),
            ),
        ]),
    );
    slot
}

fn overlay_slot_targets(hover: UiAssetPaletteHoverContext) -> Vec<UiAssetPaletteNativeSlotTarget> {
    let names = [
        ["Top Left", "Top", "Top Right"],
        ["Left", "Center", "Right"],
        ["Bottom Left", "Bottom", "Bottom Right"],
    ];
    let mut targets = Vec::new();
    for row in 0..3 {
        for column in 0..3 {
            let anchor_x = [0.0, 0.5, 1.0][column];
            let anchor_y = [0.0, 0.5, 1.0][row];
            targets.push(UiAssetPaletteNativeSlotTarget {
                label: names[row][column].to_string(),
                detail: format!("anchor {}, {}", anchor_x, anchor_y),
                slot: overlay_slot_for_anchor(
                    hover.x,
                    hover.y,
                    hover.width,
                    hover.height,
                    hover.surface_x,
                    hover.surface_y,
                    anchor_x,
                    anchor_y,
                ),
                x: hover.x + hover.width * column as f32 / 3.0,
                y: hover.y + hover.height * row as f32 / 3.0,
                width: hover.width / 3.0,
                height: hover.height / 3.0,
            });
        }
    }
    targets
}

fn grid_slot_target_overlays(
    node: &UiNodeDefinition,
    frame: UiAssetPaletteTargetFrame,
    selected_slot: &BTreeMap<String, Value>,
) -> Vec<UiAssetPaletteSlotTargetOverlay> {
    let selected_row = selected_slot.get("row").and_then(Value::as_integer);
    let selected_column = selected_slot.get("column").and_then(Value::as_integer);
    grid_slot_targets(
        node,
        UiAssetPaletteHoverContext {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
            surface_x: frame.x + frame.width * 0.5,
            surface_y: frame.y + frame.height * 0.5,
        },
    )
    .into_iter()
    .map(|target| UiAssetPaletteSlotTargetOverlay {
        label: target.label,
        detail: target.detail,
        x: target.x,
        y: target.y,
        width: target.width,
        height: target.height,
        selected: target.slot.get("row").and_then(Value::as_integer) == selected_row
            && target.slot.get("column").and_then(Value::as_integer) == selected_column,
    })
    .collect()
}

fn grid_slot_for_hover(
    node: &UiNodeDefinition,
    hover: UiAssetPaletteHoverContext,
) -> BTreeMap<String, Value> {
    let columns = estimated_grid_axis(node, "column", "column_span").max(2);
    let rows = estimated_grid_axis(node, "row", "row_span").max(2);
    let column = ((hover.normalized_x() * columns as f32).floor() as i64).min(columns - 1);
    let row = ((hover.normalized_y() * rows as f32).floor() as i64).min(rows - 1);
    BTreeMap::from([
        ("row".to_string(), Value::Integer(row)),
        ("column".to_string(), Value::Integer(column)),
    ])
}

fn grid_slot_targets(
    node: &UiNodeDefinition,
    hover: UiAssetPaletteHoverContext,
) -> Vec<UiAssetPaletteNativeSlotTarget> {
    let columns = estimated_grid_axis(node, "column", "column_span").max(2) as usize;
    let rows = estimated_grid_axis(node, "row", "row_span").max(2) as usize;
    let mut targets = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            targets.push(UiAssetPaletteNativeSlotTarget {
                label: format!("R{row} C{column}"),
                detail: format!("row {row}, column {column}"),
                slot: BTreeMap::from([
                    ("row".to_string(), Value::Integer(row as i64)),
                    ("column".to_string(), Value::Integer(column as i64)),
                ]),
                x: hover.x + hover.width * column as f32 / columns as f32,
                y: hover.y + hover.height * row as f32 / rows as f32,
                width: hover.width / columns as f32,
                height: hover.height / rows as f32,
            });
        }
    }
    targets
}

fn estimated_grid_axis(node: &UiNodeDefinition, key: &str, span_key: &str) -> i64 {
    node.children
        .iter()
        .filter_map(|child| {
            let value = child.slot.get(key)?.as_integer()?;
            let span = child
                .slot
                .get(span_key)
                .and_then(Value::as_integer)
                .unwrap_or(1);
            Some(value + span)
        })
        .max()
        .unwrap_or(1)
}

fn flow_slot_target_overlays(
    frame: UiAssetPaletteTargetFrame,
    selected_slot: &BTreeMap<String, Value>,
) -> Vec<UiAssetPaletteSlotTargetOverlay> {
    let selected_break_before = selected_slot
        .get("break_before")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let selected_alignment = selected_slot
        .get("alignment")
        .and_then(Value::as_str)
        .unwrap_or("Center");
    flow_slot_targets(UiAssetPaletteHoverContext {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
        surface_x: frame.x + frame.width * 0.5,
        surface_y: frame.y + frame.height * 0.5,
    })
    .into_iter()
    .map(|target| UiAssetPaletteSlotTargetOverlay {
        label: target.label,
        detail: target.detail,
        x: target.x,
        y: target.y,
        width: target.width,
        height: target.height,
        selected: target
            .slot
            .get("break_before")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            == selected_break_before
            && target
                .slot
                .get("alignment")
                .and_then(Value::as_str)
                .unwrap_or("Center")
                == selected_alignment,
    })
    .collect()
}

fn flow_slot_for_hover(hover: UiAssetPaletteHoverContext) -> BTreeMap<String, Value> {
    let alignment = if hover.normalized_x() <= 0.33 {
        "Start"
    } else if hover.normalized_x() >= 0.66 {
        "End"
    } else {
        "Center"
    };
    BTreeMap::from([
        (
            "break_before".to_string(),
            Value::Boolean(hover.normalized_y() >= 0.5),
        ),
        (
            "alignment".to_string(),
            Value::String(alignment.to_string()),
        ),
    ])
}

fn flow_slot_targets(hover: UiAssetPaletteHoverContext) -> Vec<UiAssetPaletteNativeSlotTarget> {
    let alignments = ["Start", "Center", "End"];
    let mut targets = Vec::new();
    for row in 0..2 {
        for (column, alignment) in alignments.iter().enumerate() {
            let break_before = row == 1;
            targets.push(UiAssetPaletteNativeSlotTarget {
                label: if break_before {
                    format!("Break {alignment}")
                } else {
                    (*alignment).to_string()
                },
                detail: format!("break_before={break_before}, alignment={alignment}"),
                slot: BTreeMap::from([
                    ("break_before".to_string(), Value::Boolean(break_before)),
                    (
                        "alignment".to_string(),
                        Value::String((*alignment).to_string()),
                    ),
                ]),
                x: hover.x + hover.width * column as f32 / alignments.len() as f32,
                y: hover.y + hover.height * row as f32 / 2.0,
                width: hover.width / alignments.len() as f32,
                height: hover.height / 2.0,
            });
        }
    }
    targets
}

fn normalized_axis(position: f32, origin: f32, extent: f32) -> f32 {
    if extent <= f32::EPSILON {
        0.5
    } else {
        ((position - origin) / extent).clamp(0.0, 0.999_999)
    }
}

fn quantized_anchor(value: f32) -> f32 {
    if value <= 0.33 {
        0.0
    } else if value >= 0.66 {
        1.0
    } else {
        0.5
    }
}

fn round_position(value: f32) -> f64 {
    (value.round() * 100.0).round() as f64 / 100.0
}

fn numeric_value(value: f64) -> Value {
    if value.fract().abs() <= f64::EPSILON {
        Value::Integer(value as i64)
    } else {
        Value::Float(value)
    }
}

fn slot_table_numeric_value(value: &Value, path: &[&str]) -> Option<f32> {
    let (segment, rest) = path.split_first()?;
    let table = value.as_table()?;
    let next = table.get(*segment)?;
    if rest.is_empty() {
        return numeric_value_as_f32(next);
    }
    slot_table_numeric_value(next, rest)
}

fn numeric_value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .map(|value| value as f32)
        .or_else(|| value.as_integer().map(|value| value as f32))
}

fn normalized_slot_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn title_case_identifier(value: &str) -> String {
    let words = value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!(
                "{}{}",
                first.to_ascii_uppercase(),
                chars.as_str().to_ascii_lowercase()
            )
        })
        .collect::<Vec<_>>();
    if words.is_empty() {
        value.to_string()
    } else {
        words.join(" ")
    }
}

fn table_value(entries: &[(&str, Value)]) -> Value {
    Value::Table(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect(),
    )
}
