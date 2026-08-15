use super::*;
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::menu_bar::{
    workbench_menu_slot_width_from_label_width, WORKBENCH_MENU_SLOT_FONT_SIZE,
};

#[derive(Clone)]
struct MenuChromeCompositionGeneration {
    menus: ModelRc<super::super::HostMenuChromeMenuData>,
}

impl PartialEq for MenuChromeCompositionGeneration {
    fn eq(&self, other: &Self) -> bool {
        self.menus.shares_values_with(&other.menus)
    }
}

pub(super) fn menu_chrome_nodes(
    menus: &ModelRc<super::super::HostMenuChromeMenuData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    for row in 0..menus.row_count() {
        if let Some(menu) = menus.row_data(row) {
            text_overrides.insert(format!("{MENU_SLOT_PREFIX}{row}"), menu.label.to_string());
        }
    }

    let Ok(projection) = build_view_template_node_projection(
        "host.menu.chrome",
        MENU_CHROME_ASSET,
        &[],
        UiSize::new(width.max(0.0), height.max(0.0)),
        &text_overrides,
    ) else {
        return fallback_menu_chrome_nodes(menus, width, height);
    };
    let generation = MenuChromeCompositionGeneration {
        menus: menus.clone(),
    };
    let nodes = compose_view_template_node_model(
        "host.menu.chrome.composition",
        projection,
        &generation,
        |nodes| {
            *nodes = expand_menu_chrome_slot_nodes(std::mem::take(nodes), menus);
        },
    );
    if nodes.row_count() == 0 || control_frame(&nodes, "MenuSlot0").width <= 0.0 {
        return fallback_menu_chrome_nodes(menus, width, height);
    }
    nodes
}

pub(super) fn menu_control_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    count: usize,
) -> ModelRc<HostChromeControlFrameData> {
    control_frames(nodes, MENU_SLOT_PREFIX, count)
}

fn fallback_menu_chrome_nodes(
    menus: &ModelRc<super::super::HostMenuChromeMenuData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let slot_count = menus.row_count().max(MENU_SLOT_COUNT);
    let mut nodes = Vec::with_capacity(slot_count + 1);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackWorkbenchMenuTopBar".into(),
        control_id: MENU_TOP_BAR_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: height.max(24.0),
        },
        ..ViewTemplateNodeData::default()
    });

    let mut x = 8.0;
    for row in 0..slot_count {
        let label = menus
            .row_data(row)
            .map(|menu| menu.label)
            .unwrap_or_default();
        let slot_width = menu_slot_width(label.as_str());
        nodes.push(ViewTemplateNodeData {
            node_id: format!("FallbackMenuSlot{row}").into(),
            control_id: format!("{MENU_SLOT_PREFIX}{row}").into(),
            role: "Button".into(),
            text: label,
            text_tone: "default".into(),
            font_size: WORKBENCH_MENU_SLOT_FONT_SIZE,
            font_weight: 500,
            surface_variant: "".into(),
            button_variant: "ghost".into(),
            frame: ViewTemplateFrameData {
                x,
                y: 2.0,
                width: slot_width,
                height: (height - 4.0).clamp(20.0, 24.0),
            },
            ..ViewTemplateNodeData::default()
        });
        x += slot_width + 4.0;
    }

    model_rc(nodes)
}

