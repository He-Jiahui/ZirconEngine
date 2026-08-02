use super::*;
use crate::core::framework::render::{
    EnvironmentExtract, GridOverlayExtract, PreviewEnvironmentExtract, RenderFrameExtract,
    RenderOverlayExtract, RenderParticleGpuFrameExtract, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use crate::core::math::{UVec2, Vec4};
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;
use crate::text::{
    ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine, layout_text,
    register_resolved_text_glyph_artifact,
};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiRenderExtract, UiRenderList, UiResolvedStyle, UiResolvedTextLayout,
    UiResolvedTextLine, UiResolvedTextRun, UiRichTextFormat, UiTextAlign, UiTextCaret,
    UiTextCaretAffinity, UiTextComposition, UiTextDirection, UiTextOverflow, UiTextRange,
    UiTextRenderMode, UiTextRunKind, UiTextSelection, UiTextWrap, UiTextWritingMode,
};

mod background;
mod distance_field_effects;
mod parity;
mod rich_inline;
mod rich_table;
mod text_style_decorations;

#[test]
fn screen_space_ui_plan_keeps_text_batches_for_quad_commands() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(1),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#112233".to_string()),
                        foreground_color: Some("#ddeeff".to_string()),
                        language: Some("zh-Hans-CN".to_string()),
                        font_weight: 650,
                        font_size: 18.0,
                        line_height: 22.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Launch".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(200, 100),
    );

    assert_eq!(plan.draws.len(), 1);
    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].language.as_deref(), Some("zh-hans-cn"));
    assert_eq!(plan.native_texts[0].font_weight, 650);
    assert_eq!(
        plan.native_texts[0].background_color,
        Some([
            0x11 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x33 as f32 / 255.0,
            1.0,
        ])
    );
    assert!(plan.sdf_texts.is_empty());
}

#[test]
fn screen_space_ui_plan_preserves_auto_text_route_identity_and_generation() {
    let command = UiRenderCommand {
        node_id: UiNodeId::new(17),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            font_size: 23.0,
            line_height: 28.0,
            text_render_mode: UiTextRenderMode::Auto,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some("Route identity".to_string()),
        image: None,
        opacity: 1.0,
    };
    let first = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.auto-route"),
            list: UiRenderList {
                commands: vec![command.clone()],
            },
        },
        UVec2::new(200, 100),
    );
    let second = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.auto-route"),
            list: UiRenderList {
                commands: vec![command.clone()],
            },
        },
        UVec2::new(200, 100),
    );

    assert_eq!(first.auto_texts.len(), 1);
    assert_eq!(
        first.auto_texts[0].route_identity,
        ScreenSpaceUiTextRouteIdentity::new("runtime.ui.auto-route", UiNodeId::new(17), None,)
    );
    assert_ne!(first.auto_texts[0].command_generation, 0);
    assert_eq!(
        first.auto_texts[0].command_generation,
        second.auto_texts[0].command_generation
    );

    let mut changed = command;
    changed.style.font_size = 24.0;
    let changed = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.auto-route"),
            list: UiRenderList {
                commands: vec![changed],
            },
        },
        UVec2::new(200, 100),
    );
    assert_ne!(
        first.auto_texts[0].command_generation,
        changed.auto_texts[0].command_generation
    );
}

