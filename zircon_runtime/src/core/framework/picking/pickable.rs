#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 命中后的遮挡与悬停策略彼此独立；不可悬停目标仍可阻断更低层目标。
pub struct Pickable {
    pub should_block_lower: bool,
    pub is_hoverable: bool,
}

impl Pickable {
    pub const IGNORE: Self = Self {
        should_block_lower: false,
        is_hoverable: false,
    };

    pub const NON_BLOCKING: Self = Self {
        should_block_lower: false,
        is_hoverable: true,
    };

    pub const BLOCKING_NON_HOVERABLE: Self = Self {
        should_block_lower: true,
        is_hoverable: false,
    };
}

impl Default for Pickable {
    fn default() -> Self {
        Self {
            should_block_lower: true,
            is_hoverable: true,
        }
    }
}
