use std::rc::Rc;

use super::super::{
    chrome_command_from_recorded_for_test, ChromeCommandKind, ChromeCommandStream,
    ChromeImagePayload,
};
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostDocumentDockSurfaceData, HostWindowLayoutData, HostWindowPresentationData,
    PaneData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintCommand;
use crate::ui::retained_host::primitives::{
    Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel,
};

pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_IMAGE_CONTROL_ID: &str =
    "TestRootOverlayImage";
pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_IMAGE_PATH: &str =
    "ui/test/root-overlay.png";
pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_IMAGE_WIDTH: u32 = 2;
pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_IMAGE_HEIGHT: u32 = 2;
pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_UPLOAD_BYTES: u64 =
    (ROOT_OVERLAY_IMAGE_WIDTH as u64) * (ROOT_OVERLAY_IMAGE_HEIGHT as u64) * 4;
pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_FRAME_WIDTH: f32 = 48.0;
pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_FRAME_HEIGHT: f32 = 32.0;
pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_FRAME_SIZE: (u32, u32) =
    (96, 72);
pub(in crate::ui::retained_host::host_contract) const ROOT_OVERLAY_COLOR: [u8; 4] =
    [28, 199, 215, 255];
pub(in crate::ui::retained_host::host_contract) const LEGACY_CENTER_BAND: [u8; 4] =
    [23, 27, 34, 255];
pub(in crate::ui::retained_host::host_contract) const LEGACY_DOCUMENT_PANEL: [u8; 4] =
    [13, 16, 22, 255];
pub(in crate::ui::retained_host::host_contract) const LEGACY_VIEWPORT_PANEL: [u8; 4] =
    [7, 10, 15, 255];

pub(in crate::ui::retained_host::host_contract) fn presentation_with_viewport_image(
) -> HostWindowPresentationData {
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

pub(in crate::ui::retained_host::host_contract) fn presentation_with_root_overlay_image(
) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.root_template_nodes = model_rc(vec![root_overlay_image_node()]);
    presentation
}

pub(in crate::ui::retained_host::host_contract) fn presentation_with_componentized_workbench_frame_owner(
) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_layout = test_layout();
    presentation.host_scene_data.layout = test_layout();
    presentation.workbench_window_nodes = model_rc(vec![
        componentized_workbench_frame_only_node(
            "WorkbenchWindowMainBandRegion",
            0.0,
            36.0,
            200.0,
            144.0,
        ),
        componentized_workbench_frame_only_node(
            "WorkbenchMainBandViewportPanel",
            24.0,
            50.0,
            150.0,
            120.0,
        ),
        componentized_workbench_frame_only_node("WorkbenchViewportSurface", 40.0, 92.0, 80.0, 60.0),
    ]);
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

fn componentized_workbench_frame_only_node(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: "frame_only".into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn root_overlay_image_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: "test_root_overlay_image".into(),
        control_id: ROOT_OVERLAY_IMAGE_CONTROL_ID.into(),
        role: "Image".into(),
        component_role: "image".into(),
        media_source: ROOT_OVERLAY_IMAGE_PATH.into(),
        has_preview_image: true,
        preview_image: solid_image(ROOT_OVERLAY_COLOR),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: ROOT_OVERLAY_FRAME_WIDTH,
            height: ROOT_OVERLAY_FRAME_HEIGHT,
        },
        ..TemplatePaneNodeData::default()
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

fn solid_image(color: [u8; 4]) -> Image {
    Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &solid_rgba(color),
        ROOT_OVERLAY_IMAGE_WIDTH,
        ROOT_OVERLAY_IMAGE_HEIGHT,
    ))
}

pub(in crate::ui::retained_host::host_contract) fn solid_rgba(color: [u8; 4]) -> Vec<u8> {
    [color, color, color, color].concat()
}

pub(in crate::ui::retained_host::host_contract) fn stream_has_quad_color(
    stream: &ChromeCommandStream,
    color: [u8; 4],
) -> bool {
    stream.commands().iter().any(|command| {
        matches!(
            &command.kind,
            ChromeCommandKind::Quad {
                color: command_color,
                ..
            } if *command_color == color
        )
    })
}

pub(in crate::ui::retained_host::host_contract) fn root_overlay_image_command<'a>(
    stream: &'a ChromeCommandStream,
    overlay_rgba: &[u8],
) -> Option<&'a ChromeImagePayload> {
    stream
        .commands()
        .iter()
        .find_map(|command| match &command.kind {
            ChromeCommandKind::Image { payload }
                if payload.width == ROOT_OVERLAY_IMAGE_WIDTH
                    && payload.height == ROOT_OVERLAY_IMAGE_HEIGHT
                    && payload.upload_bytes == ROOT_OVERLAY_UPLOAD_BYTES
                    && payload.rgba.as_deref() == Some(overlay_rgba) =>
            {
                Some(payload)
            }
            _ => None,
        })
}

pub(in crate::ui::retained_host::host_contract) fn first_pixel_difference(
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

pub(in crate::ui::retained_host::host_contract) fn push_recorded_for_test(
    stream: &mut ChromeCommandStream,
    command: HostRecordedPaintCommand,
    full_rebuild: bool,
) {
    if let Some(command) = chrome_command_from_recorded_for_test(command, full_rebuild, true) {
        stream.commands.push(command);
    }
}

pub(in crate::ui::retained_host::host_contract) fn pixel(
    bytes: &[u8],
    width: u32,
    x: u32,
    y: u32,
) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
