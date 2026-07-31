use std::rc::Rc;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostViewportImageData, HostWindowPresentationData, TemplateNodeFrameData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::{
    draw_componentized_extension_workspace, draw_componentized_workbench_window,
    ComponentizedChromeFallbackTransform, ExtensionWorkspaceSubtree,
    EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID,
};

const SENTINEL: [u8; 4] = [241, 17, 193, 255];

#[test]
fn componentized_workbench_keeps_host_menu_chrome_above_its_mount() {
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![template_mount(
            "workbench/root",
            "",
            "WorkbenchWindowRoot",
            FrameRect {
                x: 0.0,
                y: 57.0,
                width: 160.0,
                height: 63.0,
            },
            "panel",
        )]),
        ..HostWindowPresentationData::default()
    };
    presentation.host_layout.center_band_frame = FrameRect {
        x: 0.0,
        y: 96.0,
        width: 160.0,
        height: 20.0,
    };
    presentation.host_layout.status_bar_frame = FrameRect {
        x: 0.0,
        y: 116.0,
        width: 160.0,
        height: 4.0,
    };
    presentation.host_scene_data.menu_chrome.template_nodes = model(vec![template_mount(
        "host/menu",
        "",
        "WorkbenchMenuTopBar",
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 24.0,
        },
        "accent",
    )]);
    presentation.host_scene_data.page_chrome.template_nodes = model(vec![template_mount(
        "host/pages",
        "",
        "WorkbenchPageBar",
        FrameRect {
            x: 0.0,
            y: 24.0,
            width: 160.0,
            height: 32.0,
        },
        "inset",
    )]);
    let mut frame = HostRgbaFrame::filled(160, 120, SENTINEL);

    draw_componentized_workbench_window(&mut frame, &presentation);

    assert_ne!(pixel(&frame, 4, 4), SENTINEL);
    assert_ne!(pixel(&frame, 4, 30), SENTINEL);
}

#[test]
fn live_viewport_fallback_filter_keeps_chrome_and_dynamic_overlays() {
    let presentation = HostWindowPresentationData {
        viewport_image: Some(HostViewportImageData {
            resource_key: "viewport:test".into(),
            width: 1,
            height: 1,
            rgba: vec![255; 4],
        }),
        ..HostWindowPresentationData::default()
    };
    let filter = ComponentizedChromeFallbackTransform::from_presentation(&presentation);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };

    assert!(filter
        .transform(template_node("WorkbenchViewportBackdrop"), clip.clone())
        .is_none());
    for control_id in [
        "WorkbenchViewportToolbar",
        "WorkbenchViewportSelectionTop",
        "WorkbenchViewportAxisX",
        "WorkbenchViewportGizmoCenter",
        "WorkbenchUnrelatedControl",
    ] {
        assert!(filter
            .transform(template_node(control_id), clip.clone())
            .is_some());
    }
}

#[test]
fn missing_or_invalid_viewport_image_keeps_the_fallback_scene() {
    for presentation in [
        HostWindowPresentationData::default(),
        HostWindowPresentationData {
            viewport_image: Some(HostViewportImageData::default()),
            ..HostWindowPresentationData::default()
        },
    ] {
        let filter = ComponentizedChromeFallbackTransform::from_presentation(&presentation);
        assert!(filter
            .transform(
                template_node("WorkbenchViewportBackdrop"),
                FrameRect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
            )
            .is_some());
    }
}

