use std::path::Path;

use super::visual_screenshot::{save_window_snapshot, welcome_input_window};

const WELCOME_MVP_ACTIONS_SCREENSHOT: &str = "editor-window-m3-welcome-mvp-actions-640x520.png";

#[test]
#[ignore = "writes the Welcome MVP action-layout screenshot under docs/tests/editor"]
fn capture_welcome_mvp_actions_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");

    let welcome = welcome_input_window(640, 520);
    let output = save_window_snapshot(&welcome, WELCOME_MVP_ACTIONS_SCREENSHOT);
    let expected_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live below the repository root")
        .join("docs")
        .join("tests")
        .join("editor");

    assert_eq!(output.parent(), Some(expected_dir.as_path()));
    assert_eq!(
        output.file_name().and_then(|name| name.to_str()),
        Some(WELCOME_MVP_ACTIONS_SCREENSHOT)
    );
    let captured = image::open(&output)
        .unwrap_or_else(|error| panic!("Welcome MVP screenshot should decode: {error}"));
    assert_eq!((captured.width(), captured.height()), (640, 520));
}
