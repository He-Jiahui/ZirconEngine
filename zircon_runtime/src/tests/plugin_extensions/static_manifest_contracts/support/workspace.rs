use std::path::{Path, PathBuf};

pub(in crate::tests::plugin_extensions::static_manifest_contracts) fn plugins_workspace_root(
) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have a repository parent")
        .join("zircon_plugins")
}
