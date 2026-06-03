use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
#[cfg(test)]
use super::style_selector::WORKBENCH_ICON_PANEL_RADIUS as ICON_PANEL_RADIUS;
use super::style_selector::{
    select_workbench_icon_button_style, WorkbenchIconButtonContext as IconButtonContext,
    WorkbenchIconButtonStyle,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

const ICON_GRID: f32 = 16.0;

pub(super) fn push_icon_button_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_icon_button(node) {
        return false;
    }

    let rect = icon_button_paint_rect(node, rect);
    let context = icon_button_context(node);
    let style = icon_button_style(node, context);
    push_icon_button_surface(commands, &rect, clip, order, style, opacity);
    let glyph = icon_glyph_rect(node, &rect, context);
    push_icon_glyph(
        commands,
        &glyph,
        clip,
        order + 2,
        icon_glyph_kind(node),
        style.glyph,
        style.state,
        opacity,
    );
    true
}

fn is_workbench_icon_button(node: &TemplatePaneNodeData) -> bool {
    let control_id = node.control_id.as_str();
    is_component_family(node, TemplateComponentFamily::IconButton)
        && uses_workbench_visual_language(node)
        && !control_id.starts_with("WorkbenchStatus")
}

fn icon_button_context(node: &TemplatePaneNodeData) -> IconButtonContext {
    let control_id = node.control_id.as_str();
    if control_id.starts_with("WorkbenchRail") {
        IconButtonContext::Rail
    } else if control_id.starts_with("WorkbenchToolbar")
        || control_id.starts_with("WorkbenchTool")
        || control_id.starts_with("WorkbenchRun")
        || control_id.starts_with("WorkbenchLayout")
        || control_id.starts_with("WorkbenchTheme")
    {
        IconButtonContext::Toolbar
    } else {
        IconButtonContext::Panel
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IconGlyphKind {
    Menu,
    File,
    Folder,
    Save,
    Cursor,
    Move,
    Rotate,
    Scale,
    Snap,
    Play,
    ChevronDown,
    Grid,
    Sun,
    More,
    Plus,
    Trash,
    Filter,
    Cube,
    Graph,
    Image,
    Audio,
    Code,
    Eye,
    EyeOff,
    Lock,
}

fn icon_glyph_kind(node: &TemplatePaneNodeData) -> IconGlyphKind {
    let key =
        format!("{} {}", node.control_id.as_str(), node.icon_name.as_str()).to_ascii_lowercase();
    if key.contains("menu") {
        IconGlyphKind::Menu
    } else if key.contains("file-new") || key.contains("toolbarnew") {
        IconGlyphKind::File
    } else if key.contains("add") {
        IconGlyphKind::Plus
    } else if key.contains("open") || key.contains("folder") {
        IconGlyphKind::Folder
    } else if key.contains("save") {
        IconGlyphKind::Save
    } else if key.contains("select") || key.contains("cursor") {
        IconGlyphKind::Cursor
    } else if key.contains("move") {
        IconGlyphKind::Move
    } else if key.contains("rotate") {
        IconGlyphKind::Rotate
    } else if key.contains("scale") || key.contains("fullscreen") {
        IconGlyphKind::Scale
    } else if key.contains("snap") || key.contains("magnet") {
        IconGlyphKind::Snap
    } else if key.contains("play") || key.contains("runplay") || key.contains("railscene") {
        IconGlyphKind::Play
    } else if key.contains("chevron") || key.contains("overflow") || key.contains("runmode") {
        IconGlyphKind::ChevronDown
    } else if key.contains("layout")
        || key.contains("grid")
        || key.contains("columns")
        || key.contains("list")
    {
        IconGlyphKind::Grid
    } else if key.contains("theme") || key.contains("sun") || key.contains("command-palette") {
        IconGlyphKind::Sun
    } else if key.contains("delete") || key.contains("trash") {
        IconGlyphKind::Trash
    } else if key.contains("filter") {
        IconGlyphKind::Filter
    } else if key.contains("cube") {
        IconGlyphKind::Cube
    } else if key.contains("graph") {
        IconGlyphKind::Graph
    } else if key.contains("image") {
        IconGlyphKind::Image
    } else if key.contains("audio") {
        IconGlyphKind::Audio
    } else if key.contains("code") {
        IconGlyphKind::Code
    } else if key.contains("eye-off") || key.contains("eyeoff") {
        IconGlyphKind::EyeOff
    } else if key.contains("eye") {
        IconGlyphKind::Eye
    } else if key.contains("lock") {
        IconGlyphKind::Lock
    } else {
        IconGlyphKind::More
    }
}

fn push_icon_button_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchIconButtonStyle,
    opacity: f32,
) {
    let Some(background) = style.background else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        style.border,
        style.border_width,
        style.radius,
        opacity,
    ));
}

fn icon_button_style(
    node: &TemplatePaneNodeData,
    context: IconButtonContext,
) -> WorkbenchIconButtonStyle {
    select_workbench_icon_button_style(node, context)
}