#[test]
fn screen_space_ui_plan_infers_text_background_from_prior_opaque_quad() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![
                    UiRenderCommand {
                        node_id: UiNodeId::new(12),
                        kind: UiRenderCommandKind::Quad,
                        frame: UiFrame::new(0.0, 0.0, 180.0, 60.0),
                        clip_frame: None,
                        z_index: 0,
                        style: UiResolvedStyle {
                            background_color: Some("#203040".to_string()),
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: None,
                        image: None,
                        opacity: 1.0,
                    },
                    UiRenderCommand {
                        node_id: UiNodeId::new(13),
                        kind: UiRenderCommandKind::Text,
                        frame: UiFrame::new(16.0, 12.0, 80.0, 20.0),
                        clip_frame: None,
                        z_index: 1,
                        style: UiResolvedStyle {
                            foreground_color: Some("#ddeeff".to_string()),
                            text_render_mode: UiTextRenderMode::Native,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: Some("Panel label".to_string()),
                        image: None,
                        opacity: 1.0,
                    },
                ],
            },
        },
        UVec2::new(200, 100),
    );

    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(
        plan.native_texts[0].background_color,
        Some([
            0x20 as f32 / 255.0,
            0x30 as f32 / 255.0,
            0x40 as f32 / 255.0,
            1.0,
        ])
    );
}

#[test]
fn screen_space_ui_plan_keeps_inherited_background_unknown_after_transparent_overlay() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![
                    UiRenderCommand {
                        node_id: UiNodeId::new(14),
                        kind: UiRenderCommandKind::Quad,
                        frame: UiFrame::new(0.0, 0.0, 180.0, 60.0),
                        clip_frame: None,
                        z_index: 0,
                        style: UiResolvedStyle {
                            background_color: Some("#203040".to_string()),
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: None,
                        image: None,
                        opacity: 1.0,
                    },
                    UiRenderCommand {
                        node_id: UiNodeId::new(15),
                        kind: UiRenderCommandKind::Quad,
                        frame: UiFrame::new(0.0, 0.0, 180.0, 60.0),
                        clip_frame: None,
                        z_index: 1,
                        style: UiResolvedStyle {
                            background_color: Some("#ffffff80".to_string()),
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: None,
                        image: None,
                        opacity: 1.0,
                    },
                    UiRenderCommand {
                        node_id: UiNodeId::new(16),
                        kind: UiRenderCommandKind::Text,
                        frame: UiFrame::new(16.0, 12.0, 80.0, 20.0),
                        clip_frame: None,
                        z_index: 2,
                        style: UiResolvedStyle {
                            foreground_color: Some("#ddeeff".to_string()),
                            text_render_mode: UiTextRenderMode::Native,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: Some("Overlay label".to_string()),
                        image: None,
                        opacity: 1.0,
                    },
                ],
            },
        },
        UVec2::new(200, 100),
    );

    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].background_color, None);
}

#[test]
fn screen_space_ui_plan_keeps_transparent_text_background_unknown_with_prior_quad() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![
                    UiRenderCommand {
                        node_id: UiNodeId::new(17),
                        kind: UiRenderCommandKind::Quad,
                        frame: UiFrame::new(0.0, 0.0, 180.0, 60.0),
                        clip_frame: None,
                        z_index: 0,
                        style: UiResolvedStyle {
                            background_color: Some("#203040".to_string()),
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: None,
                        image: None,
                        opacity: 1.0,
                    },
                    UiRenderCommand {
                        node_id: UiNodeId::new(18),
                        kind: UiRenderCommandKind::Text,
                        frame: UiFrame::new(16.0, 12.0, 80.0, 20.0),
                        clip_frame: None,
                        z_index: 1,
                        style: UiResolvedStyle {
                            background_color: Some("#11223380".to_string()),
                            foreground_color: Some("#ddeeff".to_string()),
                            text_render_mode: UiTextRenderMode::Native,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: Some("Transparent label".to_string()),
                        image: None,
                        opacity: 1.0,
                    },
                ],
            },
        },
        UVec2::new(200, 100),
    );

    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].background_color, None);
}

