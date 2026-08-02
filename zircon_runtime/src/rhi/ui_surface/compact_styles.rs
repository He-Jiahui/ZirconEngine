use std::collections::HashMap;

use super::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfaceTextStyle,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiSurfaceStyleHandle(usize);

impl UiSurfaceStyleHandle {
    pub(super) const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiSurfaceStyle {
    Quad {
        color: [u8; 4],
        corner_radius: f32,
    },
    Border {
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    },
    Text {
        color: [u8; 4],
        font_family: Option<String>,
        font_weight: u16,
        font_size: f32,
        line_height: f32,
        style: UiSurfaceTextStyle,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiSurfaceStyledPayload {
    None,
    Text(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiSurfaceResolvedCommandKind<'a> {
    Quad {
        color: [u8; 4],
        corner_radius: f32,
    },
    Border {
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    },
    Text {
        text: &'a str,
        color: [u8; 4],
        font_family: Option<&'a str>,
        font_weight: u16,
        font_size: f32,
        line_height: f32,
        style: UiSurfaceTextStyle,
    },
    Image {
        payload: &'a UiSurfaceImagePayload,
    },
    Clip,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum UiSurfaceStyleKey {
    Quad {
        color: [u8; 4],
        corner_radius_bits: u32,
    },
    Border {
        color: [u8; 4],
        width_bits: u32,
        corner_radius_bits: u32,
    },
    Text {
        color: [u8; 4],
        font_family: Option<String>,
        font_weight: u16,
        font_size_bits: u32,
        line_height_bits: u32,
        style: UiSurfaceTextStyle,
    },
}

pub(super) fn compact_commands(
    commands: Vec<UiSurfaceCommand>,
) -> (Vec<UiSurfaceCommand>, Vec<UiSurfaceStyle>) {
    let mut styles = Vec::new();
    let mut handles = HashMap::<UiSurfaceStyleKey, UiSurfaceStyleHandle>::new();
    let commands = commands
        .into_iter()
        .map(|mut command| {
            command.kind = compact_command_kind(command.kind, &mut styles, &mut handles);
            command
        })
        .collect();
    (commands, styles)
}

pub(super) fn resolved_kind<'a>(
    draw_list: &'a UiSurfaceDrawList,
    command: &'a UiSurfaceCommand,
) -> Option<UiSurfaceResolvedCommandKind<'a>> {
    match &command.kind {
        UiSurfaceCommandKind::Quad {
            color,
            corner_radius,
        } => Some(UiSurfaceResolvedCommandKind::Quad {
            color: *color,
            corner_radius: *corner_radius,
        }),
        UiSurfaceCommandKind::Border {
            color,
            width,
            corner_radius,
        } => Some(UiSurfaceResolvedCommandKind::Border {
            color: *color,
            width: *width,
            corner_radius: *corner_radius,
        }),
        UiSurfaceCommandKind::Text {
            text,
            color,
            font_family,
            font_weight,
            font_size,
            line_height,
            style,
        } => Some(UiSurfaceResolvedCommandKind::Text {
            text,
            color: *color,
            font_family: font_family.as_deref(),
            font_weight: *font_weight,
            font_size: *font_size,
            line_height: *line_height,
            style: *style,
        }),
        UiSurfaceCommandKind::Image { payload } => {
            Some(UiSurfaceResolvedCommandKind::Image { payload })
        }
        UiSurfaceCommandKind::Styled { style, payload } => {
            let style = draw_list.styles.get(style.index())?;
            match (style, payload) {
                (
                    UiSurfaceStyle::Quad {
                        color,
                        corner_radius,
                    },
                    UiSurfaceStyledPayload::None,
                ) => Some(UiSurfaceResolvedCommandKind::Quad {
                    color: *color,
                    corner_radius: *corner_radius,
                }),
                (
                    UiSurfaceStyle::Border {
                        color,
                        width,
                        corner_radius,
                    },
                    UiSurfaceStyledPayload::None,
                ) => Some(UiSurfaceResolvedCommandKind::Border {
                    color: *color,
                    width: *width,
                    corner_radius: *corner_radius,
                }),
                (
                    UiSurfaceStyle::Text {
                        color,
                        font_family,
                        font_weight,
                        font_size,
                        line_height,
                        style,
                    },
                    UiSurfaceStyledPayload::Text(text),
                ) => Some(UiSurfaceResolvedCommandKind::Text {
                    text,
                    color: *color,
                    font_family: font_family.as_deref(),
                    font_weight: *font_weight,
                    font_size: *font_size,
                    line_height: *line_height,
                    style: *style,
                }),
                _ => None,
            }
        }
        UiSurfaceCommandKind::Clip => Some(UiSurfaceResolvedCommandKind::Clip),
    }
}

fn compact_command_kind(
    kind: UiSurfaceCommandKind,
    styles: &mut Vec<UiSurfaceStyle>,
    handles: &mut HashMap<UiSurfaceStyleKey, UiSurfaceStyleHandle>,
) -> UiSurfaceCommandKind {
    let (style, payload) = match kind {
        UiSurfaceCommandKind::Quad {
            color,
            corner_radius,
        } => (
            UiSurfaceStyle::Quad {
                color,
                corner_radius,
            },
            UiSurfaceStyledPayload::None,
        ),
        UiSurfaceCommandKind::Border {
            color,
            width,
            corner_radius,
        } => (
            UiSurfaceStyle::Border {
                color,
                width,
                corner_radius,
            },
            UiSurfaceStyledPayload::None,
        ),
        UiSurfaceCommandKind::Text {
            text,
            color,
            font_family,
            font_weight,
            font_size,
            line_height,
            style,
        } => (
            UiSurfaceStyle::Text {
                color,
                font_family,
                font_weight,
                font_size,
                line_height,
                style,
            },
            UiSurfaceStyledPayload::Text(text),
        ),
        other => return other,
    };
    let key = UiSurfaceStyleKey::from_style(&style);
    let handle = *handles.entry(key).or_insert_with(|| {
        let handle = UiSurfaceStyleHandle(styles.len());
        styles.push(style);
        handle
    });
    UiSurfaceCommandKind::Styled {
        style: handle,
        payload,
    }
}

impl UiSurfaceStyleKey {
    fn from_style(style: &UiSurfaceStyle) -> Self {
        match style {
            UiSurfaceStyle::Quad {
                color,
                corner_radius,
            } => Self::Quad {
                color: *color,
                corner_radius_bits: corner_radius.to_bits(),
            },
            UiSurfaceStyle::Border {
                color,
                width,
                corner_radius,
            } => Self::Border {
                color: *color,
                width_bits: width.to_bits(),
                corner_radius_bits: corner_radius.to_bits(),
            },
            UiSurfaceStyle::Text {
                color,
                font_family,
                font_weight,
                font_size,
                line_height,
                style,
            } => Self::Text {
                color: *color,
                font_family: font_family.clone(),
                font_weight: *font_weight,
                font_size_bits: font_size.to_bits(),
                line_height_bits: line_height.to_bits(),
                style: *style,
            },
        }
    }
}
