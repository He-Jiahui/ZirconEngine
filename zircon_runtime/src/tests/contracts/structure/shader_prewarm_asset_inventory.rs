use super::{assert_contains_all_exact, runtime_src_path};

fn source(relative: &str) -> String {
    let path = runtime_src_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn line_count(source: &str) -> usize {
    source.lines().count()
}

#[test]
fn shader_prewarm_inventory_separates_collection_snapshot_and_traversal_owners() {
    let root = source("bin/zircon_shader_prewarm/manifest/asset_inventory.rs");
    let snapshot = source("bin/zircon_shader_prewarm/manifest/asset_inventory/snapshot.rs");
    let traversal = source("bin/zircon_shader_prewarm/manifest/asset_inventory/traversal.rs");

    assert!(
        line_count(&root) <= 320,
        "asset inventory orchestration owner regrew to {} lines",
        line_count(&root)
    );
    assert!(
        line_count(&snapshot) <= 500,
        "warm snapshot owner regrew to {} lines",
        line_count(&snapshot)
    );
    assert!(
        line_count(&traversal) <= 220,
        "asset traversal owner regrew to {} lines",
        line_count(&traversal)
    );
    assert_contains_all_exact(
        "asset inventory root",
        &root,
        &[
            "mod snapshot;",
            "mod traversal;",
            "pub(crate) struct ShaderPrewarmAssetInventory",
            "fn collect_fresh_with_text_budget_excluding",
        ],
    );
    assert_contains_all_exact(
        "warm snapshot owner",
        &snapshot,
        &[
            "struct ShaderPrewarmAssetInventorySnapshot",
            "fn load_snapshot(",
            "fn write_snapshot(",
            "fn snapshot_entry_paths_are_safe(",
            "fn write_snapshot_json(",
        ],
    );
    assert_contains_all_exact(
        "asset traversal owner",
        &traversal,
        &[
            "fn collect_file_paths(",
            "fn ensure_below_root(",
            "fn reject_link_or_reparse(",
            "fn is_reparse_point(",
        ],
    );
}
