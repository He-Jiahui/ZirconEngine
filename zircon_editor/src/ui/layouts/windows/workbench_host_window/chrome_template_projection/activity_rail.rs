use super::*;

#[derive(Clone)]
struct ActivityRailCompositionGeneration {
    tabs: ModelRc<TabData>,
    shell_preset_id: SharedString,
}

impl PartialEq for ActivityRailCompositionGeneration {
    fn eq(&self, other: &Self) -> bool {
        self.tabs.shares_values_with(&other.tabs) && self.shell_preset_id == other.shell_preset_id
    }
}

pub(super) fn activity_rail_nodes(
    surface_id: &str,
    tabs: &ModelRc<TabData>,
    shell_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let Ok(projection) = build_view_template_node_projection(
        surface_id,
        ACTIVITY_RAIL_ASSET,
        &[],
        UiSize::new(width.max(0.0), height.max(0.0)),
        &BTreeMap::new(),
    ) else {
        return fallback_activity_rail_nodes(tabs, shell_preset_id, width, height);
    };
    let generation = ActivityRailCompositionGeneration {
        tabs: tabs.clone(),
        shell_preset_id: shell_preset_id.clone(),
    };
    compose_view_template_node_model(
        &format!("{surface_id}.composition"),
        projection,
        &generation,
        |nodes| {
            *nodes =
                expand_activity_rail_button_nodes(std::mem::take(nodes), tabs, shell_preset_id);
        },
    )
}

pub(super) fn activity_rail_button_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeControlFrameData> {
    control_frames(nodes, ACTIVITY_RAIL_BUTTON_PREFIX, tabs.row_count())
}

pub(super) fn activity_rail_active_control_id(tabs: &ModelRc<TabData>) -> SharedString {
    tabs.iter()
        .position(|tab| tab.active)
        .map(|row| format!("{ACTIVITY_RAIL_BUTTON_PREFIX}{row}").into())
        .unwrap_or_default()
}

fn expand_activity_rail_button_nodes(
    raw_nodes: Vec<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
    shell_preset_id: &SharedString,
) -> Vec<ViewTemplateNodeData> {
    let mut output_nodes = Vec::new();
    let mut button_templates = BTreeMap::new();
    let mut icon_templates = BTreeMap::new();

    for node in raw_nodes {
        if let Some(row) = slot_index(node.control_id.as_str(), ACTIVITY_RAIL_BUTTON_ICON_PREFIX) {
            retain_dominant_control_template(&mut icon_templates, row, node);
        } else if let Some(row) = slot_index(node.control_id.as_str(), ACTIVITY_RAIL_BUTTON_PREFIX)
        {
            retain_dominant_control_template(&mut button_templates, row, node);
        } else {
            output_nodes.push(node);
        }
    }

    let row_step = indexed_row_step(&button_templates, ACTIVITY_RAIL_ROW_STEP_FALLBACK_PX);
    for item_index in 0..tabs.row_count() {
        let Some(tab) = tabs.get(item_index) else {
            continue;
        };

        if let Some(mut button_node) = indexed_slot_node(
            &button_templates,
            ACTIVITY_RAIL_BUTTON_PREFIX,
            ACTIVITY_RAIL_STENCIL_COUNT,
            item_index,
            row_step,
            None,
        ) {
            button_node.surface_variant = if tab.active { "inset" } else { "transparent" }.into();
            button_node.selected = tab.active;
            button_node.focused = false;
            output_nodes.push(button_node);
        }
        if let Some(mut icon_node) = indexed_slot_node(
            &icon_templates,
            ACTIVITY_RAIL_BUTTON_ICON_PREFIX,
            ACTIVITY_RAIL_STENCIL_COUNT,
            item_index,
            row_step,
            None,
        ) {
            let icon_name = chrome_tab_icon_name(tab);
            icon_node.text = "".into();
            apply_template_icon(&mut icon_node, &icon_name);
            icon_node.selected = tab.active;
            icon_node.focused = false;
            if tab.active {
                icon_node.text_tone = "default".into();
                icon_node.font_weight = 700;
            } else if shell_preset_id.as_str() == "jetbrains_shell" {
                icon_node.text_tone = "subtle".into();
                icon_node.font_weight = 600;
            } else {
                icon_node.text_tone = "muted".into();
                icon_node.font_weight = 600;
            }
            output_nodes.push(icon_node);
        }
    }

    output_nodes
}

pub(super) fn fallback_activity_rail_nodes(
    tabs: &ModelRc<TabData>,
    shell_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let rail_width = width.max(RAIL_WIDTH_PX);
    let mut nodes = Vec::with_capacity(tabs.row_count() * 2 + 1);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackActivityRailPanel".into(),
        control_id: "ActivityRailPanel".into(),
        role: "Panel".into(),
        surface_variant: "shell".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: rail_width,
            height: height.max(1.0),
        },
        ..ViewTemplateNodeData::default()
    });

    let row_step = ACTIVITY_RAIL_ROW_STEP_FALLBACK_PX;
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.get(row) else {
            continue;
        };
        let y = 4.0 + row as f32 * row_step;
        let button_height = (row_step - 3.0).max(24.0);
        nodes.push(ViewTemplateNodeData {
            node_id: format!("FallbackActivityRailButton{row}").into(),
            control_id: format!("{ACTIVITY_RAIL_BUTTON_PREFIX}{row}").into(),
            role: "Button".into(),
            surface_variant: if tab.active { "inset" } else { "transparent" }.into(),
            button_variant: "ghost".into(),
            corner_radius: fallback_chrome_control_radius(),
            selected: tab.active,
            focused: false,
            frame: ViewTemplateFrameData {
                x: 3.0,
                y,
                width: (rail_width - 6.0).max(1.0),
                height: button_height,
            },
            ..ViewTemplateNodeData::default()
        });

        let mut icon_node = ViewTemplateNodeData {
            node_id: format!("FallbackActivityRailButtonIcon{row}").into(),
            control_id: format!("{ACTIVITY_RAIL_BUTTON_ICON_PREFIX}{row}").into(),
            role: "SvgIcon".into(),
            text_tone: if tab.active {
                "default".into()
            } else if shell_preset_id.as_str() == "jetbrains_shell" {
                "subtle".into()
            } else {
                "muted".into()
            },
            font_weight: if tab.active { 700 } else { 600 },
            selected: tab.active,
            focused: false,
            frame: ViewTemplateFrameData {
                x: (rail_width - 18.0) * 0.5,
                y: y + (button_height - 18.0) * 0.5,
                width: 18.0,
                height: 18.0,
            },
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut icon_node, &chrome_tab_icon_name(&tab));
        nodes.push(icon_node);
    }

    model_rc(nodes)
}
