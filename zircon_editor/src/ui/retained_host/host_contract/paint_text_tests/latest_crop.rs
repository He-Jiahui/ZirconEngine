use super::*;

#[test]
fn retained_text_editor_latest_crop_labels_keep_stable_ink_spacing() {
    let frame = editor_latest_crop_proof_framebuffer();
    let full_label = editor_latest_crop_ink_profile(
        &frame,
        FrameRect {
            x: 34.0,
            y: 22.0,
            width: 260.0,
            height: 58.0,
        },
    );
    let narrow_label = editor_latest_crop_ink_profile(
        &frame,
        FrameRect {
            x: 610.0,
            y: 22.0,
            width: 280.0,
            height: 58.0,
        },
    );
    let shifted_full_label = editor_latest_crop_label_profile("editor base.zui", 44.95, 210.0);
    let baseline_full_label = editor_latest_crop_label_profile("editor base.zui", 44.875, 210.0);
    export_editor_latest_crop_framebuffer_if_requested();

    assert!(full_label.painted_pixels > 100);
    assert!(narrow_label.painted_pixels > 120);
    assert!(
        full_label.max_internal_empty_columns <= 7,
        "latest editor crop full label should not show visible character-position holes: {full_label:?}"
    );
    assert!(
        narrow_label.max_internal_empty_columns <= 7,
        "latest editor crop ellipsis label should keep compact readable spacing: {narrow_label:?}"
    );
    assert!(
        shifted_full_label.left.abs_diff(baseline_full_label.left) <= 1,
        "latest editor crop label should not jump horizontally between nearby subpixel origins: base={baseline_full_label:?}, shifted={shifted_full_label:?}"
    );
    assert!(
        (shifted_full_label.ink_center_x - baseline_full_label.ink_center_x).abs() <= 1.0,
        "latest editor crop label center should stay stable between nearby subpixel origins: base={baseline_full_label:?}, shifted={shifted_full_label:?}"
    );
}

#[test]
fn retained_text_editor_tiny_crop_labels_keep_stable_ink_spacing() {
    let frame = editor_tiny_crop_proof_framebuffer();
    let body_label = editor_latest_crop_ink_profile(
        &frame,
        FrameRect {
            x: 34.0,
            y: 22.0,
            width: 260.0,
            height: 42.0,
        },
    );
    let caption_label = editor_latest_crop_ink_profile(
        &frame,
        FrameRect {
            x: 610.0,
            y: 22.0,
            width: 280.0,
            height: 42.0,
        },
    );
    let shifted_body_label =
        editor_latest_crop_label_profile_with_size("editor base.zui", 44.95, 210.0, 10.0, 12.0);
    let baseline_body_label =
        editor_latest_crop_label_profile_with_size("editor base.zui", 44.875, 210.0, 10.0, 12.0);
    export_editor_tiny_crop_framebuffer_if_requested();

    assert!(body_label.painted_pixels > 60);
    assert!(caption_label.painted_pixels > 40);
    assert!(
        body_label.max_internal_empty_columns <= 7,
        "tiny editor body label should not show visible character-position holes: {body_label:?}"
    );
    assert!(
        caption_label.max_internal_empty_columns <= 7,
        "tiny editor caption label should keep compact readable spacing: {caption_label:?}"
    );
    assert!(
        shifted_body_label.left.abs_diff(baseline_body_label.left) <= 1,
        "tiny editor label should not jump horizontally between nearby subpixel origins: base={baseline_body_label:?}, shifted={shifted_body_label:?}"
    );
    assert!(
        (shifted_body_label.ink_center_x - baseline_body_label.ink_center_x).abs() <= 1.0,
        "tiny editor label center should stay stable between nearby subpixel origins: base={baseline_body_label:?}, shifted={shifted_body_label:?}"
    );
}

#[test]
fn retained_text_editor_default_grayscale_crop_labels_keep_line_origin_ink_stable() {
    let fractional =
        editor_latest_crop_label_profile_with_size("editor base.zui", 44.875, 210.0, 13.0, 16.0);
    let snapped =
        editor_latest_crop_label_profile_with_size("editor base.zui", 45.0, 210.0, 13.0, 16.0);

    assert!(
        fractional.left.abs_diff(snapped.left) <= 1,
        "default grayscale editor labels should keep the first ink column stable at nearby fractional origins: fractional={fractional:?}, snapped={snapped:?}"
    );
    assert!(
        fractional.painted_pixels.abs_diff(snapped.painted_pixels) <= 8,
        "default grayscale editor labels should keep stable painted coverage at nearby fractional origins: fractional={fractional:?}, snapped={snapped:?}"
    );
    assert!(
        (fractional.ink_center_x - snapped.ink_center_x).abs() <= 0.5,
        "default grayscale editor labels should keep the ink center stable after line-origin snapping: fractional={fractional:?}, snapped={snapped:?}"
    );
    assert_eq!(
        fractional.max_internal_empty_columns, snapped.max_internal_empty_columns,
        "default grayscale editor labels should keep character gaps stable at nearby fractional origins: fractional={fractional:?}, snapped={snapped:?}"
    );
}

