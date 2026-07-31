mod markers;

use self::markers::{draw_debug_refresh_rate_marker, draw_project_marker};
use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{
    draw_border, draw_label_marker, draw_rect, draw_separator_line,
};
use super::super::paint_theme::{current_host_metrics, current_host_palette, HostMaterialPalette};
use super::root_frames::RootFrames;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RootSkeletonPalette {
    pub(super) top_bar: [u8; 4],
    pub(super) center_band: [u8; 4],
    pub(super) dock: [u8; 4],
    pub(super) document: [u8; 4],
    pub(super) viewport: [u8; 4],
    pub(super) status: [u8; 4],
    pub(super) separator: [u8; 4],
    pub(super) accent: [u8; 4],
    pub(super) text_muted: [u8; 4],
    pub(super) marker_surface: [u8; 4],
}

fn root_skeleton_palette(palette: HostMaterialPalette) -> RootSkeletonPalette {
    RootSkeletonPalette {
        top_bar: palette.popup,
        center_band: palette.surface,
        dock: palette.surface_inset,
        document: palette.surface_inset,
        viewport: palette.shell_background,
        status: palette.surface_hover,
        separator: palette.border,
        accent: palette.accent,
        text_muted: palette.text_muted,
        marker_surface: palette.surface_inset,
    }
}

pub(in crate::ui::retained_host::host_contract) fn draw_root_skeleton(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    let metrics = current_host_metrics();
    let palette = root_skeleton_palette(current_host_palette());
    draw_rect(frame, root.top_bar.clone(), palette.top_bar);
    draw_rect(frame, root.center_band.clone(), palette.center_band);
    draw_rect(frame, root.left_region.clone(), palette.dock);
    draw_rect(frame, root.right_region.clone(), palette.dock);
    draw_rect(frame, root.document_region.clone(), palette.document);
    draw_rect(frame, root.bottom_region.clone(), palette.dock);
    draw_rect(frame, root.viewport_region.clone(), palette.viewport);
    draw_rect(frame, root.status_bar.clone(), palette.status);

    draw_border(frame, root.left_region.clone(), palette.separator);
    draw_border(frame, root.right_region.clone(), palette.separator);
    draw_border(frame, root.document_region.clone(), palette.separator);
    draw_border(frame, root.bottom_region.clone(), palette.separator);
    draw_border(frame, root.viewport_region.clone(), palette.accent);
    draw_separator_line(
        frame,
        0,
        root.top_bar.height.round() as u32,
        frame.width(),
        palette.separator,
    );

    draw_project_marker(
        frame,
        &presentation.host_shell.project_path,
        &root.top_bar,
        palette,
        metrics,
    );
    draw_debug_refresh_rate_marker(
        frame,
        &root.top_bar,
        &presentation.host_shell.debug_refresh_rate,
        palette,
        metrics,
    );
    draw_label_marker(
        frame,
        &root.viewport_region,
        &presentation.host_shell.viewport_label,
        palette.accent,
    );
    draw_label_marker(
        frame,
        &root.status_bar,
        &presentation.host_shell.status_secondary,
        palette.text_muted,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn root_skeleton_palette_projects_every_surface_from_the_current_theme_roles() {
        let mut palette = PALETTE;
        palette.popup = [1, 2, 3, 255];
        palette.surface = [4, 5, 6, 255];
        palette.surface_inset = [7, 8, 9, 255];
        palette.shell_background = [10, 11, 12, 255];
        palette.surface_hover = [13, 14, 15, 255];
        palette.border = [16, 17, 18, 255];
        palette.accent = [19, 20, 21, 255];
        palette.text_muted = [22, 23, 24, 255];

        assert_eq!(
            root_skeleton_palette(palette),
            RootSkeletonPalette {
                top_bar: [1, 2, 3, 255],
                center_band: [4, 5, 6, 255],
                dock: [7, 8, 9, 255],
                document: [7, 8, 9, 255],
                viewport: [10, 11, 12, 255],
                status: [13, 14, 15, 255],
                separator: [16, 17, 18, 255],
                accent: [19, 20, 21, 255],
                text_muted: [22, 23, 24, 255],
                marker_surface: [7, 8, 9, 255],
            }
        );
    }
}