fn expand_menu_chrome_slot_nodes(
    raw_nodes: Vec<ViewTemplateNodeData>,
    menus: &ModelRc<super::super::HostMenuChromeMenuData>,
) -> Vec<ViewTemplateNodeData> {
    let mut output_nodes = Vec::new();
    let mut slot_templates = BTreeMap::new();

    for node in raw_nodes {
        if let Some(row) = slot_index(node.control_id.as_str(), MENU_SLOT_PREFIX) {
            retain_dominant_control_template(&mut slot_templates, row, node);
        } else {
            output_nodes.push(node);
        }
    }
    if slot_templates.is_empty() {
        return output_nodes;
    }

    let slot_count = menus.row_count().max(MENU_SLOT_COUNT);
    let gap = menu_slot_gap(&slot_templates).unwrap_or(2.0);
    let mut next_x = slot_templates
        .get(&0)
        .map(|node| node.frame.x)
        .unwrap_or(8.0);
    for row in 0..slot_count {
        let template_index = row.min(MENU_SLOT_COUNT - 1);
        let Some(mut node) = slot_templates.get(&template_index).cloned() else {
            continue;
        };
        let label = menus
            .row_data(row)
            .map(|menu| menu.label.to_string())
            .unwrap_or_default();
        node.node_id = format!("{MENU_SLOT_PREFIX}{row}").into();
        node.control_id = format!("{MENU_SLOT_PREFIX}{row}").into();
        node.text = label.clone().into();
        node.font_size = WORKBENCH_MENU_SLOT_FONT_SIZE;
        node.frame.x = next_x;
        node.frame.width = menu_slot_width(&label);
        next_x = node.frame.x + node.frame.width + gap;
        output_nodes.push(node);
    }
    output_nodes
}

fn menu_slot_width(label: &str) -> f32 {
    let label_width = measure_runtime_text_width(label, WORKBENCH_MENU_SLOT_FONT_SIZE);
    workbench_menu_slot_width_from_label_width(label_width)
}

fn menu_slot_gap(templates: &BTreeMap<usize, ViewTemplateNodeData>) -> Option<f32> {
    let ordered = templates.values().collect::<Vec<_>>();
    ordered.windows(2).rev().find_map(|pair| {
        let gap = pair[1].frame.x - (pair[0].frame.x + pair[0].frame.width);
        (gap > 0.0).then_some(gap)
    })
}

#[cfg(test)]
pub(super) fn menu_popup_nodes(
    items: &ModelRc<super::super::HostMenuChromeItemData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    for row in 0..items.row_count().min(MENU_POPUP_ITEM_COUNT) {
        if let Some(item) = items.row_data(row) {
            text_overrides.insert(
                format!("{MENU_POPUP_ITEM_LABEL_PREFIX}{row}"),
                item.label.to_string(),
            );
            text_overrides.insert(
                format!("{MENU_POPUP_ITEM_SHORTCUT_PREFIX}{row}"),
                item.shortcut.to_string(),
            );
        }
    }

    let raw_nodes = raw_template_nodes(
        "host.menu.popup",
        MENU_POPUP_ASSET,
        width,
        height,
        &text_overrides,
    );
    model_rc(expand_menu_popup_item_nodes(model_nodes(&raw_nodes), items))
}

#[cfg(test)]
fn expand_menu_popup_item_nodes(
    raw_nodes: Vec<ViewTemplateNodeData>,
    items: &ModelRc<super::super::HostMenuChromeItemData>,
) -> Vec<ViewTemplateNodeData> {
    let mut output_nodes = Vec::new();
    let mut row_templates = BTreeMap::new();
    let mut label_templates = BTreeMap::new();
    let mut shortcut_templates = BTreeMap::new();

    for node in raw_nodes {
        if let Some(row) = slot_index(node.control_id.as_str(), MENU_POPUP_ITEM_ROW_PREFIX) {
            retain_dominant_control_template(&mut row_templates, row, node);
        } else if let Some(row) = slot_index(node.control_id.as_str(), MENU_POPUP_ITEM_LABEL_PREFIX)
        {
            retain_dominant_control_template(&mut label_templates, row, node);
        } else if let Some(row) =
            slot_index(node.control_id.as_str(), MENU_POPUP_ITEM_SHORTCUT_PREFIX)
        {
            retain_dominant_control_template(&mut shortcut_templates, row, node);
        } else {
            output_nodes.push(node);
        }
    }

    let row_step = indexed_row_step(&row_templates, MENU_POPUP_ROW_STEP_FALLBACK_PX);
    for item_index in 0..items.row_count() {
        let Some(item) = items.row_data(item_index) else {
            continue;
        };

        if let Some(node) = indexed_slot_node(
            &row_templates,
            MENU_POPUP_ITEM_ROW_PREFIX,
            MENU_POPUP_ITEM_COUNT,
            item_index,
            row_step,
            None,
        ) {
            output_nodes.push(node);
        }
        if let Some(mut label_node) = indexed_slot_node(
            &label_templates,
            MENU_POPUP_ITEM_LABEL_PREFIX,
            MENU_POPUP_ITEM_COUNT,
            item_index,
            row_step,
            Some(item.label.as_str()),
        ) {
            apply_template_icon(&mut label_node, &menu_popup_item_icon_name(&item));
            if !item.enabled {
                label_node.text_tone = "muted".into();
            }
            output_nodes.push(label_node);
        }
        if let Some(mut shortcut_node) = indexed_slot_node(
            &shortcut_templates,
            MENU_POPUP_ITEM_SHORTCUT_PREFIX,
            MENU_POPUP_ITEM_COUNT,
            item_index,
            row_step,
            Some(item.shortcut.as_str()),
        ) {
            if !item.enabled {
                shortcut_node.text_tone = "muted".into();
            }
            output_nodes.push(shortcut_node);
        }
    }

    output_nodes
}

