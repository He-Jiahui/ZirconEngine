/// 冻结桥接表内的局部位置，供插件调用和生命周期报告复用。
///
/// 槽位只对产生它的表有效，不能跨冻结表或持久化后当作稳定接口身份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InterfaceSlot(u32);

impl InterfaceSlot {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
