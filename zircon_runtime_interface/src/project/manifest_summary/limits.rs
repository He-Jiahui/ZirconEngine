/// Maximum bytes admitted by the lightweight project-manifest reader.
pub const MAX_PROJECT_MANIFEST_BYTES: usize = 4 * 1024 * 1024;

/// Maximum asset roots admitted by one project manifest.
pub const MAX_PROJECT_ASSET_ROOTS: usize = 4_096;

/// Maximum nested TOML container depth below the root manifest table.
pub const MAX_PROJECT_MANIFEST_NESTING_DEPTH: usize = 32;

/// Maximum cumulative key/value entries across all manifest TOML tables.
pub const MAX_PROJECT_MANIFEST_TABLE_ENTRIES: usize = 16_384;

/// Maximum cumulative elements across all manifest TOML arrays.
pub const MAX_PROJECT_MANIFEST_ARRAY_ITEMS: usize = 65_536;