#[test]
fn screen_space_ui_plan_infers_text_background_from_framebuffer_background() {
    let plan = plan_screen_space_ui_batches_with_framebuffer_background(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(19),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(16.0, 12.0, 80.0, 20.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ddeeff".to_string()),
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Clear label".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(200, 100),
        Some([0.02, 0.03, 0.04, 1.0]),
    );

    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(
        plan.native_texts[0].background_color,
        Some([0.02, 0.03, 0.04, 1.0])
    );
}

#[test]
fn screen_space_ui_plan_blocks_framebuffer_background_after_transparent_overlay() {
    let plan = plan_screen_space_ui_batches_with_framebuffer_background(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![
                    UiRenderCommand {
                        node_id: UiNodeId::new(20),
                        kind: UiRenderCommandKind::Quad,
                        frame: UiFrame::new(0.0, 0.0, 180.0, 60.0),
                        clip_frame: None,
                        z_index: 0,
                        style: UiResolvedStyle {
                            background_color: Some("#ffffff80".to_string()),
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: None,
                        image: None,
                        opacity: 1.0,
                    },
                    UiRenderCommand {
                        node_id: UiNodeId::new(21),
                        kind: UiRenderCommandKind::Text,
                        frame: UiFrame::new(16.0, 12.0, 80.0, 20.0),
                        clip_frame: None,
                        z_index: 1,
                        style: UiResolvedStyle {
                            foreground_color: Some("#ddeeff".to_string()),
                            text_render_mode: UiTextRenderMode::Native,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: Some("Overlay label".to_string()),
                        image: None,
                        opacity: 1.0,
                    },
                ],
            },
        },
        UVec2::new(200, 100),
        Some([0.02, 0.03, 0.04, 1.0]),
    );

    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].background_color, None);
}

#[test]
fn screen_space_ui_plan_routes_sdf_text_to_a_separate_batch() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(2),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(0.0, 0.0, 160.0, 48.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 20.0,
                        line_height: 24.0,
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Sdf,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("SDF".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(320, 180),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
}

#[test]
fn screen_space_ui_plan_keeps_auto_text_in_a_separate_batch() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(3),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(4.0, 6.0, 144.0, 40.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font: Some("res://fonts/default.font.toml".to_string()),
                        font_size: 16.0,
                        line_height: 20.0,
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Auto".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(320, 180),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert_eq!(plan.auto_texts.len(), 1);
}

#[test]
fn screen_space_ui_plan_requires_glyph_artifact_for_plain_resolved_layout() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(4),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 90.0, 48.0),
                    clip_frame: Some(UiFrame::new(0.0, 0.0, 120.0, 48.0)),
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        font_weight: 500,
                        line_height: 12.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::Word,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::Word,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 50.0,
                        measured_height: 24.0,
                        source_range: UiTextRange { start: 0, end: 16 },
                        lines: vec![
                            UiResolvedTextLine {
                                text: "Alpha Beta".to_string(),
                                frame: UiFrame::new(20.0, 20.0, 50.0, 12.0),
                                source_range: UiTextRange { start: 0, end: 10 },
                                visual_range: UiTextRange { start: 0, end: 10 },
                                measured_width: 50.0,
                                glyph_advances: vec![],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: vec![UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Alpha Beta".to_string(),
                                    source_range: UiTextRange { start: 0, end: 10 },
                                    visual_range: UiTextRange { start: 0, end: 10 },
                                    direction: UiTextDirection::LeftToRight,
                                }],
                                ellipsized: false,
                            },
                            UiResolvedTextLine {
                                text: "Gamma".to_string(),
                                frame: UiFrame::new(35.0, 32.0, 25.0, 12.0),
                                source_range: UiTextRange { start: 11, end: 16 },
                                visual_range: UiTextRange { start: 0, end: 5 },
                                measured_width: 25.0,
                                glyph_advances: vec![],
                                baseline: 8.0,
                                direction: UiTextDirection::LeftToRight,
                                runs: vec![UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Gamma".to_string(),
                                    source_range: UiTextRange { start: 11, end: 16 },
                                    visual_range: UiTextRange { start: 0, end: 5 },
                                    direction: UiTextDirection::LeftToRight,
                                }],
                                ellipsized: false,
                            },
                        ],
                        boxes: Vec::new(),
                        overflow_clipped: false,
                        editable: None,
                        rich_text_artifact: None,
                    }),
                    text: Some("Alpha Beta Gamma".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(160, 120),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
}

