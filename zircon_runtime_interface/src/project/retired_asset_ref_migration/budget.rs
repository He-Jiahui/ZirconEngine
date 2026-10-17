/// Maximum nested JSON depth accepted by the compatibility walker.
/// 根值深度为 0；深度恰好达到上限仍可通过。
pub const MAX_RETIRED_ASSET_REF_MIGRATION_DEPTH: usize = 128;
/// Maximum JSON values inspected by one compatibility migration attempt.
/// 根值和每个子值都计为节点，对象键名本身不计。
pub const MAX_RETIRED_ASSET_REF_MIGRATION_NODES: usize = 2_000_000;
/// Maximum exact retired references resolved by one migration attempt.
/// 只按精确的 uuid/url 双键形状计数，不预先解析字段值。
pub const MAX_RETIRED_ASSET_REF_MIGRATION_REFERENCES: usize = 1_000_000;

/// Caller-owned limits for one retired asset-reference migration attempt.
/// 单次旧资产引用迁移的调用方预算；完整遍历通过后才会调用身份解析器。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetiredAssetRefMigrationBudget {
    max_nodes: usize,
    max_depth: usize,
    max_references: usize,
}

impl RetiredAssetRefMigrationBudget {
    /// # 参数
    ///
    /// - `max_nodes` 限制整棵 JSON 值树的节点数，根值也计入。
    /// - `max_depth` 以根值深度 0 计数，超出而非等于上限时拒绝。
    /// - `max_references` 限制键恰为 `uuid` 和 `url` 的对象数量。
    ///
    /// # 约束
    ///
    /// 数值按调用方原样使用，不会被默认上限截断；超限判定由迁移入口执行。
    pub const fn new(max_nodes: usize, max_depth: usize, max_references: usize) -> Self {
        Self {
            max_nodes,
            max_depth,
            max_references,
        }
    }

    /// 默认递归迁移入口共用的节点、深度与引用数上限。
    pub const fn standard() -> Self {
        Self::new(
            MAX_RETIRED_ASSET_REF_MIGRATION_NODES,
            MAX_RETIRED_ASSET_REF_MIGRATION_DEPTH,
            MAX_RETIRED_ASSET_REF_MIGRATION_REFERENCES,
        )
    }

    /// 返回每次迁移允许遍历的最大 JSON 值节点数。
    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    /// 返回允许的最大嵌套深度；根值的深度为 0。
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    /// 返回可解析的精确旧引用对象数上限。
    pub const fn max_references(self) -> usize {
        self.max_references
    }
}
