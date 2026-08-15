use super::*;
use crate::core::framework::render::{
    EnvironmentExtract, GridOverlayExtract, PreviewEnvironmentExtract, RenderFrameExtract,
    RenderOverlayExtract, RenderParticleGpuFrameExtract, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;
use crate::ui::surface::layout_text;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiRenderExtract, UiRenderList, UiResolvedStyle, UiResolvedTextLayout,
    UiResolvedTextLine, UiResolvedTextRun, UiRichTextFormat, UiTextAlign, UiTextCaret,
    UiTextCaretAffinity, UiTextComposition, UiTextDirection, UiTextOverflow, UiTextRange,
    UiTextRenderMode, UiTextRunKind, UiTextSelection, UiTextWrap, UiTextWritingMode,
};

mod background;
mod clipping;
mod distance_field_effects;
mod fallback_provenance;
mod glyph_artifacts;
mod parity;
mod rich_inline;
mod rich_table;
mod text_style_decorations;

#[test]
fn screen_space_ui_vertex_buffer_writes_only_for_new_payloads_or_reallocation() {
    let payload = [7; 32];
    assert!(!record::screen_space_ui_vertex_buffer_write_required(
        false,
        Some(payload),
        payload
    ));
    assert!(record::screen_space_ui_vertex_buffer_write_required(
        false,
        Some(payload),
        [8; 32]
    ));
    assert!(record::screen_space_ui_vertex_buffer_write_required(
        true,
        Some(payload),
        payload
    ));
    assert!(record::screen_space_ui_vertex_buffer_write_required(
        false, None, payload
    ));
}

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
            raster_scale: 1.0,
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
fn screen_space_ui_plan_carries_extract_raster_scale_to_native_text() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.raster-scale"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(1),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        font_size: 18.0,
                        line_height: 22.0,
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Crisp at 2x".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 2.0,
        },
        UVec2::new(200, 100),
    );

    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].raster_scale, 2.0);
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
            raster_scale: 1.0,
        },
        UVec2::new(200, 100),
    );
    let second = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.auto-route"),
            list: UiRenderList {
                commands: vec![command.clone()],
            },
            raster_scale: 1.0,
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
            raster_scale: 1.0,
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
            raster_scale: 1.0,
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
            raster_scale: 1.0,
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
            raster_scale: 1.0,
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
            raster_scale: 1.0,
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
            raster_scale: 1.0,
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
            raster_scale: 1.0,
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
            raster_scale: 1.0,
        },
        UVec2::new(320, 180),
    );

    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert_eq!(plan.auto_texts.len(), 1);
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
            raster_scale: 1.0,
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
                                preedit_clauses: Vec::new(),
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
            raster_scale: 1.0,
        },
        UVec2::new(80, 48),
    );

    assert_eq!(plan.draws.len(), 1);
    assert_eq!(plan.draws[0].vertices, 0..12);
    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.post_text_draws.len(), 1);
    assert_eq!(plan.post_text_draws[0].vertices, 12..24);
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
