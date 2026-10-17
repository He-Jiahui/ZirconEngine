/// 冻结桥接表内的局部位置，供插件调用和生命周期报告复用。
///
/// 槽位只对产生它的表有效，不能跨冻结表或持久化后当作稳定接口身份。
///
/// # 构造与验证边界
/// 本类型是中立的 Rust 位置值；[`Self::from_raw`] 不检查对应表中的 entry。
/// 当前原生宿主回调传递 `u32` 编号，在入口包装成此类型后由冻结表检查 entry 是否存在。
/// 这条路径传递编号值，不要求本类型具有 C ABI 布局。
///
/// # 参考
/// [`Self::from_raw`]、[`Self::raw`]、[`Self::index`]、
/// [`super::BridgeInvocationTable::resolve_interface_slot`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InterfaceSlot(u32);

impl InterfaceSlot {
    /// 将原始编号包装为 Rust 槽位值，供相应冻结表继续查验。
    ///
    /// # 参数
    /// - `raw`：任意 `u32` 编号；函数不接收表，也不检查编号是否属于某张表或是否越界。
    ///
    /// # 返回
    /// 返回包含该编号的 [`InterfaceSlot`]，不验证 entry 是否存在，且没有错误返回。
    /// 取得一个槽位值不能证明接口存在或当前可调用。
    ///
    /// # 使用时机与约束
    /// 当前原生入口从 ABI 的 `u32` 编号构造槽位后，再由冻结表查验 entry；
    /// 没有 entry 时由入口返回错误。调用者必须继续使用与该编号关联的表，
    /// 不能将包装后的编号当作跨表或持久化的稳定身份。
    ///
    /// # 参考
    /// [`Self::raw`]、[`Self::index`]、[`super::BridgeInvocationTable::interface_status_at`]。
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// 返回保存的原始槽号，供同一冻结表的调用入口传递位置。
    /// 槽号所属表与验证前提见 [`Self::from_raw`]。
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// 将槽号转换为数组索引用的 `usize`；entry 是否越界仍由持有表的调用方检查。
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