fn template_node(control_id: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn active_extension_workspace_paints_only_inside_its_adaptive_host_frame() {
    let extension_nodes = vec![
        template_mount(
            "root/extension_workspaces",
            "root",
            EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID,
            FrameRect {
                x: 12.0,
                y: 18.0,
                width: 52.0,
                height: 38.0,
            },
            "panel",
        ),
        template_mount(
            "root/extension_workspaces/blend_host",
            "root/extension_workspaces",
            "WorkbenchExtensionBlendSpaceWorkspaceHost",
            FrameRect {
                x: 16.0,
                y: 22.0,
                width: 44.0,
                height: 30.0,
            },
            "panel",
        ),
        template_mount(
            "component-instance-41",
            "root/extension_workspaces/blend_host",
            "WorkbenchExtensionBlendSpaceWorkspace",
            FrameRect {
                x: 16.0,
                y: 22.0,
                width: 44.0,
                height: 30.0,
            },
            "inset",
        ),
        template_mount(
            "generated-node-92",
            "component-instance-41",
            "BlendSpaceSampleCanvas",
            FrameRect {
                x: 20.0,
                y: 26.0,
                width: 22.0,
                height: 14.0,
            },
            "accent",
        ),
    ];
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(extension_nodes.clone()),
        ..HostWindowPresentationData::default()
    };
    let subtree =
        ExtensionWorkspaceSubtree::from_presentation(&presentation, "root/extension_workspaces");
    assert!(subtree
        .included_node_ids
        .contains("root/extension_workspaces"));
    assert!(subtree
        .included_node_ids
        .contains("root/extension_workspaces/blend_host"));
    assert!(subtree.included_node_ids.contains("component-instance-41"));
    assert!(subtree.included_node_ids.contains("generated-node-92"));
    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 72.0,
    };
    let mut frame = HostRgbaFrame::filled(80, 72, SENTINEL);

    assert!(draw_componentized_extension_workspace(
        &mut frame,
        &presentation,
        &frame_bounds,
    ));

    assert_ne!(pixel(&frame, 20, 28), SENTINEL);
    assert_eq!(pixel(&frame, 4, 28), SENTINEL);
    assert_eq!(pixel(&frame, 72, 28), SENTINEL);

    let baseline = frame.into_bytes();
    let presentation_with_overlapping_scene_sibling = HostWindowPresentationData {
        workbench_window_nodes: model(
            extension_nodes
                .into_iter()
                .chain([template_mount(
                    "root/scene_workspace",
                    "root",
                    "WorkbenchSceneWorkspace",
                    FrameRect {
                        x: 12.0,
                        y: 18.0,
                        width: 52.0,
                        height: 38.0,
                    },
                    "accent",
                )])
                .collect(),
        ),
        ..HostWindowPresentationData::default()
    };
    let filtered_subtree = ExtensionWorkspaceSubtree::from_presentation(
        &presentation_with_overlapping_scene_sibling,
        "root/extension_workspaces",
    );
    assert!(!filtered_subtree
        .included_node_ids
        .contains("root/scene_workspace"));
    let mut filtered = HostRgbaFrame::filled(80, 72, SENTINEL);
    assert!(draw_componentized_extension_workspace(
        &mut filtered,
        &presentation_with_overlapping_scene_sibling,
        &frame_bounds,
    ));
    assert_eq!(
        filtered.into_bytes(),
        baseline,
        "legacy SceneWorkspace siblings must not overpaint the activated extension subtree"
    );
}

#[test]
fn scene_workspace_without_extension_host_keeps_legacy_dock_pixels_untouched() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![template_mount(
            "root/scene_workspace",
            "root",
            "WorkbenchSceneWorkspace",
            FrameRect {
                x: 12.0,
                y: 18.0,
                width: 52.0,
                height: 38.0,
            },
            "panel",
        )]),
        ..HostWindowPresentationData::default()
    };
    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 72.0,
    };
    let mut frame = HostRgbaFrame::filled(80, 72, SENTINEL);

    assert!(!draw_componentized_extension_workspace(
        &mut frame,
        &presentation,
        &frame_bounds,
    ));
    assert!(frame
        .as_bytes()
        .chunks_exact(4)
        .all(|pixel| pixel == SENTINEL));
}

#[test]
fn inactive_extension_workspace_hosts_do_not_overpaint_the_visible_workspace() {
    let shared_frame = FrameRect {
        x: 12.0,
        y: 18.0,
        width: 148.0,
        height: 42.0,
    };
    let active_nodes = vec![
        template_mount(
            "root/extension_workspaces",
            "root",
            EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID,
            shared_frame.clone(),
            "panel",
        ),
        template_mount(
            "root/extension_workspaces/blend_host",
            "root/extension_workspaces",
            "WorkbenchExtensionBlendSpaceWorkspaceHost",
            shared_frame.clone(),
            "panel",
        ),
        template_mount(
            "root/extension_workspaces/blend_host/blend",
            "root/extension_workspaces/blend_host",
            "WorkbenchExtensionBlendSpaceWorkspace",
            shared_frame.clone(),
            "panel",
        ),
        template_mount(
            "root/extension_workspaces/blend_host/blend/canvas",
            "root/extension_workspaces/blend_host/blend",
            "WorkbenchExtensionBlendSpaceSampleCanvas",
            FrameRect {
                x: 22.0,
                y: 24.0,
                width: 96.0,
                height: 28.0,
            },
            "accent",
        ),
    ];
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(active_nodes.clone()),
        ..HostWindowPresentationData::default()
    };
    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 172.0,
        height: 72.0,
    };
    let mut baseline = HostRgbaFrame::filled(172, 72, SENTINEL);
    assert!(draw_componentized_extension_workspace(
        &mut baseline,
        &presentation,
        &frame_bounds,
    ));

    let presentation_with_inactive_host = HostWindowPresentationData {
        workbench_window_nodes: model(
            active_nodes
                .into_iter()
                .chain([template_mount(
                    "root/extension_workspaces/inactive_host",
                    "root/extension_workspaces",
                    "WorkbenchExtensionShaderEditorWorkspaceHost",
                    shared_frame,
                    "panel",
                )])
                .collect(),
        ),
        ..HostWindowPresentationData::default()
    };
    let mut with_inactive_host = HostRgbaFrame::filled(172, 72, SENTINEL);
    assert!(draw_componentized_extension_workspace(
        &mut with_inactive_host,
        &presentation_with_inactive_host,
        &frame_bounds,
    ));

    assert_eq!(
        with_inactive_host.into_bytes(),
        baseline.into_bytes(),
        "visible workspace selection must exclude empty sibling Overlay hosts"
    );
}