#[test]
fn screen_space_ui_plan_does_not_shape_visual_bidi_runs_without_an_artifact() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(19),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 70.0, 12.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        font_weight: 500,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::None,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 70.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 10 },
                        lines: vec![UiResolvedTextLine {
                            text: "abc גבא".to_string(),
                            frame: UiFrame::new(10.0, 20.0, 70.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 10 },
                            visual_range: UiTextRange { start: 0, end: 10 },
                            measured_width: 70.0,
                            glyph_advances: vec![10.0; 7],
                            baseline: 9.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: vec![
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "abc ".to_string(),
                                    source_range: UiTextRange { start: 0, end: 4 },
                                    visual_range: UiTextRange { start: 0, end: 4 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "ג".to_string(),
                                    source_range: UiTextRange { start: 8, end: 10 },
                                    visual_range: UiTextRange { start: 4, end: 6 },
                                    direction: UiTextDirection::RightToLeft,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "ב".to_string(),
                                    source_range: UiTextRange { start: 6, end: 8 },
                                    visual_range: UiTextRange { start: 6, end: 8 },
                                    direction: UiTextDirection::RightToLeft,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "א".to_string(),
                                    source_range: UiTextRange { start: 4, end: 6 },
                                    visual_range: UiTextRange { start: 8, end: 10 },
                                    direction: UiTextDirection::RightToLeft,
                                },
                            ],
                            ellipsized: false,
                        }],
                        boxes: Vec::new(),
                        overflow_clipped: false,
                        editable: None,
                        rich_text_artifact: None,
                    }),
                    text: Some("abc אבג".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(160, 120),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
}

#[test]
fn screen_space_ui_plan_preserves_plain_glyph_artifact_through_sdf_routing() {
    let glyph = |glyph_id, source_range| TextGlyph {
        glyph_id,
        source_range,
        visual_range: 0..0,
        advance: 10.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 1,
        flags: TextGlyphFlags {
            right_to_left: true,
            ..TextGlyphFlags::default()
        },
        requires_rasterization: true,
    };
    let artifact = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("سلام"),
        source_text_origin: 0,
        font_generation: 0,
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![
                glyph(104, 6..8),
                glyph(103, 4..6),
                glyph(102, 2..4),
                glyph(101, 0..2),
            ],
            layout_line: UiResolvedTextLine {
                text: "مالس".to_string(),
                frame: UiFrame::new(10.0, 20.0, 40.0, 12.0),
                source_range: UiTextRange { start: 0, end: 8 },
                visual_range: UiTextRange { start: 0, end: 8 },
                measured_width: 40.0,
                glyph_advances: vec![10.0; 4],
                baseline: 9.0,
                direction: UiTextDirection::RightToLeft,
                runs: Vec::new(),
                ellipsized: false,
            },
        })],
    });
    let mut plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.glyph-artifact"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(23),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 40.0, 12.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::None,
                        direction: UiTextDirection::RightToLeft,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 40.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 8 },
                        lines: vec![UiResolvedTextLine {
                            text: "مالس".to_string(),
                            frame: UiFrame::new(10.0, 20.0, 40.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 8 },
                            visual_range: UiTextRange { start: 0, end: 8 },
                            measured_width: 40.0,
                            glyph_advances: vec![10.0; 4],
                            baseline: 9.0,
                            direction: UiTextDirection::RightToLeft,
                            runs: Vec::new(),
                            ellipsized: false,
                        }],
                        boxes: Vec::new(),
                        overflow_clipped: false,
                        editable: None,
                        rich_text_artifact: Some(register_resolved_text_glyph_artifact(
                            Arc::clone(&artifact),
                        )),
                    }),
                    text: Some("سلام".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(120, 80),
    );

    assert!(plan.native_texts.is_empty());
    assert_eq!(plan.sdf_texts.len(), 1);
    assert!(plan.sdf_texts[0].preserve_shaped_glyphs);
    assert!(plan.sdf_texts[0].shaped_glyphs.is_empty());
    let artifact_line = plan.sdf_texts[0]
        .glyph_artifact_line
        .as_ref()
        .expect("text-owned glyph artifact line");
    assert!(Arc::ptr_eq(&artifact_line.artifact, &artifact));
    assert_eq!(
        artifact_line
            .glyphs()
            .expect("glyph artifact source line")
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![104, 103, 102, 101]
    );
    assert_eq!(
        artifact_line
            .glyphs()
            .expect("glyph artifact source line")
            .iter()
            .map(|glyph| glyph.advance)
            .collect::<Vec<_>>(),
        vec![10.0; 4]
    );

    let batch = &mut plan.sdf_texts[0];
    batch
        .glyph_artifact_line
        .as_mut()
        .expect("glyph artifact line")
        .font_generation = u64::MAX;
    super::text_advances::refresh_screen_space_text_batch_glyphs(batch);

    let refreshed_line = batch
        .glyph_artifact_line
        .as_ref()
        .expect("glyph artifact line")
        .refreshed_line
        .as_ref()
        .expect("font generation rebuild must retain a text-owned line");
    assert!(batch.shaped_glyphs.is_empty());
    assert!(
        refreshed_line
            .glyphs
            .windows(2)
            .all(|pair| pair[0].source_range.start >= pair[1].source_range.start)
    );
    assert!(
        refreshed_line
            .glyphs
            .iter()
            .all(|glyph| glyph.source_range.start < glyph.source_range.end
                && glyph.source_range.end <= 8)
    );
    assert_eq!(
        refreshed_line
            .glyphs
            .iter()
            .map(|glyph| glyph.advance)
            .sum::<f32>(),
        40.0
    );
}

