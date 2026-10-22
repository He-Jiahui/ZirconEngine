use super::*;

#[test]
fn splash_paints_all_stages_at_compact_normal_and_high_dpi_sizes() {
    let mut fonts = usvg::fontdb::Database::new();
    fonts.load_font_data(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../zircon_runtime/assets/fonts/FiraSans-Regular.ttf"
        ))
        .to_vec(),
    );
    let fonts = Arc::new(fonts);
    for (width, height, scale) in [(400, 360, 1.0), (640, 440, 1.0), (1280, 880, 2.0)] {
        let mut previous = None;
        for stage in EditorStartupStage::ALL {
            let pixels = render(
                width,
                height,
                scale,
                stage,
                true,
                Duration::from_secs(12),
                fonts.clone(),
            )
            .unwrap();
            assert_eq!(pixels.len(), width as usize * height as usize * 4);
            assert!(pixels.chunks_exact(4).all(|rgba| rgba[3] == 255));
            if let Some(previous) = previous {
                assert_ne!(pixels, previous);
            }
            previous = Some(pixels);
        }
    }
}