#[test]
fn extension_workspace_search_field_paints_surface_glyph_and_placeholder_text() {
    let search_frame = FrameRect {
        x: 18.0,
        y: 24.0,
        width: 136.0,
        height: 30.0,
    };
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            template_mount(
                "root/extension_workspaces",
                "root",
                EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID,
                FrameRect {
                    x: 12.0,
                    y: 18.0,
                    width: 148.0,
                    height: 42.0,
                },
                "panel",
            ),
            template_mount(
                "root/extension_workspaces/blend_host",
                "root/extension_workspaces",
                "WorkbenchExtensionBlendSpaceWorkspaceHost",
                FrameRect {
                    x: 12.0,
                    y: 18.0,
                    width: 148.0,
                    height: 42.0,
                },
                "panel",
            ),
            template_mount(
                "root/extension_workspaces/blend_host/blend",
                "root/extension_workspaces/blend_host",
                "WorkbenchExtensionBlendSpaceWorkspace",
                FrameRect {
                    x: 12.0,
                    y: 18.0,
                    width: 148.0,
                    height: 42.0,
                },
                "panel",
            ),
            TemplatePaneNodeData {
                node_id: "root/extension_workspaces/search".into(),
                parent_node_id: "root/extension_workspaces/blend_host/blend".into(),
                control_id: "WorkbenchExtensionBlendSpaceSearch".into(),
                role: "SearchField".into(),
                component_role: "search-field".into(),
                component_category: "input".into(),
                component_layout_role: "leaf".into(),
                text: "Search samples".into(),
                surface_variant: "inset".into(),
                border_width: 1.0,
                corner_radius: 5.0,
                frame: TemplateNodeFrameData {
                    x: search_frame.x,
                    y: search_frame.y,
                    width: search_frame.width,
                    height: search_frame.height,
                },
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };
    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 172.0,
        height: 72.0,
    };
    let mut frame = HostRgbaFrame::filled(172, 72, SENTINEL);

    assert!(draw_componentized_extension_workspace(
        &mut frame,
        &presentation,
        &frame_bounds,
    ));

    assert!(
        distinct_colors(&frame, &search_frame) >= 4,
        "SearchField should paint its inset surface, border, search glyph, and Runtime text placeholder"
    );
}

fn template_mount(
    node_id: &str,
    parent_node_id: &str,
    control_id: &str,
    frame: FrameRect,
    surface_variant: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id.into(),
        parent_node_id: parent_node_id.into(),
        control_id: control_id.into(),
        role: "Mount".into(),
        surface_variant: surface_variant.into(),
        frame: TemplateNodeFrameData {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn pixel(frame: &HostRgbaFrame, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * frame.width() as usize + x) * 4;
    frame.as_bytes()[offset..offset + 4]
        .try_into()
        .expect("pixel should expose four RGBA channels")
}

fn distinct_colors(frame: &HostRgbaFrame, rect: &FrameRect) -> usize {
    let left = rect.x.max(0.0).floor() as usize;
    let top = rect.y.max(0.0).floor() as usize;
    let right = (rect.x + rect.width).max(0.0).ceil() as usize;
    let bottom = (rect.y + rect.height).max(0.0).ceil() as usize;
    (top..bottom)
        .flat_map(|y| (left..right).map(move |x| pixel(frame, x, y)))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

fn model<T: Clone>(values: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(values)).into()
}
