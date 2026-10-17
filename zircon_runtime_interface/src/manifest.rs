//! 插件清单与模块描述的跨 ABI 只读布局。
//! 描述符中的字节视图借用插件提供的存储；加载器必须在读取期间保持其地址和长度有效。

use crate::buffer::ZrByteSlice;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 运行时目标类别；数值会进入插件清单和 C ABI，不能按 Rust 枚举顺序推断。
pub enum ZrRuntimeTargetMode {
    ClientRuntime = 1,
    ServerRuntime = 2,
    EditorHost = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 插件模块由哪一种宿主入口加载；同一插件可在清单中声明多个模块类别。
pub enum ZrPluginModuleKind {
    Runtime = 1,
    Editor = 2,
    Native = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 插件入口报告中的借用描述；字节存储由报告生产方提供，消费方不接管释放责任。
pub struct ZrPluginModuleDescriptorV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub name: ZrByteSlice,
    pub crate_name: ZrByteSlice,
    pub target_modes: ZrByteSlice,
    pub capabilities: ZrByteSlice,
}

impl ZrPluginModuleDescriptorV1 {
    /// 创建空描述作为 ABI 初始化值；不分配字符串或能力列表，也不取得任何所有权。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            kind: ZrPluginModuleKind::Runtime as u32,
            name: ZrByteSlice::empty(),
            crate_name: ZrByteSlice::empty(),
            target_modes: ZrByteSlice::empty(),
            capabilities: ZrByteSlice::empty(),
        }
    }
}