fn icon_button_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

fn icon_glyph_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    context: IconButtonContext,
) -> FrameRect {
    let max_size = rect.width.min(rect.height).max(1.0);
    let default_size = match context {
        IconButtonContext::Rail => (max_size * 0.48).clamp(18.0, 24.0),
        IconButtonContext::Toolbar | IconButtonContext::Panel => {
            (max_size * 0.50).clamp(15.0, 21.0)
        }
    };
    let size = if node.value_number.is_finite() && node.value_number > 0.0 {
        node.value_number
    } else {
        default_size
    }
    .min((max_size - 6.0).max(1.0));
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

fn push_icon_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: IconGlyphKind,
    color: [u8; 4],
    state: UiPainterResolvedState,
    opacity: f32,
) {
    match kind {
        IconGlyphKind::Menu => push_menu_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::File => push_file_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Folder => push_folder_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Save => push_save_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Cursor => push_cursor_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Move => push_move_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Rotate => push_rotate_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Scale => push_scale_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Snap => push_snap_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Play => push_play_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::ChevronDown => {
            push_chevron_down_icon(commands, rect, clip, order, color, opacity)
        }
        IconGlyphKind::Grid => push_grid_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Sun => push_sun_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::More => push_more_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Plus => push_plus_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Trash => push_trash_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Filter => push_filter_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Cube => push_cube_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Graph => push_graph_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Image => push_image_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Audio => push_audio_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Code => push_code_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::Eye => push_eye_icon(commands, rect, clip, order, color, opacity),
        IconGlyphKind::EyeOff => {
            push_eye_icon(commands, rect, clip, order, color, opacity);
            push_segments(
                commands,
                rect,
                clip,
                order + 1,
                color,
                opacity,
                &[(3.0, 12.0, 10.0, 1.4)],
            );
        }
        IconGlyphKind::Lock => push_lock_icon(commands, rect, clip, order, color, opacity),
    }

    if state == UiPainterResolvedState::Pressed {
        push_segments(
            commands,
            rect,
            clip,
            order + 3,
            color,
            opacity * 0.28,
            &[(2.0, 13.0, 12.0, 1.0)],
        );
    }
}

fn push_menu_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (2.5, 4.0, 11.0, 1.5),
            (2.5, 7.5, 11.0, 1.5),
            (2.5, 11.0, 11.0, 1.5),
        ],
    );
}

fn push_file_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 2.0, 6.0, 1.2),
            (3.0, 3.0, 1.2, 10.0),
            (12.0, 5.0, 1.2, 8.0),
            (4.0, 12.0, 8.0, 1.2),
            (10.0, 3.0, 1.2, 3.0),
            (10.0, 5.0, 3.0, 1.2),
        ],
    );
}

fn push_folder_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (2.0, 5.0, 4.0, 1.2),
            (5.0, 4.0, 4.0, 1.2),
            (2.0, 6.0, 12.0, 1.2),
            (2.0, 7.0, 1.2, 5.0),
            (13.0, 7.0, 1.2, 5.0),
            (3.0, 12.0, 10.0, 1.2),
        ],
    );
}

fn push_save_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 2.5, 10.0, 1.2),
            (3.0, 3.0, 1.2, 10.0),
            (12.0, 3.0, 1.2, 10.0),
            (4.0, 12.0, 8.0, 1.2),
            (5.0, 3.0, 5.0, 3.0),
            (6.0, 9.0, 5.0, 1.2),
        ],
    );
}

fn push_cursor_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 2.0, 1.5, 10.0),
            (4.5, 4.0, 2.0, 1.4),
            (6.0, 6.0, 2.0, 1.4),
            (7.5, 8.0, 2.0, 1.4),
            (8.0, 10.0, 1.4, 3.0),
            (9.5, 12.0, 2.0, 1.4),
        ],
    );
}

fn push_move_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (7.2, 2.0, 1.6, 12.0),
            (2.0, 7.2, 12.0, 1.6),
            (6.0, 3.0, 4.0, 1.2),
            (6.0, 12.0, 4.0, 1.2),
            (3.0, 6.0, 1.2, 4.0),
            (12.0, 6.0, 1.2, 4.0),
        ],
    );
}

fn push_rotate_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 3.0, 6.0, 1.3),
            (3.0, 4.0, 1.3, 5.0),
            (4.0, 10.5, 7.0, 1.3),
            (11.0, 7.0, 1.3, 4.5),
            (9.0, 2.0, 3.5, 1.3),
            (11.0, 2.0, 1.3, 3.5),
        ],
    );
}

fn push_scale_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 3.0, 5.0, 1.3),
            (3.0, 3.0, 1.3, 5.0),
            (8.0, 8.0, 5.0, 1.3),
            (12.0, 8.0, 1.3, 5.0),
            (4.0, 11.0, 8.0, 1.3),
            (10.0, 5.0, 1.3, 7.0),
        ],
    );
}

