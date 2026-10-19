use std::time::Duration;

use super::NeutralMvpRenderer;

#[test]
fn neutral_mvp_renderer_captures_the_completed_offscreen_triangle() {
    let Ok(renderer) = NeutralMvpRenderer::new_offscreen(64, 64) else {
        return;
    };
    let pixels = renderer.capture_rgba8(62, Duration::from_secs(5)).unwrap();
    let center = ((32 * 64 + 32) * 4) as usize;
    let center = &pixels[center..center + 4];
    // UNORM quantization permits these adjacent red/blue codes; green/alpha stay exact.
    assert!(
        matches!(center, [25 | 26, 204, 76 | 77, 255]),
        "unexpected MVP RGBA8 UNORM center: {center:?}"
    );
    renderer.destroy().unwrap();
}
