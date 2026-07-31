use super::ShellSizePx;

const DEFAULT_SCALE_FACTOR: f32 = 1.0;

/// Root-owned conversion boundary between physical window metrics and logical layout units.
/// Every exposed extent stays finite and non-negative before it reaches shell geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolutionContext {
    scale_factor: f32,
    physical_size: ShellSizePx,
    logical_size: ShellSizePx,
}

impl ResolutionContext {
    pub fn from_physical_size(physical_size: ShellSizePx, scale_factor: f32) -> Self {
        let scale_factor = normalized_scale_factor(scale_factor);
        let physical_size = ShellSizePx::new(
            normalized_extent(physical_size.width),
            normalized_extent(physical_size.height),
        );
        let logical_size = ShellSizePx::new(
            normalized_extent(physical_size.width / scale_factor),
            normalized_extent(physical_size.height / scale_factor),
        );
        Self {
            scale_factor,
            physical_size,
            logical_size,
        }
    }

    pub fn scale_factor(self) -> f32 {
        self.scale_factor
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
        normalized_extent(normalized_extent(logical_extent) * self.scale_factor)
    }

    pub fn to_logical(self, physical_extent: f32) -> f32 {
        normalized_extent(normalized_extent(physical_extent) / self.scale_factor)
    }

    pub(crate) fn logical_extent(physical_extent: f32, scale_factor: f32) -> f32 {
        normalized_extent(
            normalized_extent(physical_extent) / normalized_scale_factor(scale_factor),
        )
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

#[cfg(test)]
mod tests {
    use super::{ResolutionContext, ShellSizePx};

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
