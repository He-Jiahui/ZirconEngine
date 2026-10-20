use std::num::NonZeroU64;

/// Opaque identifier assigned by the active platform backend to one live
/// native window. It is intentionally not serializable or reusable across
/// platform-host instances.
///
/// # 作用域与存活
/// 内部只保存后端提供的非零数值，不拥有或借用原生窗口对象。
/// 数值须在发放它的宿主上下文内解释，不作为持久化或跨宿主的身份。
/// 本类型不单独记录注册表或代际；复制、相等比较或 `new` 成功都不能证明窗口仍存活。
/// 需要抵御槽位复用与旧代际时，使用注册表发放的 [`WindowId`](super::WindowId)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativeWindowId(NonZeroU64);

impl NativeWindowId {
    /// 包装后端提供的原生窗口数值；不创建或查找窗口。
    ///
    /// 参数 `raw` 为 0 时返回 `None`，否则原样返回 `Some` 身份。
    /// 仅检查非零，不验证该值由哪个宿主发放、仍然存活或可作为平台原生句柄使用。
    pub const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    /// 按值返回保存的非零数值，供原宿主的映射查找使用。
    ///
    /// `self` 的复制不会保活窗口；返回值不附带当前存活、代际或平台句柄有效性的证明。
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}
