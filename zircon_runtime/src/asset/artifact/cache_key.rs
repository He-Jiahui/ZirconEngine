use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LibraryCacheKey {
    source_hash: String,
    importer_version: u32,
    config_hash: String,
}

impl LibraryCacheKey {
    pub fn new(
        source_hash: impl Into<String>,
        importer_version: u32,
        config_hash: impl Into<String>,
    ) -> Self {
        Self {
            source_hash: source_hash.into(),
            importer_version,
            config_hash: config_hash.into(),
        }
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
