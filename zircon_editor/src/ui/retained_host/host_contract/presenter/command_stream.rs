use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::painter::{
    draw_rect_clipped, draw_rgba_image_clipped_with_resource_key, draw_rounded_border_clipped,
    draw_rounded_rect_clipped, draw_text_with_size_and_style, record_host_frame_commands,
    HostRecordedPaintCommand, HostRecordedPaintKind, HostRgbaFrame,
};

const FALLBACK_IMAGE_COLOR: [u8; 4] = [42, 58, 78, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChromeCommandLayer {
    Static,
    Dynamic,
    Text,
    Viewport,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ChromeCommandKind {
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
        text: String,
        color: [u8; 4],
        size: f32,
        line_height: f32,
        style: UiTextRunPaintStyle,
    },
    Image {
        payload: ChromeImagePayload,
    },
    Clip,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ChromeImagePayload {
    pub(super) resource_key: String,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) upload_bytes: u64,
    pub(super) rgba: Option<Vec<u8>>,
    pub(super) atlas_uv: Option<ChromeImageUvRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ChromeImageUvRect {
    pub(super) min: [f32; 2],
    pub(super) max: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ChromeCommand {
    pub(super) layer: ChromeCommandLayer,
    pub(super) z_index: i32,
    pub(super) frame: FrameRect,
    pub(super) clip: Option<FrameRect>,
    pub(super) kind: ChromeCommandKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ChromeCommandStreamStats {
    pub(super) command_count: usize,
    pub(super) static_command_count: usize,
    pub(super) dynamic_command_count: usize,
    pub(super) text_command_count: usize,
    pub(super) image_command_count: usize,
    pub(super) clip_command_count: usize,
    pub(super) image_upload_bytes: u64,
    pub(super) draw_call_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ChromeCommandStream {
    surface_size: (u32, u32),
    damage: Option<FrameRect>,
    full_rebuild: bool,
    commands: Vec<ChromeCommand>,
}

impl ChromeCommandStream {
    pub(super) fn full_rebuild(surface_size: (u32, u32)) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: None,
            full_rebuild: true,
            commands: Vec::new(),
        }
    }

    pub(super) fn patch(surface_size: (u32, u32), damage: FrameRect) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: Some(damage),
            full_rebuild: false,
            commands: Vec::new(),
        }
    }

    pub(super) fn is_full_rebuild(&self) -> bool {
        self.full_rebuild
    }

    pub(super) fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub(super) fn damage(&self) -> Option<&FrameRect> {
        self.damage.as_ref()
    }

    pub(super) fn commands(&self) -> &[ChromeCommand] {
        &self.commands
    }

    pub(super) fn push_quad(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        color: [u8; 4],
        corner_radius: f32,
    ) {
        self.push_command(
            layer,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Quad {
                color,
                corner_radius,
            },
        );
    }

    pub(super) fn push_border(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    ) {
        self.push_command(
            layer,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Border {
                color,
                width,
                corner_radius,
            },
        );
    }

    pub(super) fn push_text(
        &mut self,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        text: impl Into<String>,
        color: [u8; 4],
        size: f32,
    ) {
        self.push_command(
            ChromeCommandLayer::Text,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Text {
                text: text.into(),
                color,
                size,
                line_height: size.max(1.0) * 1.2,
                style: UiTextRunPaintStyle::default(),
            },
        );
    }

    pub(super) fn push_image(
        &mut self,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        payload: ChromeImagePayload,
    ) {
        self.push_command(
            ChromeCommandLayer::Viewport,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Image { payload },
        );
    }

    pub(super) fn push_clip(&mut self, layer: ChromeCommandLayer, z_index: i32, frame: FrameRect) {
        self.push_command(
            layer,
            z_index,
            frame.clone(),
            Some(frame),
            ChromeCommandKind::Clip,
        );
    }

    pub(super) fn stats(&self) -> ChromeCommandStreamStats {
        let mut stats = ChromeCommandStreamStats {
            command_count: self.commands.len(),
            ..ChromeCommandStreamStats::default()
        };
        for command in &self.commands {
            match command.layer {
                ChromeCommandLayer::Static => stats.static_command_count += 1,
                ChromeCommandLayer::Dynamic => stats.dynamic_command_count += 1,
                ChromeCommandLayer::Text => stats.text_command_count += 1,
                ChromeCommandLayer::Viewport => stats.dynamic_command_count += 1,
            }
            match &command.kind {
                ChromeCommandKind::Quad { .. }
                | ChromeCommandKind::Border { .. }
                | ChromeCommandKind::Text { .. } => stats.draw_call_count += 1,
                ChromeCommandKind::Image { payload } => {
                    stats.image_command_count += 1;
                    stats.image_upload_bytes = stats
                        .image_upload_bytes
                        .saturating_add(payload.upload_bytes);
                    stats.draw_call_count += 1;
                }
                ChromeCommandKind::Clip => stats.clip_command_count += 1,
            }
        }
        stats
    }

    fn push_recorded_command(
        &mut self,
        command: HostRecordedPaintCommand,
        include_image_bytes: bool,
    ) {
        let layer = match &command.kind {
            HostRecordedPaintKind::Text { .. } => ChromeCommandLayer::Text,
            HostRecordedPaintKind::Image { .. } => ChromeCommandLayer::Viewport,
            HostRecordedPaintKind::Quad { .. } | HostRecordedPaintKind::Border { .. } => {
                if self.full_rebuild {
                    ChromeCommandLayer::Static
                } else {
                    ChromeCommandLayer::Dynamic
                }
            }
        };
        let kind = match command.kind {
            HostRecordedPaintKind::Quad {
                color,
                corner_radius,
            } => ChromeCommandKind::Quad {
                color,
                corner_radius,
            },
            HostRecordedPaintKind::Border {
                color,
                width,
                corner_radius,
            } => ChromeCommandKind::Border {
                color,
                width,
                corner_radius,
            },
            HostRecordedPaintKind::Text {
                text,
                color,
                font_size,
                line_height,
                style,
            } => ChromeCommandKind::Text {
                text,
                color,
                size: font_size,
                line_height,
                style,
            },
            HostRecordedPaintKind::Image {
                resource_key,
                width,
                height,
                rgba,
            } => {
                let upload_bytes = rgba
                    .as_ref()
                    .map(|rgba| rgba.len() as u64)
                    .unwrap_or_else(|| u64::from(width) * u64::from(height) * 4);
                ChromeCommandKind::Image {
                    payload: ChromeImagePayload {
                        resource_key,
                        width,
                        height,
                        upload_bytes,
                        rgba: include_image_bytes.then_some(rgba).flatten(),
                        atlas_uv: None,
                    },
                }
            }
        };
        self.push_command(
            layer,
            command.z_index,
            command.frame,
            command.clip_frame,
            kind,
        );
    }

    fn push_command(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        kind: ChromeCommandKind,
    ) {
        if !visible_frame(&frame) {
            return;
        }
        self.commands.push(ChromeCommand {
            layer,
            z_index,
            frame,
            clip,
            kind,
        });
    }
}

pub(super) fn build_chrome_command_stream(
    presentation: &HostWindowPresentationData,
    surface_size: (u32, u32),
    damage: Option<&FrameRect>,
    include_image_bytes: bool,
) -> ChromeCommandStream {
    let surface_size = clamp_surface_size(surface_size);
    let (recorded_commands, clipped_damage) =
        record_host_frame_commands(surface_size.0, surface_size.1, presentation, damage);
    let mut stream = if let Some(damage) = clipped_damage.clone() {
        ChromeCommandStream::patch(surface_size, damage.clone())
    } else {
        ChromeCommandStream::full_rebuild(surface_size)
    };
    if let Some(damage) = clipped_damage {
        stream.push_clip(ChromeCommandLayer::Dynamic, 0, damage);
    }
    for command in recorded_commands {
        stream.push_recorded_command(command, include_image_bytes);
    }
    stream
}

pub(super) fn paint_chrome_command_stream_to_frame(
    width: u32,
    height: u32,
    stream: &ChromeCommandStream,
) -> HostRgbaFrame {
    let mut frame = HostRgbaFrame::filled(width, height, [0, 0, 0, 255]);
    paint_chrome_command_stream_into_frame(&mut frame, stream);
    frame
}

pub(super) fn repaint_chrome_command_stream_region(
    frame: &mut HostRgbaFrame,
    stream: &ChromeCommandStream,
) -> Option<FrameRect> {
    let damage = stream.damage().cloned()?;
    let previous_clip = frame.replace_paint_clip(Some(damage.clone()));
    paint_chrome_command_stream_into_frame(frame, stream);
    frame.replace_paint_clip(previous_clip);
    Some(damage)
}

fn paint_chrome_command_stream_into_frame(frame: &mut HostRgbaFrame, stream: &ChromeCommandStream) {
    let mut ordered = stream.commands().iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(index, command)| (command.z_index, *index));
    for (_, command) in ordered {
        paint_chrome_command(frame, command);
    }
}

