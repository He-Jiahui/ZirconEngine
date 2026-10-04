use super::{ResolutionContext, ResolutionScaleMode, ShellSizePx};

#[test]
fn default_context_keeps_constant_physical_dpi_behavior() {
    let default_context =
        ResolutionContext::from_physical_size(ShellSizePx::new(3840.0, 2160.0), 2.0);
    let explicit_context = ResolutionContext::from_physical_size_with_scale_mode(
        ShellSizePx::new(3840.0, 2160.0),
        2.0,
        ResolutionScaleMode::ConstantPhysical,
    );

    assert_eq!(default_context, explicit_context);
    assert_eq!(default_context.scale_factor(), 2.0);
    assert_eq!(
        default_context.logical_size(),
        ShellSizePx::new(1920.0, 1080.0)
    );
}

#[test]
fn constant_pixel_mode_keeps_layout_coordinates_in_physical_pixels() {
    let context = ResolutionContext::from_physical_size_with_scale_mode(
        ShellSizePx::new(3840.0, 2160.0),
        2.0,
        ResolutionScaleMode::ConstantPixel,
    );

    assert_eq!(context.scale_factor(), 1.0);
    assert_eq!(context.logical_size(), ShellSizePx::new(3840.0, 2160.0));
    assert_eq!(context.to_physical(24.0), 24.0);
}

#[test]
fn scale_with_resolution_uses_dpi_independent_size_before_reference_ratio() {
    let reference_size = ShellSizePx::new(1920.0, 1080.0);
    let standard = ResolutionContext::from_physical_size_with_scale_mode(
        ShellSizePx::new(3840.0, 2160.0),
        1.0,
        ResolutionScaleMode::ScaleWithResolution { reference_size },
    );
    let high_dpi = ResolutionContext::from_physical_size_with_scale_mode(
        ShellSizePx::new(7680.0, 4320.0),
        2.0,
        ResolutionScaleMode::ScaleWithResolution { reference_size },
    );

    assert_eq!(standard.scale_factor(), 2.0);
    assert_eq!(high_dpi.scale_factor(), 4.0);
    assert_eq!(standard.logical_size(), reference_size);
    assert_eq!(high_dpi.logical_size(), reference_size);
    assert_eq!(standard.to_physical(24.0), 48.0);
    assert_eq!(high_dpi.to_physical(24.0), 96.0);
}

#[test]
fn scale_with_resolution_normalizes_invalid_reference_extents() {
    let context = ResolutionContext::from_physical_size_with_scale_mode(
        ShellSizePx::new(3840.0, 2160.0),
        2.0,
        ResolutionScaleMode::ScaleWithResolution {
            reference_size: ShellSizePx::new(f32::NAN, 0.0),
        },
    );

    assert_eq!(context.effective_scale_factor(), 2.0);
    assert_eq!(context.logical_size(), ShellSizePx::new(1920.0, 1080.0));
}

#[test]
fn equivalent_physical_windows_share_one_logical_resolution() {
    let standard = ResolutionContext::from_physical_size(ShellSizePx::new(1920.0, 1080.0), 1.0);
    let high_dpi = ResolutionContext::from_physical_size(ShellSizePx::new(3840.0, 2160.0), 2.0);

    assert_eq!(standard.logical_size(), high_dpi.logical_size());
    assert_eq!(standard.logical_width(), 1920.0);
    assert_eq!(high_dpi.to_physical(24.0), 48.0);
}

#[test]
fn invalid_window_metrics_fall_back_without_poisoning_layout() {
    let context =
        ResolutionContext::from_physical_size(ShellSizePx::new(f32::NAN, f32::INFINITY), 0.0);

    assert_eq!(context.scale_factor(), 1.0);
    assert_eq!(context.physical_size(), ShellSizePx::new(0.0, 0.0));
    assert_eq!(context.logical_size(), ShellSizePx::new(0.0, 0.0));
    assert_eq!(context.to_logical(80.0), 80.0);
}

#[test]
fn construction_normalizes_scaled_extent_overflow() {
    let context = ResolutionContext::from_physical_size(
        ShellSizePx::new(f32::MAX, f32::MAX),
        f32::MIN_POSITIVE,
    );

    assert_eq!(
        context.physical_size(),
        ShellSizePx::new(f32::MAX, f32::MAX)
    );
    assert_eq!(context.logical_size(), ShellSizePx::new(0.0, 0.0));
}

#[test]
fn conversions_keep_invalid_extents_out_of_layout() {
    let context = ResolutionContext::from_physical_size(ShellSizePx::new(1920.0, 1080.0), 2.0);

    assert_eq!(context.to_physical(-12.0), 0.0);
    assert_eq!(context.to_physical(f32::NAN), 0.0);
    assert_eq!(context.to_physical(f32::MAX), 0.0);
    assert_eq!(context.to_logical(-48.0), 0.0);
    assert_eq!(context.to_logical(f32::INFINITY), 0.0);
    assert_eq!(
        ResolutionContext::logical_extent(f32::MAX, f32::MIN_POSITIVE),
        0.0
    );
}
