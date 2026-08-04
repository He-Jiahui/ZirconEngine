use super::ShellSizePx;

const DEFAULT_SCALE_FACTOR: f32 = 1.0;
const DEFAULT_REFERENCE_WIDTH: f32 = 1920.0;
const DEFAULT_REFERENCE_HEIGHT: f32 = 1080.0;

/// Declares how one rendering root converts layout coordinates to physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ResolutionScaleMode {
    #[default]
    ConstantPhysical,
    ConstantPixel,
    ScaleWithResolution {
        reference_size: ShellSizePx,
    },
}

/// Root-owned conversion boundary between physical window metrics and logical layout units.
/// Every exposed extent stays finite and non-negative before it reaches shell geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolutionContext {
    effective_scale_factor: f32,
    scale_mode: ResolutionScaleMode,
    physical_size: ShellSizePx,
    logical_size: ShellSizePx,
}

impl ResolutionContext {
    /// Builds the editor's default constant-physical root context.
    pub fn from_physical_size(physical_size: ShellSizePx, scale_factor: f32) -> Self {
        Self::from_physical_size_with_scale_mode(
            physical_size,
            scale_factor,
            ResolutionScaleMode::ConstantPhysical,
        )
    }

    /// Builds a root context with one explicitly declared scale policy.
    ///
    /// Resolution-relative scaling uses the DPI-independent window extent, so a
    /// high-DPI display does not count its pixel density twice.
    pub fn from_physical_size_with_scale_mode(
        physical_size: ShellSizePx,
        system_scale_factor: f32,
        scale_mode: ResolutionScaleMode,
    ) -> Self {
        let system_scale_factor = normalized_scale_factor(system_scale_factor);
        let physical_size = ShellSizePx::new(
            normalized_extent(physical_size.width),
            normalized_extent(physical_size.height),
        );
        let effective_scale_factor =
            scale_mode.effective_scale_factor(physical_size, system_scale_factor);
        let logical_size = ShellSizePx::new(
            normalized_extent(physical_size.width / effective_scale_factor),
            normalized_extent(physical_size.height / effective_scale_factor),
        );
        Self {
            effective_scale_factor,
            scale_mode,
            physical_size,
            logical_size,
        }
    }

    pub fn scale_factor(self) -> f32 {
        self.effective_scale_factor
    }

    pub fn effective_scale_factor(self) -> f32 {
        self.effective_scale_factor
    }

    pub fn scale_mode(self) -> ResolutionScaleMode {
        self.scale_mode
    }

    pub fn physical_size(self) -> ShellSizePx {
        self.physical_size
    }

    pub fn logical_size(self) -> ShellSizePx {
        self.logical_size
    }

    pub fn logical_width(self) -> f32 {
        self.logical_size.width
    }

    pub fn to_physical(self, logical_extent: f32) -> f32 {
        normalized_extent(normalized_extent(logical_extent) * self.effective_scale_factor)
    }

    pub fn to_logical(self, physical_extent: f32) -> f32 {
        normalized_extent(normalized_extent(physical_extent) / self.effective_scale_factor)
    }

    pub(crate) fn logical_extent(physical_extent: f32, scale_factor: f32) -> f32 {
        normalized_extent(
            normalized_extent(physical_extent) / normalized_scale_factor(scale_factor),
        )
    }
}

impl ResolutionScaleMode {
    fn effective_scale_factor(self, physical_size: ShellSizePx, system_scale_factor: f32) -> f32 {
        match self {
            Self::ConstantPhysical => system_scale_factor,
            Self::ConstantPixel => DEFAULT_SCALE_FACTOR,
            Self::ScaleWithResolution { reference_size } => normalized_scale_factor(
                system_scale_factor
                    * resolution_relative_scale(physical_size, system_scale_factor, reference_size),
            ),
        }
    }
}

fn resolution_relative_scale(
    physical_size: ShellSizePx,
    system_scale_factor: f32,
    reference_size: ShellSizePx,
) -> f32 {
    let dpi_independent_width = normalized_extent(physical_size.width / system_scale_factor);
    let dpi_independent_height = normalized_extent(physical_size.height / system_scale_factor);
    let reference_width =
        normalized_reference_extent(reference_size.width, DEFAULT_REFERENCE_WIDTH);
    let reference_height =
        normalized_reference_extent(reference_size.height, DEFAULT_REFERENCE_HEIGHT);
    let width_scale = dpi_independent_width / reference_width;
    let height_scale = dpi_independent_height / reference_height;
    let scale = width_scale.min(height_scale);

    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        DEFAULT_SCALE_FACTOR
    }
}

fn normalized_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        DEFAULT_SCALE_FACTOR
    }
}

fn normalized_extent(extent: f32) -> f32 {
    if extent.is_finite() {
        extent.max(0.0)
    } else {
        0.0
    }
}

fn normalized_reference_extent(extent: f32, fallback: f32) -> f32 {
    if extent.is_finite() && extent > 0.0 {
        extent
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
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
}
