use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneNodeData,
};

use super::support::{hit_test_workbench_window_template_node, model};

#[test]
fn componentized_workbench_hit_test_skips_template_controls_outside_painted_regions() {
    let presentation = HostWindowPresentationData {
        host_layout: crate::ui::retained_host::host_contract::data::HostWindowLayoutData {
            authoritative: true,
            center_band_frame: FrameRect {
                x: 0.0,
                y: 44.0,
                width: 900.0,
                height: 520.0,
            },
            status_bar_frame: FrameRect {
                x: 0.0,
                y: 584.0,
                width: 900.0,
                height: 36.0,
            },
            ..Default::default()
        },
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "top-button".into(),
                control_id: "WorkbenchTopButton".into(),
                action_id: "top.click".into(),
                frame: TemplateNodeFrameData {
                    x: 16.0,
                    y: 8.0,
                    width: 120.0,
                    height: 28.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "scene-button".into(),
                control_id: "WorkbenchSceneButton".into(),
                action_id: "scene.click".into(),
                frame: TemplateNodeFrameData {
                    x: 16.0,
                    y: 120.0,
                    width: 120.0,
                    height: 28.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "status-button".into(),
                control_id: "WorkbenchStatusButton".into(),
                action_id: "status.click".into(),
                frame: TemplateNodeFrameData {
                    x: 16.0,
                    y: 592.0,
                    width: 120.0,
                    height: 20.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "extension-host".into(),
                control_id: "WorkbenchExtensionModuleWorkspacesHost".into(),
                frame: TemplateNodeFrameData {
                    x: 300.0,
                    y: 96.0,
                    width: 280.0,
                    height: 280.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "extension-workspace-host".into(),
                parent_node_id: "extension-host".into(),
                control_id: "WorkbenchExtensionBlendSpaceWorkspaceHost".into(),
                frame: TemplateNodeFrameData {
                    x: 300.0,
                    y: 96.0,
                    width: 280.0,
                    height: 280.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "extension-workspace".into(),
                parent_node_id: "extension-workspace-host".into(),
                control_id: "WorkbenchExtensionBlendSpaceWorkspace".into(),
                frame: TemplateNodeFrameData {
                    x: 300.0,
                    y: 96.0,
                    width: 280.0,
                    height: 280.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "extension-button".into(),
                parent_node_id: "extension-workspace".into(),
                control_id: "WorkbenchExtensionBlendSpaceApply".into(),
                action_id: "blend.apply".into(),
                frame: TemplateNodeFrameData {
                    x: 320.0,
                    y: 120.0,
                    width: 120.0,
                    height: 28.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "inactive-workspace-host".into(),
                parent_node_id: "extension-host".into(),
                control_id: "WorkbenchExtensionShaderEditorWorkspaceHost".into(),
                frame: TemplateNodeFrameData {
                    x: 300.0,
                    y: 96.0,
                    width: 280.0,
                    height: 280.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "inactive-workspace".into(),
                parent_node_id: "inactive-workspace-host".into(),
                control_id: "WorkbenchExtensionShaderEditorWorkspace".into(),
                frame: TemplateNodeFrameData {
                    x: 300.0,
                    y: 96.0,
                    width: 280.0,
                    height: 280.0,
                },
                ..Default::default()
            },
            TemplatePaneNodeData {
                node_id: "inactive-button".into(),
                parent_node_id: "inactive-workspace".into(),
                control_id: "WorkbenchExtensionShaderEditorApply".into(),
                action_id: "shader.apply".into(),
                z_index: 99,
                frame: TemplateNodeFrameData {
                    x: 320.0,
                    y: 120.0,
                    width: 120.0,
                    height: 28.0,
                },
                ..Default::default()
            },
        ]),
        ..Default::default()
    };
    let top_hit = hit_test_workbench_window_template_node(&presentation, 24.0, 20.0)
        .expect("top chrome template control should remain hit-testable");
    assert_eq!(top_hit.control_id.as_str(), "WorkbenchTopButton");
    let status_hit = hit_test_workbench_window_template_node(&presentation, 24.0, 600.0)
        .expect("status template control should remain hit-testable");
    assert_eq!(status_hit.control_id.as_str(), "WorkbenchStatusButton");
    let extension_hit = hit_test_workbench_window_template_node(&presentation, 328.0, 132.0)
        .expect("active extension workspace template control should remain hit-testable");
    assert_eq!(
        extension_hit.control_id.as_str(),
        "WorkbenchExtensionBlendSpaceApply"
    );
    assert!(
        hit_test_workbench_window_template_node(&presentation, 24.0, 132.0).is_none(),
        "ordinary pane template controls must not capture points where the componentized painter draws host scene data"
    );
}
