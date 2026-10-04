use super::ShellSizePx;

const DEFAULT_SCALE_FACTOR: f32 = 1.0;
const DEFAULT_REFERENCE_WIDTH: f32 = 1920.0;
const DEFAULT_REFERENCE_HEIGHT: f32 = 1080.0;

/// Declares how one rendering root converts layout coordinates to physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// 宿主根声明的坐标策略；同根布局、命中和最终几何须共用，不能各自叠加DPI。
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
/// 本轮窗口度量的归一快照；拖动偏好进入logical与最终frame返回physical沿同一有效比例。
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

    /// 最终发布或host输入所需的物理长度；仅用于logical到physical的边界转换。
    pub fn to_physical(self, logical_extent: f32) -> f32 {
        normalized_extent(normalized_extent(logical_extent) * self.effective_scale_factor)
    }

    /// host物理长度进入布局时的边界转换；持久化logical偏好不能重复转换。
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

/// 相对参考分辨率按DPI独立的窗口大小求比例，避免高DPI密度被计算两次。
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

/// 将无效或溢出度量隔离在根边界，避免非有限值污染布局执行器。
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
#[path = "tests/resolution_context.rs"]
mod tests;
