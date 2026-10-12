pub(super) fn default_popper_placement(component_role: &str) -> &'static str {
    match component_role {
        "tooltip" => "top",
        "menu" => "bottom-start",
        "popper" => "bottom-start",
        _ => "bottom",
    }
}

pub(super) fn popper_position(
    placement: &str,
    component_role: &str,
    anchor_x: f32,
    anchor_y: f32,
    anchor_width: f32,
    anchor_height: f32,
    width: f32,
    height: f32,
) -> (f32, f32) {
    let (side, align) = split_placement(placement);
    let gap = if component_role == "tooltip" {
        8.0
    } else {
        0.0
    };
    match side {
        "top" => (
            horizontal_aligned(anchor_x, anchor_width, width, align),
            anchor_y - height - gap,
        ),
        "left" => (
            anchor_x - width - gap,
            vertical_aligned(anchor_y, anchor_height, height, align),
        ),
        "right" => (
            anchor_x + anchor_width + gap,
            vertical_aligned(anchor_y, anchor_height, height, align),
        ),
        _ => (
            horizontal_aligned(anchor_x, anchor_width, width, align),
            anchor_y + anchor_height + gap,
        ),
    }
}

fn split_placement(placement: &str) -> (&str, &str) {
    placement.split_once('-').unwrap_or((placement, "center"))
}

fn horizontal_aligned(anchor_x: f32, anchor_width: f32, width: f32, align: &str) -> f32 {
    match align {
        "start" | "left" => anchor_x,
        "end" | "right" => anchor_x + anchor_width - width,
        _ => anchor_x + anchor_width * 0.5 - width * 0.5,
    }
}

fn vertical_aligned(anchor_y: f32, anchor_height: f32, height: f32, align: &str) -> f32 {
    match align {
        "start" | "top" => anchor_y,
        "end" | "bottom" => anchor_y + anchor_height - height,
        _ => anchor_y + anchor_height * 0.5 - height * 0.5,
    }
}
