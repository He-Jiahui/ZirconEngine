use super::super::{ChromeCommandKind, ChromeCommandStream};
use super::support::push_recorded_for_test;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::{
    HostRecordedPaintCommand, HostRecordedPaintKind,
};

#[test]
fn recorded_commands_preserve_corner_radius_in_chrome_stream() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    push_recorded_for_test(
        &mut stream,
        HostRecordedPaintCommand {
            frame: FrameRect {
                x: 4.0,
                y: 4.0,
                width: 24.0,
                height: 16.0,
            },
            clip_frame: None,
            z_index: 3,
            source: None,
            kind: HostRecordedPaintKind::Quad {
                color: [10, 20, 30, 255],
                corner_radius: 8.0,
            },
        },
        false,
    );
    push_recorded_for_test(
        &mut stream,
        HostRecordedPaintCommand {
            frame: FrameRect {
                x: 4.0,
                y: 24.0,
                width: 24.0,
                height: 16.0,
            },
            clip_frame: None,
            z_index: 4,
            source: None,
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
