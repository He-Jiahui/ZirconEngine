//! App 窗口与动态 Runtime 呈现表面的生命周期边界。
//! 默认原生表面和显式 CPU 诊断路径互斥。

mod binding;
mod lifecycle;
mod redraw;
mod reference_cpu;
mod resize;

pub(in crate::entry::runtime_entry_app) use resize::surface_resize_changes_viewport;
