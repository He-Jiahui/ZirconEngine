use std::num::NonZeroU32;

use super::WindowRegistryId;

/// A non-persistent, generation-qualified address of a live platform window.
///
/// # 身份与生命周期
/// 注册表身份、槽号和非零代际共同限定一个窗口地址；相等与哈希比较全部三项。
/// 宿主注册表在解析时校验注册表、槽位占用和代际，关闭中窗口还会被常规路由拒绝。
/// 本值只保存数值，复制或保留它不持有资源租约，也不延长窗口或表面的生存期。
/// 该地址仅在发放它的宿主与进程中使用；销毁或重建后须重新取得身份，不用于持久化恢复。
/// 原生后端数值见 [`NativeWindowId`](super::NativeWindowId)；本类型不是该数值的直接别名。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId {
    /// 发放地址的注册表身份；即使槽号和代际相同，不同注册表的地址也不同。
    registry: WindowRegistryId,
    /// 注册表内的槽号，0 可合法使用；单独的槽号不能区分复用前后的窗口。
    slot: u32,
    /// 发放地址时的非零槽位代际；销毁后可复用时由注册表推进，本值不自行更新。
    generation: NonZeroU32,
}

impl WindowId {
    /// 在 crate 内封装已经选定的注册表、槽号与代际。
    ///
    /// `registry` 是发放地址的注册表身份，`slot` 是槽号，`generation` 是类型上非零的槽位代际。
    /// 返回值逐项保留参数；不查询注册表、预留槽位、验证存活或创建原生窗口。
    /// 生产路径由注册表注册及解析操作调用，测试也可构造尚未注册或已失效的地址。
    pub(crate) const fn new(registry: WindowRegistryId, slot: u32, generation: NonZeroU32) -> Self {
        Self {
            registry,
            slot,
            generation,
        }
    }

    /// 按值返回地址所属的注册表身份。
    ///
    /// `self` 是可复制的地址；返回身份不查询当前宿主，不能单独证明地址仍可路由。
    pub const fn registry(self) -> WindowRegistryId {
        self.registry
    }

    /// 按值返回注册表内的槽号，可能为 0。
    ///
    /// `self` 中的槽号须与注册表身份和代际一同使用；返回值不验证槽位是否仍被占用。
    pub const fn slot(self) -> u32 {
        self.slot
    }

    /// 按值返回该地址保存的非零代际数值。
    ///
    /// `self` 保存发放时的代际；返回值不读取槽位当前代际，也不同于主窗口角色或其他状态的代际。
    pub const fn generation(self) -> u32 {
        self.generation.get()
    }
}