#[cfg(test)]
fn menu_popup_item_icon_name(item: &super::super::HostMenuChromeItemData) -> String {
    let action = item.action_id.as_str();
    if let Some(icon) = normalized_chrome_icon_key(action) {
        return icon;
    }
    match action {
        "workbench.project.open" => "folder-open-outline",
        "workbench.scene.open" => "cube-outline",
        "workbench.scene.create" => "add-outline",
        "workbench.project.save" | "workbench.layout.save" => "save-outline",
        "workbench.layout.reset" => "sync-outline",
        "workbench.play_mode.enter" => "play-outline",
        "workbench.play_mode.exit" => "remove-outline",
        "workbench.history.undo" => "chevron-back-outline",
        "workbench.history.redo" => "chevron-forward-outline",
        "workbench.selection.delete_selected" => "remove-outline",
        _ if action.starts_with("workbench.layout.preset.save.") => "save-outline",
        _ if action.starts_with("workbench.layout.preset.load.") => "folder-open-outline",
        _ if action.starts_with("workbench.scene.node.create.") => {
            scene_create_menu_icon_name(action)
        }
        _ if action.starts_with("workbench.view.open.") => open_view_menu_icon_name(action),
        _ => menu_label_icon_name(item.label.as_str()),
    }
    .to_string()
}

#[cfg(test)]
fn scene_create_menu_icon_name(action: &str) -> &'static str {
    match action
        .strip_prefix("workbench.scene.node.create.")
        .unwrap_or_default()
    {
        "cube" => "cube-outline",
        "camera" => "scan-outline",
        "ambient_light" | "directional_light" | "point_light" | "rect_light" | "spot_light" => {
            "color-fill-outline"
        }
        _ => "add-outline",
    }
}

#[cfg(test)]
fn open_view_menu_icon_name(action: &str) -> &'static str {
    let descriptor = action
        .strip_prefix("workbench.view.open.")
        .unwrap_or_default()
        .replace('-', "_")
        .to_lowercase();
    match descriptor.as_str() {
        "editor.project" => "albums-outline",
        "editor.hierarchy" => "layers-outline",
        "editor.inspector" => "options-outline",
        "editor.scene" => "cube-outline",
        "editor.game" => "game-controller-outline",
        "editor.assets" | "editor.asset_browser" => "folder-open-outline",
        "editor.console" => "terminal-outline",
        "editor.runtime_diagnostics" => "grid-outline",
        "editor.module_plugins" => "git-network-outline",
        "editor.build_export_desktop" => "share-outline",
        "editor.prefab" => "cube-outline",
        _ => "ellipse-outline",
    }
}

#[cfg(test)]
fn menu_label_icon_name(label: &str) -> &'static str {
    let label = label.to_lowercase();
    if label.contains("open") {
        "folder-open-outline"
    } else if label.contains("save") {
        "save-outline"
    } else if label.contains("reset") || label.contains("reload") || label.contains("refresh") {
        "sync-outline"
    } else if label.contains("play") {
        "play-outline"
    } else if label.contains("delete") || label.contains("remove") {
        "remove-outline"
    } else if label.contains("guide") || label.contains("help") {
        "construct-outline"
    } else {
        "ellipse-outline"
    }
}
