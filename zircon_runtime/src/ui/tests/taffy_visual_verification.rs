use std::path::PathBuf;

#[test]
fn taffy_layout_docs_keep_visual_profile_gate() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should live under repository root")
        .to_path_buf();
    let doc_path = repo_root.join("docs/zircon_runtime/ui/layout/pass.md");
    let script_path = repo_root.join("tools/ui-profile-capture.ps1");

    let doc = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", doc_path.display()));
    let script = std::fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", script_path.display()));

    for token in [
        "Visual Verification Gate",
        "tools/ui-profile-capture.ps1",
        "material_lab_startup,material_lab_hover,material_lab_click,drawer_resize",
        "-CaptureSoftbufferScreenshot",
        "screenshot_gpu.png",
        "screenshot_softbuffer.png",
        "ui_hotspots.json",
        "Layout Engine",
        "selected=Taffy",
        "selected=LegacyZircon",
        "no overlapping or clipped rows",
        "If `-CaptureSoftbufferScreenshot` is not available",
    ] {
        assert!(
            doc.contains(token),
            "layout pass docs should keep visual verification token `{token}`"
        );
    }

    for scenario in [
        "material_lab_startup",
        "material_lab_hover",
        "material_lab_click",
        "drawer_resize",
    ] {
        assert!(
            script.contains(scenario),
            "profile capture script should keep `{scenario}` for visual layout verification"
        );
    }

    for token in [
        "primaryInteractionEvidence",
        "ui_interaction_evidence.json",
        "Set-Content -Path $interactionEvidencePath",
    ] {
        assert!(
            script.contains(token),
            "profile capture script should preserve primary interaction evidence token `{token}`"
        );
    }
}
