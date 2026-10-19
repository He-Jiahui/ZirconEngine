//! 会话视口的尺寸、窗口度量与原生表面绑定描述。
//! 构造器按值封装字段；消费方分别处理输入度量、尺寸预算、平台种类和 ABI 版本。
//! 原生句柄仅描述宿主持有的窗口，复制描述值不延长窗口或会话的实际寿命。

use crate::handles::ZrRuntimeViewportHandle;

use super::super::constants::{
    ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1, ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 通用的成对尺寸描述，供视口事件、窗口度量和帧请求使用。
///
/// # 约束
/// [`Self::new`] 不拒绝零值或超大尺寸；逻辑／物理单位由所在请求或字段决定。
/// 当前表面绑定消费方将尺寸钳制为至少 1；抓帧和呈现入口另行检查帧尺寸预算。
///
/// # 参考
/// [`ZrRuntimeViewportMetricsV1`]、[`crate::runtime_api::ZrRuntimeFrameRequestV1`]，
/// `render_surface_descriptor` 与 `validate_frame_dimensions`。
pub struct ZrRuntimeViewportSizeV1 {
    /// 横向尺寸，单位随所在的度量字段或请求契约确定。
    pub width: u32,
    /// 纵向尺寸；构造时与宽度一起原样保存。
    pub height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
/// 同时传递窗口逻辑尺寸、设备缩放比和物理尺寸，供输入坐标与渲染像素对齐。
///
/// # 使用时机与生命周期
/// 按值附于 [`crate::runtime_api::ZrRuntimeEventV1::viewport_metrics`]；事件同时把物理尺寸
/// 复制到 `size`，供 Runtime 的 resize 路径消费。窗口 UI 适配器使用三项度量形成布局事实。
///
/// # 约束
/// [`Self::new`] 不验证缩放比的有限性、正值或两种尺寸之间的关系。
/// 窗口适配器的 `viewport_metrics` 和 `sanitized_scale_factor` 决定自身的转换与回退策略。
///
/// # 参考
/// [`crate::runtime_api::ZrRuntimeEventV1::viewport_metrics`]、
/// [`crate::runtime_api::ZrRuntimeTranslatedEventV1::viewport_metrics`] 与窗口事件适配器。
pub struct ZrRuntimeViewportMetricsV1 {
    /// 窗口逻辑尺寸，供窗口适配器形成布局尺寸；不等同于物理像素数。
    pub logical_size: ZrRuntimeViewportSizeV1,
    /// 宿主提供的设备缩放比；构造器不做检查，窗口适配器独立处理无效值。
    pub device_scale_factor: f32,
    /// 物理像素尺寸；metrics 事件还会把它复制到事件 `size` 字段。
    pub physical_size: ZrRuntimeViewportSizeV1,
}

impl ZrRuntimeViewportMetricsV1 {
    /// 保存宿主在同一次窗口观测中取得的三项度量。
    ///
    /// # 参数
    /// `logical_size` 为逻辑尺寸，`device_scale_factor` 为设备缩放比，
    /// `physical_size` 为物理像素尺寸；本方法不推算或校正任一项。
    ///
    /// # 返回值与约束
    /// 原样保存参数的值；调用方负责提供一致的窗口观测，消费方采用自身检查或回退策略。
    ///
    /// # 参考
    /// [`crate::runtime_api::ZrRuntimeEventV1::viewport_metrics`] 与 [`Self`] 的消费边界。
    pub const fn new(
        logical_size: ZrRuntimeViewportSizeV1,
        device_scale_factor: f32,
        physical_size: ZrRuntimeViewportSizeV1,
    ) -> Self {
        Self {
            logical_size,
            device_scale_factor,
            physical_size,
        }
    }
}

impl ZrRuntimeViewportSizeV1 {
    /// 原样保存宽高，供不同请求各自解释和检查尺寸。
    ///
    /// # 参数与返回值
    /// `width`、`height` 是请求所用单位的尺寸；返回值不执行钳制或预算验证。
    ///
    /// # 参考
    /// [`Self`]、[`ZrRuntimeViewportMetricsV1::new`] 与 `render_surface_descriptor`。
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 宿主窗口的原生表面标识；实际绑定前运行时检查 ABI 版本和受支持的平台种类。
///
/// # 使用时机与生命周期
/// 宿主从仍存活的原生窗口取得句柄后，把目标交给表面绑定入口。
/// 目标不拥有窗口；宿主须保持句柄有效直到成功解绑或成功销毁持有表面的会话。
/// 绑定或销毁失败不证明原生借用已结束。
///
/// # 约束
/// 当前 `render_surface_descriptor` 只接纳 V1 的 Win32 目标且要求非零 HWND，
/// HINSTANCE 为零可表示未提供；构造器不验证这些前提。
///
/// # 参考
/// [`ZrRuntimeBindViewportSurfaceRequestV1`]、`runtime_native_surface_target` 与
/// `render_surface_descriptor`。
pub struct ZrRuntimeNativeSurfaceTargetV1 {
    /// 原生目标自身的 ABI 版本，绑定消费方与外层请求版本分别检查。
    pub abi_version: u32,
    /// 原生平台种类；当前表面绑定消费方只支持 Win32。
    pub kind: u32,
    /// Win32 目标的 HWND 数值；宿主独立保证其实际有效期。
    pub window_handle: u64,
    /// Win32 目标的可选 HINSTANCE；零值在消费层转换为未提供。
    pub display_handle: u64,
}

impl ZrRuntimeNativeSurfaceTargetV1 {
    /// 创建空表面占位描述，供调用方表达尚未取得原生目标的状态。
    ///
    /// # 参数与返回值
    /// 保存 `abi_version`，使用 NONE 种类和两个零句柄，不执行平台验证。
    ///
    /// # 约束
    /// 当前绑定入口拒绝 NONE；该描述不是解绑指令，解绑使用单独的 ABI 函数。
    ///
    /// # 参考
    /// `render_surface_descriptor`、[`crate::runtime_api::ZrRuntimeUnbindViewportSurfaceFnV1`]。
    pub const fn none(abi_version: u32) -> Self {
        Self {
            abi_version,
            kind: ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1,
            window_handle: 0,
            display_handle: 0,
        }
    }

    /// 包装宿主提供的 Win32 原生窗口目标。
    ///
    /// # 参数
    /// `abi_version` 为目标版本，`hwnd` 为窗口句柄，`hinstance` 为可选模块实例句柄的数值。
    ///
    /// # 返回值与约束
    /// 返回 Win32 种类的描述，不取得窗口所有权，也不检查版本或句柄有效性。
    /// HWND 非零和目标版本由绑定入口检查；宿主仍须独立保证真实窗口存活期。
    ///
    /// # 参考
    /// `runtime_native_surface_target`、`render_surface_descriptor` 与 [`Self`]。
    pub const fn win32(abi_version: u32, hwnd: u64, hinstance: u64) -> Self {
        Self {
            abi_version,
            kind: ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
            window_handle: hwnd,
            display_handle: hinstance,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 把一个视口绑定到宿主持有的原生窗口；窗口句柄须在绑定和呈现期间保持有效。
///
/// # 使用时机与生命周期
/// 传给 [`crate::runtime_api::ZrRuntimeBindViewportSurfaceFnV1`]，按值描述所选会话的视口表面。
/// 视口句柄须属于调用时的会话；当前 Runtime 入口只接纳该会话的默认视口。
/// 请求值不持有会话或窗口，宿主仍须覆盖原生表面的实际绑定寿命。
///
/// # 参考
/// `RuntimeSession::bind_viewport_surface`、`bind_viewport_surface`、`render_surface_descriptor`，
/// [`ZrRuntimeNativeSurfaceTargetV1`]。
pub struct ZrRuntimeBindViewportSurfaceRequestV1 {
    /// 绑定请求版本；与内嵌目标版本分别检查。
    pub abi_version: u32,
    /// 会话内的视口标识；只有数值不能证明其会话归属或存活。
    pub viewport: ZrRuntimeViewportHandle,
    /// 表面物理像素尺寸；当前绑定消费层将两个零分量分别钳制为 1。
    pub size: ZrRuntimeViewportSizeV1,
    /// 宿主持有的原生窗口目标；复制该值不持有原生资源。
    pub target: ZrRuntimeNativeSurfaceTargetV1,
}

/// Independently resolved surface-binding entry; the frozen V8 table is unchanged.
pub const ZR_RUNTIME_BIND_VIEWPORT_SURFACE_SYMBOL_V2: &[u8] =
    b"zircon_runtime_bind_viewport_surface_v2\0";

/// Requested presentation policy. Numeric tags are checked before becoming Rust enums.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimePresentModeV2 {
    Fifo = 0,
    Immediate = 1,
    Mailbox = 2,
    FifoRelaxed = 3,
    AutoVsync = 4,
    AutoNoVsync = 5,
}

impl ZrRuntimePresentModeV2 {
    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Fifo),
            1 => Some(Self::Immediate),
            2 => Some(Self::Mailbox),
            3 => Some(Self::FifoRelaxed),
            4 => Some(Self::AutoVsync),
            5 => Some(Self::AutoNoVsync),
            _ => None,
        }
    }
}

/// V2 is resolved through its own symbol, never through a larger V8 table.
/// The host retains the native target until successful unbind or session destruction.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeBindViewportSurfaceRequestV2 {
    pub abi_version: u32,
    pub size_bytes: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub target: ZrRuntimeNativeSurfaceTargetV1,
    pub present_mode: u32,
    pub reserved: u32,
}

impl ZrRuntimeBindViewportSurfaceRequestV2 {
    pub const fn new(
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        target: ZrRuntimeNativeSurfaceTargetV1,
    ) -> Self {
        Self {
            abi_version: crate::version::ZIRCON_RUNTIME_ABI_VERSION_V2,
            size_bytes: core::mem::size_of::<Self>() as u32,
            viewport,
            size,
            target,
            present_mode: ZrRuntimePresentModeV2::Fifo as u32,
            reserved: 0,
        }
    }

    pub const fn with_present_mode(mut self, mode: ZrRuntimePresentModeV2) -> Self {
        self.present_mode = mode as u32;
        self
    }

    pub const fn is_valid(&self) -> bool {
        self.abi_version == crate::version::ZIRCON_RUNTIME_ABI_VERSION_V2
            && self.size_bytes == core::mem::size_of::<Self>() as u32
            && self.reserved == 0
            && ZrRuntimePresentModeV2::from_raw(self.present_mode).is_some()
    }
}

impl ZrRuntimeBindViewportSurfaceRequestV1 {
    /// 封装一次待提交的视口表面绑定请求。
    ///
    /// # 参数
    /// `abi_version` 为请求版本，`viewport` 为调用会话内的视口，`size` 为表面像素尺寸，
    /// `target` 为宿主仍持有的原生目标。
    ///
    /// # 返回值与约束
    /// 原样保存参数，不校验、绑定或登记资源；版本、默认视口和目标种类由消费入口检查。
    /// 宿主的窗口借用前提持续到成功解绑或会话成功销毁，不能因构造器返回而结束。
    ///
    /// # 参考
    /// [`crate::runtime_api::ZrRuntimeBindViewportSurfaceFnV1`] 与 `render_surface_descriptor`。
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        target: ZrRuntimeNativeSurfaceTargetV1,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
            target,
        }
    }
}