fn editor_latest_crop_proof_framebuffer() -> HostRgbaFrame {
    let mut frame = HostRgbaFrame::filled(985, 130, EDITOR_CROP_PROOF_BACKGROUND);

    frame.fill_rect(
        &FrameRect {
            x: 14.0,
            y: 4.0,
            width: 448.0,
            height: 126.0,
        },
        EDITOR_CROP_PROOF_TAB_SURFACE,
    );
    frame.fill_rect(
        &FrameRect {
            x: 558.0,
            y: 4.0,
            width: 427.0,
            height: 126.0,
        },
        EDITOR_CROP_PROOF_TAB_SURFACE,
    );
    frame.fill_rect(
        &FrameRect {
            x: 32.0,
            y: 20.0,
            width: 396.0,
            height: 62.0,
        },
        EDITOR_CROP_PROOF_TAB_INSET,
    );
    frame.fill_rect(
        &FrameRect {
            x: 594.0,
            y: 20.0,
            width: 352.0,
            height: 62.0,
        },
        EDITOR_CROP_PROOF_TAB_INSET,
    );
    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 44.875,
            y: 39.0,
            width: 210.0,
            height: 22.0,
        },
        "editor base.zui",
        None,
        EDITOR_CROP_PROOF_TEXT,
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
    );
    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 625.875,
            y: 39.0,
            width: 260.0,
            height: 22.0,
        },
        "folder-op...line.svg",
        None,
        EDITOR_CROP_PROOF_TEXT,
        13.0,
        16.0,
        UiTextRunPaintStyle::default(),
    );

    frame
}

fn editor_tiny_crop_proof_framebuffer() -> HostRgbaFrame {
    let mut frame = HostRgbaFrame::filled(985, 108, EDITOR_CROP_PROOF_BACKGROUND);

    frame.fill_rect(
        &FrameRect {
            x: 14.0,
            y: 4.0,
            width: 448.0,
            height: 104.0,
        },
        EDITOR_CROP_PROOF_TAB_SURFACE,
    );
    frame.fill_rect(
        &FrameRect {
            x: 558.0,
            y: 4.0,
            width: 427.0,
            height: 104.0,
        },
        EDITOR_CROP_PROOF_TAB_SURFACE,
    );
    frame.fill_rect(
        &FrameRect {
            x: 32.0,
            y: 20.0,
            width: 396.0,
            height: 46.0,
        },
        EDITOR_CROP_PROOF_TAB_INSET,
    );
    frame.fill_rect(
        &FrameRect {
            x: 594.0,
            y: 20.0,
            width: 352.0,
            height: 46.0,
        },
        EDITOR_CROP_PROOF_TAB_INSET,
    );
    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 44.875,
            y: 34.0,
            width: 210.0,
            height: 18.0,
        },
        "editor base.zui",
        None,
        EDITOR_CROP_PROOF_TEXT,
        10.0,
        12.0,
        UiTextRunPaintStyle::default(),
    );
    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 625.875,
            y: 35.0,
            width: 260.0,
            height: 16.0,
        },
        "folder-op...line.svg",
        None,
        EDITOR_CROP_PROOF_TEXT,
        8.5,
        10.2,
        UiTextRunPaintStyle::default(),
    );

    frame
}

fn editor_latest_crop_label_profile(text: &str, x: f32, width: f32) -> EditorCropInkProfile {
    editor_latest_crop_label_profile_with_size(text, x, width, 13.0, 16.0)
}

fn editor_latest_crop_label_profile_with_size(
    text: &str,
    x: f32,
    width: f32,
    font_size: f32,
    line_height: f32,
) -> EditorCropInkProfile {
    let frame = editor_latest_crop_label_frame_with_size(text, x, width, font_size, line_height);
    ink_profile_from_frame(&frame, [17, 22, 26, 255])
}

fn editor_latest_crop_label_frame_with_size(
    text: &str,
    x: f32,
    width: f32,
    font_size: f32,
    line_height: f32,
) -> HostRgbaFrame {
    const BACKGROUND: [u8; 4] = [17, 22, 26, 255];
    let mut frame = HostRgbaFrame::filled(320, 48, BACKGROUND);

    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x,
            y: 14.0,
            width,
            height: 22.0,
        },
        text,
        None,
        [224, 232, 238, 255],
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
    );

    frame
}

fn editor_latest_crop_ink_profile(
    frame: &HostRgbaFrame,
    region: FrameRect,
) -> EditorCropInkProfile {
    ink_profile_from_frame_region(frame, EDITOR_CROP_PROOF_TAB_INSET, region)
}

