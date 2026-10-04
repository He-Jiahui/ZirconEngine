use std::collections::BTreeMap;

pub(crate) fn directives(
    compiler_verbose_version: &[u8],
    configuration: &BTreeMap<String, String>,
) -> Result<[String; 2], String> {
    for required in [
        "TARGET",
        "HOST",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
        "CARGO_PKG_VERSION",
    ] {
        if configuration.get(required).is_none_or(String::is_empty) {
            return Err(format!(
                "Cargo must provide {required} for import compatibility"
            ));
        }
    }
    if compiler_verbose_version.is_empty() {
        return Err("compiler identity was empty".to_owned());
    }
    let target = &configuration["TARGET"];
    if target.chars().any(char::is_control) {
        return Err("Cargo TARGET must be a single printable value".to_owned());
    }
    let identity = super::fingerprint::fingerprint(compiler_verbose_version, configuration);
    Ok([
        format!("cargo:rustc-env=ZR_ASSET_IMPORT_BUILD_TARGET={target}"),
        format!("cargo:rustc-env=ZR_ASSET_IMPORT_BUILD_ID={identity}"),
    ])
}
