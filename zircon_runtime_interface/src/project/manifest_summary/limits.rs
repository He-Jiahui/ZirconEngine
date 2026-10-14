/// Maximum bytes admitted by the lightweight project-manifest reader.
/// 按原始 UTF-8 字节长度计数，恰好等于上限的清单允许进入摘要解析。
pub const MAX_PROJECT_MANIFEST_BYTES: usize = 4 * 1024 * 1024;

/// Maximum asset roots admitted by one project manifest.
/// 此数是整份清单 asset_roots 列表的最大项数，边界值允许。
pub const MAX_PROJECT_ASSET_ROOTS: usize = 4_096;

/// Maximum nested TOML container depth below the root manifest table.
/// 根表深度为 0；嵌套表和数组每层加一，超过此深度才拒绝。
pub const MAX_PROJECT_MANIFEST_NESTING_DEPTH: usize = 32;

/// Maximum cumulative key/value entries across all manifest TOML tables.
/// 计数跨整棵 TOML 值树累计，不按单个表分别重置。
pub const MAX_PROJECT_MANIFEST_TABLE_ENTRIES: usize = 16_384;

/// Maximum cumulative elements across all manifest TOML arrays.
/// 计数跨整棵 TOML 值树累计，不按单个数组分别重置。
pub const MAX_PROJECT_MANIFEST_ARRAY_ITEMS: usize = 65_536;
