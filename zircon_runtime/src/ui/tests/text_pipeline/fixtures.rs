use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::UiStateFlags,
    layout::{AxisConstraint, BoxConstraints, StretchMode},
    tree::UiTemplateNodeMetadata,
};

pub(super) fn repeated_text_metadata() -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(
            r#"
text = "Repeated text label"
font_size = 10.0
line_height = 12.0
wrap = "Word"
"#,
        )
        .expect("text metadata should parse"),
        ..Default::default()
    }
}

pub(super) fn button_metadata(text: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Button".to_string(),
        attributes: toml::from_str(&format!(
            r##"
label = "{text}"
kind = "secondary"
"##
        ))
        .expect("button metadata should parse"),
        ..Default::default()
    }
}

pub(super) fn text_metadata(text: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(&format!(
            r#"
text = "{text}"
font_size = 10.0
line_height = 12.0
wrap = "None"
"#
        ))
        .expect("text metadata should parse"),
        ..Default::default()
    }
}

pub(super) fn rich_text_metadata(text: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(&format!(
            r#"
text = "{text}"
font_size = 10.0
line_height = 12.0
wrap = "None"
rich_text_format = "markdown"
"#
        ))
        .expect("rich text metadata should parse"),
        ..Default::default()
    }
}

pub(super) fn vertical_text_metadata(text: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(&format!(
            r#"
text = "{text}"
font_size = 10.0
line_height = 12.0
wrap = "None"
writing_mode = "vertical-rl"
"#
        ))
        .expect("vertical text metadata should parse"),
        ..Default::default()
    }
}

pub(super) fn visible_text_state(visible: bool) -> UiStateFlags {
    UiStateFlags {
        visible,
        enabled: true,
        ..UiStateFlags::default()
    }
}

pub(super) fn fixed_constraints(width: f32, height: f32) -> BoxConstraints {
    BoxConstraints {
        width: fixed_axis(width),
        height: fixed_axis(height),
    }
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

pub(super) fn text_layout_command_count(surface: &UiSurface) -> usize {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| command.text_layout.is_some())
        .count()
}
