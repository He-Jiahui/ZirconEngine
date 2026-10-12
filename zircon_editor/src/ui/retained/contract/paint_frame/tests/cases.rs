use std::sync::Arc;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiRenderFrameCommandRef, UiSurfaceFrame},
};

use super::super::data::FrameRect;
use super::{HostRecordedPaintKind, HostRgbaFrame};

#[test]
fn fill_rect_replaces_contiguous_row_span() {
    let mut frame = HostRgbaFrame::filled(4, 2, [0, 0, 0, 255]);
    frame.fill_rect(
        &FrameRect {
            x: 1.0,
            y: 0.0,
            width: 2.0,
            height: 1.0,
        },
        [10, 20, 30, 255],
    );

    assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[8..12], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[12..16], &[0, 0, 0, 255]);
}

#[test]
fn recording_only_collects_quad_without_allocating_pixels() {
    let mut frame = HostRgbaFrame::recording_only(16, 12);

    frame.fill_rect(
        &FrameRect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        },
        [10, 20, 30, 255],
    );

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].z_index, 0);
    assert!(matches!(
        commands[0].kind,
        HostRecordedPaintKind::Quad {
            color: [10, 20, 30, 255],
            corner_radius: 0.0,
        }
    ));
}

#[test]
fn recording_source_scope_publishes_one_frame_and_a_compact_command_reference() {
    let source_frame = Arc::new(UiSurfaceFrame::default());
    let command_ref = UiRenderFrameCommandRef::new(UiNodeId::new(17), 3);
    let mut frame = HostRgbaFrame::recording_only(16, 12);

    frame.with_render_source_frame(Some(&source_frame), |frame| {
        frame.with_render_source_command(Some(command_ref), |frame| {
            frame.fill_rect(
                &FrameRect {
                    x: 1.0,
                    y: 2.0,
                    width: 3.0,
                    height: 4.0,
                },
                [10, 20, 30, 255],
            );
        });
    });

    let recorded = frame.into_recorded_frame();
    let source = recorded.commands[0].source.expect("recorded source");
    assert_eq!(source.command_ref, command_ref);
    assert_eq!(source.fragment_index, 0);
    assert!(recorded
        .render_sources
        .resolve(source.surface_key)
        .is_some_and(|resolved| Arc::ptr_eq(resolved, &source_frame)));
}

#[test]
fn recording_source_fragments_are_unique_across_reentered_command_scopes() {
    let source_frame = Arc::new(UiSurfaceFrame::default());
    let command_ref = UiRenderFrameCommandRef::new(UiNodeId::new(19), 2);
    let mut frame = HostRgbaFrame::recording_only(16, 12);

    frame.with_render_source_frame(Some(&source_frame), |frame| {
        for x in [1.0, 5.0] {
            frame.with_render_source_command(Some(command_ref), |frame| {
                frame.fill_rect(
                    &FrameRect {
                        x,
                        y: 2.0,
                        width: 3.0,
                        height: 4.0,
                    },
                    [10, 20, 30, 255],
                );
            });
        }
    });

    let recorded = frame.into_recorded_frame();
    assert_eq!(recorded.commands.len(), 2);
    assert_eq!(
        recorded
            .commands
            .iter()
            .map(|command| command.source.expect("recorded source").fragment_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
}

#[test]
fn fill_rect_respects_active_paint_clip() {
    let mut frame = HostRgbaFrame::filled(4, 2, [0, 0, 0, 255]);
    frame.replace_paint_clip(Some(FrameRect {
        x: 1.0,
        y: 0.0,
        width: 2.0,
        height: 1.0,
    }));

    frame.fill_rect(
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 4.0,
            height: 2.0,
        },
        [10, 20, 30, 255],
    );

    assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[8..12], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[12..16], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[16..20], &[0, 0, 0, 255]);
}
