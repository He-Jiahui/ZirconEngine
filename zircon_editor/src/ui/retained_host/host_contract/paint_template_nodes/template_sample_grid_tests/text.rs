use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::geometry::SampleGridGeometry;
use super::super::palette::sample_grid_palette_from_host;
use super::super::text::push_sample_grid_text;
use super::support::sample_grid_node;

#[test]
fn axis_titles_and_x_ticks_use_the_reference_top_gutter_without_overlap() {
    let node = sample_grid_node(360.0, 260.0);
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 260.0,
    };
    let geometry = SampleGridGeometry::from_frame(&frame);
    let mut commands = Vec::new();

    push_sample_grid_text(
        &mut commands,
        &node,
        &geometry,
        &frame,
        0,
        1.0,
        sample_grid_palette_from_host(PALETTE),
    );

    let x_title = text_command(&commands, "Direction (deg)");
    let y_title = text_command(&commands, "Speed (cm/s)");
    let first_x_tick = text_command(&commands, "-180");
    let top_y_tick = y_tick_command(&commands, "600", geometry.plot.x);
    let bottom_y_tick = y_tick_command(&commands, "0", geometry.plot.x);

    assert!(
        (x_title.frame.x + x_title.frame.width * 0.5
            - (geometry.plot.x + geometry.plot.width * 0.5))
            .abs()
            <= 0.5
    );
    assert!(x_title.frame.bottom() < first_x_tick.frame.y);
    assert!(first_x_tick.frame.bottom() < geometry.plot.y);
    assert!(y_title.frame.x >= geometry.outer.x);
    assert!(y_title.frame.right() + 4.0 <= x_title.frame.x);
    assert!((top_y_tick.frame.right() - bottom_y_tick.frame.right()).abs() <= 0.01);
    assert!((top_y_tick.frame.right() - (geometry.plot.x - 7.0)).abs() <= 0.01);
}

#[test]
fn moving_x_axis_text_to_the_top_releases_the_bottom_gutter() {
    for frame in [
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 220.0,
            height: 160.0,
        },
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 520.0,
            height: 320.0,
        },
    ] {
        let geometry = SampleGridGeometry::from_frame(&frame);
        let top_gutter = geometry.plot.y - geometry.outer.y;
        let bottom_gutter = geometry.outer.bottom() - geometry.plot.bottom();

        assert!(top_gutter >= 40.0);
        assert!(bottom_gutter <= 18.0);
        assert!(top_gutter > bottom_gutter);
    }
}

#[test]
fn x_ticks_are_omitted_as_a_group_when_the_plot_cannot_keep_them_readable() {
    let node = sample_grid_node(45.0, 80.0);
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 45.0,
        height: 80.0,
    };
    let geometry = SampleGridGeometry::from_frame(&frame);
    let mut commands = Vec::new();

    push_sample_grid_text(
        &mut commands,
        &node,
        &geometry,
        &frame,
        0,
        1.0,
        sample_grid_palette_from_host(PALETTE),
    );

    assert!(geometry.plot.width > 0.0);
    assert!(commands.iter().all(|command| {
        command.z_index != 4
            || command.frame.right() < geometry.plot.x
            || command.frame.y >= geometry.plot.y
    }));
}

fn text_command<'a>(commands: &'a [HostPaintCommand], text: &str) -> &'a HostPaintCommand {
    commands
        .iter()
        .find(|command| command.text.as_deref() == Some(text))
        .unwrap_or_else(|| panic!("missing Sample Grid text command `{text}`"))
}

fn y_tick_command<'a>(
    commands: &'a [HostPaintCommand],
    text: &str,
    plot_x: f32,
) -> &'a HostPaintCommand {
    commands
        .iter()
        .find(|command| command.text.as_deref() == Some(text) && command.frame.right() < plot_x)
        .unwrap_or_else(|| panic!("missing Sample Grid Y tick command `{text}`"))
}
