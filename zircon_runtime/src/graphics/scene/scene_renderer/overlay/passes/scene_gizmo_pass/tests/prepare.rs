const SOURCE: &str = include_str!("../prepare.rs");

#[test]
fn scene_gizmo_appends_pending_icon_uploads_once_after_icon_discovery() {
    let production = SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("scene gizmo prepare source should retain a test-module boundary");
    let discovery = production
        .find("for gizmo in &frame.overlays().scene_gizmos")
        .expect("scene gizmo icon discovery");
    let append = production
        .find(".append_pending_uploads(frame_texture_uploads)")
        .expect("pending icon upload append");

    assert!(discovery < append);
    assert_eq!(
        production
            .matches(".append_pending_uploads(frame_texture_uploads)")
            .count(),
        1
    );
    assert!(!production.contains("wgpu::Queue"));
}