#[test]
fn screen_space_ui_plan_splits_rich_text_runs_from_shared_paint() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(6),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 150.0, 18.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        font_weight: 500,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        rich_text_format: UiRichTextFormat::Markdown,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        text_align: UiTextAlign::Left,
                        wrap: UiTextWrap::None,
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 150.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 15 },
                        lines: vec![UiResolvedTextLine {
                            text: "Alpha Beta Code".to_string(),
                            frame: UiFrame::new(10.0, 20.0, 150.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 15 },
                            visual_range: UiTextRange { start: 0, end: 15 },
                            measured_width: 150.0,
                            glyph_advances: vec![],
                            baseline: 8.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: vec![
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Plain,
                                    text: "Alpha ".to_string(),
                                    source_range: UiTextRange { start: 0, end: 6 },
                                    visual_range: UiTextRange { start: 0, end: 6 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Strong,
                                    text: "Beta".to_string(),
                                    source_range: UiTextRange { start: 6, end: 10 },
                                    visual_range: UiTextRange { start: 6, end: 10 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                                UiResolvedTextRun {
                                    kind: UiTextRunKind::Code,
                                    text: " Code".to_string(),
                                    source_range: UiTextRange { start: 10, end: 15 },
                                    visual_range: UiTextRange { start: 10, end: 15 },
                                    direction: UiTextDirection::LeftToRight,
                                },
                            ],
                            ellipsized: false,
                        }],
                        boxes: Vec::new(),
                        overflow_clipped: false,
                        editable: None,
                        rich_text_artifact: None,
                    }),
                    text: Some("Alpha Beta Code".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(220, 80),
    );

    assert_eq!(plan.native_texts.len(), 3);
    assert_eq!(plan.native_texts[0].text, "Alpha ");
    assert_eq!(plan.native_texts[1].text, "Beta");
    assert_eq!(plan.native_texts[2].text, " Code");
    assert_eq!(
        plan.native_texts[0].frame,
        UiFrame::new(10.0, 20.0, 60.0, 12.0)
    );
    assert_eq!(
        plan.native_texts[1].frame,
        UiFrame::new(70.0, 20.0, 40.0, 12.0)
    );
    assert!(plan.native_texts[1].style.strong);
    assert!(plan.native_texts[2].style.code);
    assert!(plan.native_texts.iter().all(|text| text.font_weight == 500));
}