fn push_snap_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 3.0, 1.4, 7.0),
            (11.5, 3.0, 1.4, 7.0),
            (4.0, 10.0, 3.0, 1.4),
            (9.0, 10.0, 3.0, 1.4),
            (6.8, 11.0, 2.4, 2.0),
        ],
    );
}

fn push_play_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 3.0, 2.0, 10.0),
            (6.0, 4.0, 2.0, 8.0),
            (8.0, 5.0, 2.0, 6.0),
            (10.0, 6.0, 2.0, 4.0),
        ],
    );
}

fn push_chevron_down_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 6.0, 2.0, 2.0),
            (6.0, 8.0, 4.0, 2.0),
            (10.0, 6.0, 2.0, 2.0),
        ],
    );
}

fn push_grid_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 3.0, 4.0, 4.0),
            (9.0, 3.0, 4.0, 4.0),
            (3.0, 9.0, 4.0, 4.0),
            (9.0, 9.0, 4.0, 4.0),
        ],
    );
}

fn push_sun_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (6.0, 6.0, 4.0, 4.0),
            (7.2, 2.0, 1.6, 2.4),
            (7.2, 11.6, 1.6, 2.4),
            (2.0, 7.2, 2.4, 1.6),
            (11.6, 7.2, 2.4, 1.6),
        ],
    );
}

fn push_more_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (7.0, 3.0, 2.0, 2.0),
            (7.0, 7.0, 2.0, 2.0),
            (7.0, 11.0, 2.0, 2.0),
        ],
    );
}

fn push_plus_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[(7.2, 3.0, 1.6, 10.0), (3.0, 7.2, 10.0, 1.6)],
    );
}

fn push_trash_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 5.0, 8.0, 1.2),
            (5.0, 3.0, 6.0, 1.2),
            (5.0, 6.0, 1.2, 7.0),
            (10.0, 6.0, 1.2, 7.0),
            (6.0, 12.0, 4.0, 1.2),
        ],
    );
}

fn push_filter_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 3.0, 10.0, 1.4),
            (5.0, 6.0, 6.0, 1.4),
            (7.0, 8.0, 2.0, 5.0),
        ],
    );
}

fn push_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 3.0, 8.0, 1.2),
            (3.0, 4.0, 1.2, 7.0),
            (12.0, 4.0, 1.2, 7.0),
            (4.0, 11.0, 8.0, 1.2),
            (7.5, 2.0, 1.2, 10.0),
            (3.0, 7.0, 10.0, 1.2),
        ],
    );
}

fn push_graph_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 4.0, 3.0, 3.0),
            (10.0, 3.0, 3.0, 3.0),
            (9.0, 10.0, 3.0, 3.0),
            (6.0, 5.0, 5.0, 1.2),
            (10.0, 6.0, 1.2, 5.0),
        ],
    );
}

fn push_image_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 3.0, 10.0, 1.2),
            (3.0, 4.0, 1.2, 9.0),
            (12.0, 4.0, 1.2, 9.0),
            (4.0, 12.0, 8.0, 1.2),
            (5.0, 10.0, 3.0, 1.2),
            (7.0, 8.0, 3.0, 1.2),
            (10.0, 6.0, 1.6, 1.6),
        ],
    );
}

fn push_audio_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 6.0, 3.0, 4.0),
            (6.0, 4.0, 2.0, 8.0),
            (9.0, 5.0, 1.2, 2.0),
            (11.0, 4.0, 1.2, 4.0),
            (9.0, 9.0, 1.2, 2.0),
            (11.0, 8.0, 1.2, 4.0),
        ],
    );
}

fn push_code_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 6.0, 2.0, 1.4),
            (4.0, 5.0, 1.4, 2.0),
            (4.0, 9.0, 1.4, 2.0),
            (11.0, 6.0, 2.0, 1.4),
            (10.6, 5.0, 1.4, 2.0),
            (10.6, 9.0, 1.4, 2.0),
            (7.2, 4.0, 1.2, 8.0),
        ],
    );
}

fn push_eye_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (2.5, 7.0, 2.0, 2.0),
            (4.5, 5.0, 7.0, 1.2),
            (4.5, 10.0, 7.0, 1.2),
            (11.5, 7.0, 2.0, 2.0),
            (7.0, 7.0, 2.0, 2.0),
        ],
    );
}

fn push_lock_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 7.0, 8.0, 6.0),
            (5.0, 4.0, 6.0, 1.2),
            (4.0, 5.0, 1.2, 3.0),
            (11.0, 5.0, 1.2, 3.0),
        ],
    );
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[(f32, f32, f32, f32)],
) {
    for (x, y, width, height) in segments {
        commands.push(HostPaintCommand::quad(
            scaled_rect(origin, *x, *y, *width, *height),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn scaled_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    let scale_x = origin.width / ICON_GRID;
    let scale_y = origin.height / ICON_GRID;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
    }
}

#[cfg(test)]
#[path = "template_icon_buttons_tests.rs"]
mod tests;
