use super::*;

fn atlas_status_signal(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_progress(
    control_id: &str,
    value_percent: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Progress".into(),
        component_role: "progress-bar".into(),
        value_percent,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_tooltip(
    control_id: &str,
    text: &str,
    label_text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Tooltip".into(),
        component_role: "tooltip".into(),
        surface_variant: "workbench-tooltip".into(),
        text: text.into(),
        label_text: label_text.into(),
        value_number: 8.0,
        value_color: Color::from_rgb_u8(23, 28, 32),
        label_color: Color::from_rgb_u8(168, 179, 184),
        icon_color: Color::from_rgb_u8(37, 156, 167),
        layout_icon_size: 16.0,
        layout_content_offset_y: 48.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_dialog(
    control_id: &str,
    title: &str,
    message: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    severity: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "ConfirmDialog".into(),
        component_role: "confirm-dialog".into(),
        component_variant: severity.into(),
        surface_variant: "workbench-dialog".into(),
        text: title.into(),
        value_text: message.into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_status_chip(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_surface(
    control_id: &str,
    surface_variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: surface_variant.into(),
        border_width: 1.0,
        corner_radius: 4.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_label(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font_size: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_button(
    control_id: &str,
    text: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Button".into(),
        component_role: "button".into(),
        text: text.into(),
        button_variant: variant.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_button_state(
    control_id: &str,
    text: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = atlas_button(control_id, text, variant, x, y, width, height);
    match state {
        "hover" => node.hovered = true,
        "pressed" => node.pressed = true,
        "disabled" => node.disabled = true,
        _ => {}
    }
    node
}

fn atlas_field(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Input".into(),
        component_role: "text-input".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    };
    if state == "focus" {
        node.focused = true;
    }
    node
}

fn atlas_dropdown(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    };
    if state == "open" {
        node.popup_open = true;
    }
    node
}

fn atlas_selection(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    family: &str,
    checked: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "SelectionControl".into(),
        component_role: family.into(),
        text: text.into(),
        checked,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_segmented(
    control_id: &str,
    options: &[&str],
    selected: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "SegmentedControl".into(),
        component_role: "segmented-control".into(),
        value_text: selected.into(),
        options: crate::ui::layouts::common::model_rc(
            options
                .iter()
                .map(|option| SharedString::from(*option))
                .collect::<Vec<_>>(),
        ),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_list_row(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: &str,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "ListRow".into(),
        component_role: "list-row".into(),
        text: text.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    };
    match state {
        "selected" => node.selected = true,
        "hover" => node.hovered = true,
        _ => {}
    }
    node
}

fn atlas_tree_row(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    depth: i32,
    selected: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "TreeRow".into(),
        component_role: "tree-row".into(),
        text: text.into(),
        tree_depth: depth,
        expanded: depth == 1,
        hovered: !selected,
        selected,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn atlas_table_row(
    control_id: &str,
    cells: &[&str],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    selected: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "TableRow".into(),
        component_role: "table-row".into(),
        selected,
        options: crate::ui::layouts::common::model_rc(
            cells
                .iter()
                .map(|cell| SharedString::from(*cell))
                .collect::<Vec<_>>(),
        ),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}