#[test]
fn screen_space_ui_plan_uses_shared_text_decorations_as_pre_and_post_text_draws() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(5),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(10.0, 20.0, 50.0, 12.0),
                    clip_frame: Some(UiFrame::new(0.0, 0.0, 80.0, 48.0)),
                    z_index: 0,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        font_size: 10.0,
                        line_height: 12.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: Some(UiResolvedTextLayout {
                        direction: UiTextDirection::LeftToRight,
                        writing_mode: UiTextWritingMode::HorizontalTb,
                        overflow: UiTextOverflow::Clip,
                        font_size: 10.0,
                        line_height: 12.0,
                        measured_width: 50.0,
                        measured_height: 12.0,
                        source_range: UiTextRange { start: 0, end: 5 },
                        editable: Some(UiEditableTextState {
                            text: "Hello".to_string(),
                            caret: UiTextCaret {
                                offset: 4,
                                affinity: UiTextCaretAffinity::Downstream,
                            },
                            selection: Some(UiTextSelection {
                                anchor: 1,
                                focus: 3,
                            }),
                            composition: Some(UiTextComposition {
                                range: UiTextRange { start: 2, end: 4 },
                                text: "ll".to_string(),
                                restore_text: None,
                            }),
                            read_only: false,
                        }),
                        lines: vec![UiResolvedTextLine {
                            text: "Hello".to_string(),
                            frame: UiFrame::new(10.0, 20.0, 50.0, 12.0),
                            source_range: UiTextRange { start: 0, end: 5 },
                            visual_range: UiTextRange { start: 0, end: 5 },
                            measured_width: 50.0,
                            glyph_advances: vec![],
                            baseline: 8.0,
                            direction: UiTextDirection::LeftToRight,
                            runs: vec![UiResolvedTextRun {
                                kind: UiTextRunKind::Plain,
                                text: "Hello".to_string(),
                                source_range: UiTextRange { start: 0, end: 5 },
                                visual_range: UiTextRange { start: 0, end: 5 },
                                direction: UiTextDirection::LeftToRight,
                            }],
                            ellipsized: false,
                        }],
                        ..UiResolvedTextLayout::default()
                    }),
                    text: Some("Hello".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(80, 48),
    );

    assert_eq!(plan.draws.len(), 1);
    assert_eq!(plan.draws[0].vertices, 0..6);
    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.post_text_draws.len(), 1);
    assert_eq!(plan.post_text_draws[0].vertices, 6..18);
}

#[test]
fn ui_plan_projects_paint_elements_once_per_command() {
    let source = include_str!("../render.rs");
    let text_paint_source = include_str!("text_paint.rs");

    assert!(source.contains("let paint_elements = command.to_paint_elements(0);"));
    assert!(!source.contains("for element in command.to_paint_elements(0)"));
    assert!(source.contains("paint_elements: &[UiPaintElement]"));
    assert!(!text_paint_source.contains("to_paint_elements"));
    assert!(text_paint_source.contains("paint_elements: &[UiPaintElement]"));
}

#[test]
fn empty_ui_records_only_non_noop_attachment_ops() {
    let source = include_str!("record.rs");

    assert!(source.contains("fn record_empty_screen_space_ui_pass("));
    assert!(source.contains("if attachment_ops == RenderGraphAttachmentOps::load_store()"));
    assert!(source.contains("zircon-screen-space-ui-empty-pass"));
}
