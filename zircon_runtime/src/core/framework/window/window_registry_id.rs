use std::num::NonZeroU64;

/// Identifies one platform-host window registry within the running process.
///
/// It prevents a handle obtained from an old or different host instance from
/// addressing a live window in the current host.
///
/// # 作用域与存活
/// 保存注册表的非零数值身份，用于区分同一进程内的宿主窗口注册表。
/// 常规 driver 通过共享分配器取得新值；本类型的 `new` 仅包装非零数值，不负责分配或去重。
/// 复制或保存此值不保活宿主、窗口或表面，也不表示当前注册表仍存在。
/// 只在对应进程与宿主上下文中解释，不作为持久化或跨进程身份；窗口地址另见 [`WindowId`](super::WindowId)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowRegistryId(NonZeroU64);

impl WindowRegistryId {
    /// 将已有非零数值包装为注册表身份。
    ///
    /// `raw` 为调用方提供的数值；0 返回 `None`，其余值原样包装为 `Some`。
    /// 本方法不分配身份，不检查唯一性、所属 driver 或注册表是否存活。
    pub const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    /// 按值取出此身份的非零数值。
    ///
    /// `self` 为可复制的身份值；返回原 `u64`，不查询注册表或延长宿主资源的生存期。
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}