fn paint_chrome_command(frame: &mut HostRgbaFrame, command: &ChromeCommand) {
    match &command.kind {
        ChromeCommandKind::Quad {
            color,
            corner_radius,
        } => {
            if *corner_radius > 0.0 {
                draw_rounded_rect_clipped(
                    frame,
                    command.frame.clone(),
                    command.clip.as_ref(),
                    *color,
                    *corner_radius,
                )
            } else {
                draw_rect_clipped(frame, command.frame.clone(), command.clip.as_ref(), *color)
            }
        }
        ChromeCommandKind::Border {
            color,
            width,
            corner_radius,
        } => {
            if *corner_radius > 0.0 {
                draw_rounded_border_clipped(
                    frame,
                    command.frame.clone(),
                    command.clip.as_ref(),
                    *color,
                    *width,
                    *corner_radius,
                )
            } else {
                paint_border_command(frame, &command.frame, command.clip.as_ref(), *color, *width)
            }
        }
        ChromeCommandKind::Text {
            text,
            color,
            size,
            line_height,
            style,
        } => draw_text_with_size_and_style(
            frame,
            command.frame.clone(),
            text,
            command.clip.as_ref(),
            *color,
            *size,
            *line_height,
            *style,
        ),
        ChromeCommandKind::Image { payload } => {
            if let Some(rgba) = payload.rgba.as_ref() {
                let _ = draw_rgba_image_clipped_with_resource_key(
                    frame,
                    command.frame.clone(),
                    command.clip.as_ref(),
                    payload.resource_key.as_str(),
                    payload.width,
                    payload.height,
                    rgba,
                );
            } else {
                draw_rect_clipped(
                    frame,
                    command.frame.clone(),
                    command.clip.as_ref(),
                    FALLBACK_IMAGE_COLOR,
                );
            }
        }
        ChromeCommandKind::Clip => {}
    }
}

