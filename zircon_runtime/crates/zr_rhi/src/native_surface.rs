//! 平台原生表面目标的最小句柄描述；具体窗口桥接由 WGPU/宿主层负责。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 当前支持的原生窗口目标，数值仅用于跨边界传递，不承担生命周期所有权。
pub enum RenderNativeSurfaceTarget {
    Win32 { hwnd: u64, hinstance: Option<u64> },
}
