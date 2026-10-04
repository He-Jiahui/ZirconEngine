/// Maximum nested JSON depth accepted by the compatibility walker.
pub const MAX_RETIRED_ASSET_REF_MIGRATION_DEPTH: usize = 128;
/// Maximum JSON values inspected by one compatibility migration attempt.
pub const MAX_RETIRED_ASSET_REF_MIGRATION_NODES: usize = 2_000_000;
/// Maximum exact retired references resolved by one migration attempt.
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

    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    pub const fn max_references(self) -> usize {
        self.max_references
    }
}