fn paint_border_command(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    width: f32,
) {
    let width = width.ceil().max(1.0);
    for offset in 0..(width as u32) {
        let offset = offset as f32;
        draw_rect_clipped(
            frame,
            FrameRect {
                x: rect.x + offset,
                y: rect.y + offset,
                width: (rect.width - offset * 2.0).max(0.0),
                height: 1.0,
            },
            clip,
            color,
        );
        draw_rect_clipped(
            frame,
            FrameRect {
                x: rect.x + offset,
                y: rect.y + rect.height - 1.0 - offset,
                width: (rect.width - offset * 2.0).max(0.0),
                height: 1.0,
            },
            clip,
            color,
        );
        draw_rect_clipped(
            frame,
            FrameRect {
                x: rect.x + offset,
                y: rect.y + offset,
                width: 1.0,
                height: (rect.height - offset * 2.0).max(0.0),
            },
            clip,
            color,
        );
        draw_rect_clipped(
            frame,
            FrameRect {
                x: rect.x + rect.width - 1.0 - offset,
                y: rect.y + offset,
                width: 1.0,
                height: (rect.height - offset * 2.0).max(0.0),
            },
            clip,
            color,
        );
    }
}

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn clamp_surface_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::ui::retained_host::host_contract::data::{
        HostClosePromptData, HostDocumentDockSurfaceData, HostWindowLayoutData, PaneData,
        TemplateNodeFrameData, TemplatePaneNodeData,
    };
    use crate::ui::retained_host::host_contract::painter::{
        paint_host_frame, repaint_host_frame_region,
    };
    use crate::ui::retained_host::primitives::{ModelRc, VecModel};

    fn presentation_with_viewport_image() -> HostWindowPresentationData {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_layout = test_layout();
        presentation.host_scene_data.layout = test_layout();
        presentation.host_scene_data.menu_chrome.template_nodes =
            model_rc(vec![template_node("ProjectAction", "Button", "Create")]);
        presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
            region_frame: FrameRect {
                x: 24.0,
                y: 50.0,
                width: 150.0,
                height: 120.0,
            },
            header_frame: FrameRect {
                x: 0.0,
                y: 0.0,
                width: 150.0,
                height: 30.0,
            },
            content_frame: FrameRect {
                x: 0.0,
                y: 32.0,
                width: 150.0,
                height: 86.0,
            },
            pane: PaneData {
                kind: "Scene".into(),
                title: "Scene".into(),
                show_toolbar: false,
                ..PaneData::default()
            },
            ..HostDocumentDockSurfaceData::default()
        };
        presentation.host_shell.project_path = "res://project".into();
        presentation.host_shell.status_secondary = "Ready".into();
        presentation.viewport_image = Some(super::super::super::data::HostViewportImageData {
            resource_key: "viewport:test-initial".into(),
            width: 2,
            height: 2,
            rgba: vec![255; 16],
        });
        presentation
    }

    fn test_layout() -> HostWindowLayoutData {
        HostWindowLayoutData {
            center_band_frame: FrameRect {
                x: 0.0,
                y: 36.0,
                width: 200.0,
                height: 144.0,
            },
            viewport_content_frame: FrameRect {
                x: 40.0,
                y: 92.0,
                width: 80.0,
                height: 60.0,
            },
            status_bar_frame: FrameRect {
                x: 0.0,
                y: 180.0,
                width: 200.0,
                height: 20.0,
            },
            document_region_frame: FrameRect {
                x: 24.0,
                y: 50.0,
                width: 150.0,
                height: 120.0,
            },
            ..HostWindowLayoutData::default()
        }
    }

    fn template_node(control_id: &str, role: &str, text: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: role.into(),
            text: text.into(),
            surface_variant: "panel".into(),
            border_width: 1.0,
            frame: TemplateNodeFrameData {
                x: 12.0,
                y: 12.0,
                width: 72.0,
                height: 24.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
        ModelRc::from(Rc::new(VecModel::from(values)))
    }

    #[test]
    fn full_command_stream_records_full_ui_draw_list() {
        let stream = build_chrome_command_stream(
            &presentation_with_viewport_image(),
            (200, 200),
            None,
            false,
        );

        let stats = stream.stats();
        assert!(stream.is_full_rebuild());
        assert!(stats.static_command_count > 0);
        assert!(stats.text_command_count > 0);
        assert!(stats.draw_call_count > 0);
        assert_eq!(stats.image_upload_bytes, 16);
        assert!(stream
            .commands()
            .iter()
            .any(|command| matches!(command.kind, ChromeCommandKind::Image { .. })));
        assert!(stream.commands().iter().any(|command| {
            matches!(
                &command.kind,
                ChromeCommandKind::Text { text, .. } if text == "Create"
            )
        }));
    }

    #[test]
    fn recorded_commands_preserve_corner_radius_in_chrome_stream() {
        let mut stream = ChromeCommandStream::full_rebuild((64, 64));
        stream.push_recorded_command(
            HostRecordedPaintCommand {
                frame: FrameRect {
                    x: 4.0,
                    y: 4.0,
                    width: 24.0,
                    height: 16.0,
                },
                clip_frame: None,
                z_index: 3,
                kind: HostRecordedPaintKind::Quad {
                    color: [10, 20, 30, 255],
                    corner_radius: 8.0,
                },
            },
            false,
        );
        stream.push_recorded_command(
            HostRecordedPaintCommand {
                frame: FrameRect {
                    x: 4.0,
                    y: 24.0,
                    width: 24.0,
                    height: 16.0,
                },
                clip_frame: None,
                z_index: 4,
                kind: HostRecordedPaintKind::Border {
                    color: [40, 50, 60, 255],
                    width: 2.0,
                    corner_radius: 7.0,
                },
            },
            false,
        );

        assert!(matches!(
            stream.commands()[0].kind,
            ChromeCommandKind::Quad {
                color: [10, 20, 30, 255],
                corner_radius: 8.0,
            }
        ));
        assert!(matches!(
            stream.commands()[1].kind,
            ChromeCommandKind::Border {
                color: [40, 50, 60, 255],
                width: 2.0,
                corner_radius: 7.0,
            }
        ));
    }

    #[test]
    fn patch_command_stream_does_not_rebuild_static_layer() {
        let damage = FrameRect {
            x: 42.0,
            y: 94.0,
            width: 10.0,
            height: 8.0,
        };

        let stream = build_chrome_command_stream(
            &presentation_with_viewport_image(),
            (200, 200),
            Some(&damage),
            false,
        );

        let stats = stream.stats();
        assert!(!stream.is_full_rebuild());
        assert_eq!(stats.static_command_count, 0);
        assert!(stats.dynamic_command_count > 0);
        assert!(stream
            .commands()
            .iter()
            .all(|command| { !matches!(command.layer, ChromeCommandLayer::Static) }));
    }

    #[test]
    fn viewport_image_patch_can_carry_upload_bytes_for_gpu() {
        let damage = FrameRect {
            x: 42.0,
            y: 94.0,
            width: 10.0,
            height: 8.0,
        };

        let stream = build_chrome_command_stream(
            &presentation_with_viewport_image(),
            (200, 200),
            Some(&damage),
            true,
        );

        let image = stream
            .commands()
            .iter()
            .find_map(|command| match &command.kind {
                ChromeCommandKind::Image { payload } => Some(payload),
                _ => None,
            })
            .expect("viewport damage should keep the viewport image command");
        assert_eq!(image.resource_key, "viewport:test-initial");
        assert_eq!(image.upload_bytes, 16);
        assert_eq!(image.rgba.as_deref(), Some(&[255; 16][..]));
        assert_eq!(image.atlas_uv, None);
    }

    #[test]
    fn command_stream_preserves_atlas_uv_on_image_payload() {
        let mut stream = ChromeCommandStream::full_rebuild((64, 64));

        stream.push_image(
            1,
            FrameRect {
                x: 4.0,
                y: 6.0,
                width: 20.0,
                height: 12.0,
            },
            None,
            ChromeImagePayload {
                resource_key: "atlas://editor/icons".to_string(),
                width: 64,
                height: 64,
                upload_bytes: 0,
                rgba: None,
                atlas_uv: Some(ChromeImageUvRect {
                    min: [0.5, 0.25],
                    max: [0.75, 0.5],
                }),
            },
        );

        let ChromeCommandKind::Image { payload } = &stream.commands()[0].kind else {
            panic!("expected image command");
        };
        assert_eq!(payload.resource_key, "atlas://editor/icons");
        assert_eq!(
            payload.atlas_uv,
            Some(ChromeImageUvRect {
                min: [0.5, 0.25],
                max: [0.75, 0.5],
            })
        );
    }

    #[test]
    fn command_stream_executor_repaints_close_prompt_without_legacy_painter() {
        let mut presentation = presentation_with_viewport_image();
        presentation.close_prompt = HostClosePromptData {
            visible: true,
            title: "Unsaved".into(),
            message: "Save changes?".into(),
            details: "Project".into(),
            can_save: true,
            overlay_frame: FrameRect {
                x: 10.0,
                y: 20.0,
                width: 160.0,
                height: 120.0,
            },
            dialog_frame: FrameRect {
                x: 30.0,
                y: 34.0,
                width: 120.0,
                height: 90.0,
            },
            save_button_frame: FrameRect {
                x: 42.0,
                y: 92.0,
                width: 32.0,
                height: 18.0,
            },
            discard_button_frame: FrameRect {
                x: 78.0,
                y: 92.0,
                width: 32.0,
                height: 18.0,
            },
            cancel_button_frame: FrameRect {
                x: 114.0,
                y: 92.0,
                width: 32.0,
                height: 18.0,
            },
            ..HostClosePromptData::default()
        };

        let stream = build_chrome_command_stream(&presentation, (200, 200), None, true);
        let frame = paint_chrome_command_stream_to_frame(200, 200, &stream);

        assert_ne!(pixel(frame.as_bytes(), 200, 12, 22), [0, 0, 0, 255]);
        assert!(stream.commands().iter().any(|command| {
            matches!(
                &command.kind,
                ChromeCommandKind::Text { text, .. } if text == "Unsaved"
            )
        }));
    }

    #[test]
    fn full_command_stream_matches_legacy_painter_pixels() {
        let presentation = presentation_with_viewport_image();
        let legacy = paint_host_frame(200, 200, &presentation);
        let stream = build_chrome_command_stream(&presentation, (200, 200), None, true);
        let replayed = paint_chrome_command_stream_to_frame(200, 200, &stream);

        assert_eq!(
            first_pixel_difference(replayed.as_bytes(), legacy.as_bytes(), 200),
            None
        );
    }

    #[test]
    fn patch_command_stream_matches_legacy_region_repaint_pixels() {
        let mut presentation = presentation_with_viewport_image();
        let damage = FrameRect {
            x: 40.0,
            y: 92.0,
            width: 80.0,
            height: 60.0,
        };
        let mut legacy = paint_host_frame(200, 200, &presentation);
        let mut replayed = paint_host_frame(200, 200, &presentation);

        presentation.viewport_image = Some(super::super::super::data::HostViewportImageData {
            resource_key: "viewport:test-patch".into(),
            width: 2,
            height: 2,
            rgba: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
            ],
        });
        let stream = build_chrome_command_stream(&presentation, (200, 200), Some(&damage), true);

        let legacy_damage = repaint_host_frame_region(&mut legacy, &presentation, &damage)
            .expect("legacy painter should repaint visible viewport damage");
        let replayed_damage = repaint_chrome_command_stream_region(&mut replayed, &stream)
            .expect("command stream should repaint visible viewport damage");

        assert_eq!(replayed_damage, legacy_damage);
        assert_eq!(
            first_pixel_difference(replayed.as_bytes(), legacy.as_bytes(), 200),
            None
        );
    }

    fn first_pixel_difference(
        left: &[u8],
        right: &[u8],
        width: u32,
    ) -> Option<(u32, u32, [u8; 4], [u8; 4])> {
        left.chunks_exact(4)
            .zip(right.chunks_exact(4))
            .enumerate()
            .find_map(|(index, (left, right))| {
                (left != right).then(|| {
                    let x = index as u32 % width;
                    let y = index as u32 / width;
                    (
                        x,
                        y,
                        [left[0], left[1], left[2], left[3]],
                        [right[0], right[1], right[2], right[3]],
                    )
                })
            })
    }

    fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y as usize * width as usize) + x as usize) * 4;
        [
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]
    }
}