fn export_editor_latest_crop_framebuffer_if_requested() {
    let Ok(directory) = std::env::var("ZR_TEXT_EDITOR_CROP_PROOF_DIR") else {
        return;
    };
    let directory = Path::new(&directory);
    std::fs::create_dir_all(directory).expect("create latest editor crop proof directory");
    let frame = editor_latest_crop_proof_framebuffer();
    let stem = std::env::var("ZR_TEXT_EDITOR_CROP_PROOF_STEM")
        .unwrap_or_else(|_| "runtime_text_editor_latest_crop_framebuffer_20260705".to_string());
    let png_path = directory.join(format!("{stem}.png"));
    save_rgba_png(&png_path, frame.as_bytes(), frame.width(), frame.height());
    let full_crop_rect = FrameRect {
        x: 34.0,
        y: 22.0,
        width: 260.0,
        height: 58.0,
    };
    let narrow_crop_rect = FrameRect {
        x: 610.0,
        y: 22.0,
        width: 280.0,
        height: 58.0,
    };
    let full_crop_path = directory.join(format!("{stem}_full_label.png"));
    let narrow_crop_path = directory.join(format!("{stem}_narrow_label.png"));
    let full_crop_zoom_path = directory.join(format!(
        "{stem}_full_label_zoom{}x.png",
        EDITOR_CROP_PROOF_ZOOM_SCALE
    ));
    let narrow_crop_zoom_path = directory.join(format!(
        "{stem}_narrow_label_zoom{}x.png",
        EDITOR_CROP_PROOF_ZOOM_SCALE
    ));
    save_frame_region_png(&frame, &full_crop_rect, &full_crop_path);
    save_frame_region_png(&frame, &narrow_crop_rect, &narrow_crop_path);
    save_frame_region_png_scaled_nearest(
        &frame,
        &full_crop_rect,
        EDITOR_CROP_PROOF_ZOOM_SCALE,
        &full_crop_zoom_path,
    );
    save_frame_region_png_scaled_nearest(
        &frame,
        &narrow_crop_rect,
        EDITOR_CROP_PROOF_ZOOM_SCALE,
        &narrow_crop_zoom_path,
    );
    let full_label = editor_latest_crop_ink_profile(&frame, full_crop_rect);
    let narrow_label = editor_latest_crop_ink_profile(&frame, narrow_crop_rect);
    let log = format!(
        "{stem}\n\
         source_test=retained_text_editor_latest_crop_labels_keep_stable_ink_spacing\n\
         artifact={}\n\
         full_crop={}\n\
         narrow_crop={}\n\
         full_crop_zoom={}\n\
         narrow_crop_zoom={}\n\
         full_label={full_label:?}\n\
         narrow_label={narrow_label:?}\n\
         zoom_note=Zoom crops are nearest-neighbor expansions of the same HostRgbaFrame pixels for small-glyph visual inspection.\n\
         note=PNGs reproduce the latest editor crop labels: editor base.zui and folder-op...line.svg.\n",
        png_path.display(),
        full_crop_path.display(),
        narrow_crop_path.display(),
        full_crop_zoom_path.display(),
        narrow_crop_zoom_path.display()
    );
    std::fs::write(directory.join(format!("{stem}.log")), log)
        .expect("save latest editor crop proof log");
}

fn export_editor_tiny_crop_framebuffer_if_requested() {
    let Ok(directory) = std::env::var("ZR_TEXT_EDITOR_CROP_PROOF_DIR") else {
        return;
    };
    let directory = Path::new(&directory);
    std::fs::create_dir_all(directory).expect("create tiny editor crop proof directory");
    let frame = editor_tiny_crop_proof_framebuffer();
    let png_path = directory.join("runtime_text_editor_tiny_crop_framebuffer_20260706.png");
    save_rgba_png(&png_path, frame.as_bytes(), frame.width(), frame.height());
    let body_crop_rect = FrameRect {
        x: 34.0,
        y: 22.0,
        width: 260.0,
        height: 42.0,
    };
    let caption_crop_rect = FrameRect {
        x: 610.0,
        y: 22.0,
        width: 280.0,
        height: 42.0,
    };
    let body_crop_path = directory.join("runtime_text_editor_tiny_crop_body_label_20260706.png");
    let caption_crop_path =
        directory.join("runtime_text_editor_tiny_crop_caption_label_20260706.png");
    save_frame_region_png(&frame, &body_crop_rect, &body_crop_path);
    save_frame_region_png(&frame, &caption_crop_rect, &caption_crop_path);
    let body_label = editor_latest_crop_ink_profile(&frame, body_crop_rect);
    let caption_label = editor_latest_crop_ink_profile(&frame, caption_crop_rect);
    let log = format!(
        "runtime_text_editor_tiny_crop_framebuffer_20260706\n\
         source_test=retained_text_editor_tiny_crop_labels_keep_stable_ink_spacing\n\
         artifact={}\n\
         body_crop={}\n\
         caption_crop={}\n\
         body_label={body_label:?}\n\
         caption_label={caption_label:?}\n\
         note=PNGs reproduce tiny editor labels at 10px body and 8.5px caption sizes.\n",
        png_path.display(),
        body_crop_path.display(),
        caption_crop_path.display()
    );
    std::fs::write(
        directory.join("runtime_text_editor_tiny_crop_framebuffer_20260706.log"),
        log,
    )
    .expect("save tiny editor crop proof log");
}
