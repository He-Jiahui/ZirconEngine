use std::collections::BTreeMap;

pub(crate) fn is_compatibility_variable(name: &str) -> bool {
    name.starts_with("CARGO_CFG_")
        || name.starts_with("CARGO_FEATURE_")
        || matches!(
            name,
            "TARGET"
                | "HOST"
                | "PROFILE"
                | "OPT_LEVEL"
                | "DEBUG"
                | "CARGO_PKG_VERSION"
                | "CARGO_ENCODED_RUSTFLAGS"
        )
}

pub(crate) fn canonical_configuration(
    environment: impl IntoIterator<Item = (String, String)>,
) -> BTreeMap<String, String> {
    environment
        .into_iter()
        .filter(|(name, _)| is_compatibility_variable(name))
        .map(|(name, value)| {
            // Cargo encodes cfg value sets as comma-delimited lists; their order is not semantic.
            let value = if name.starts_with("CARGO_CFG_") {
                let mut values: Vec<_> = value.split(',').collect();
                values.sort_unstable();
                values.dedup();
                values.join(",")
            } else {
                value
            };
            (name, value)
        })
        .collect()
}

pub(crate) fn fingerprint(
    compiler_verbose_version: &[u8],
    configuration: &BTreeMap<String, String>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"zircon.runtime.asset.import.build.v1");
    hash_field(&mut hasher, compiler_verbose_version);
    hasher.update(&(configuration.len() as u64).to_le_bytes());
    for (name, value) in configuration {
        hash_field(&mut hasher, name.as_bytes());
        hash_field(&mut hasher, value.as_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn hash_field(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}
