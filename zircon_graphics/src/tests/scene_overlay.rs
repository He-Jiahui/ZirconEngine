#[test]
fn viewport_overlay_renderer_reports_specified_pass_order() {
    assert_eq!(
        crate::ViewportOverlayRenderer::pass_order(),
        &[
            "PreviewSkyPass",
            "BaseScenePass",
            "SelectionOutlinePass",
            "WireframePass",
            "GridPass",
            "SceneGizmoPass",
            "HandlePass",
        ]
    );
}
