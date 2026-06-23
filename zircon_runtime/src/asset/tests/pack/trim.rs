use super::*;

#[test]
fn unreferenced_asset_trimmed_and_reported() {
    let report = ZrPackTrimPlanner::trim(
        ZrPackTrimConfig::new(["scenes/main.zscene"]),
        [
            ZrPackTrimInputAsset::new("scenes/main.zscene").with_dependency("textures/hero.png"),
            ZrPackTrimInputAsset::new("textures/hero.png"),
            ZrPackTrimInputAsset::new("textures/unused.png"),
        ],
    );

    assert_eq!(
        report.included_assets,
        ["scenes/main.zscene", "textures/hero.png"]
    );
    assert_eq!(report.trimmed_asset_count(), 1);
    assert_eq!(report.trimmed_assets[0].path, "textures/unused.png");
    assert_eq!(
        report.trimmed_assets[0].reason,
        ZrPackTrimReason::Unreferenced
    );
    assert_eq!(
        report.diagnostics,
        ["trimmed asset textures/unused.png: unreferenced"]
    );
}

#[test]
fn asset_filter_trim_is_reported() {
    let report = ZrPackTrimPlanner::trim(
        ZrPackTrimConfig::new(["scenes/main.zscene"]).with_asset_filter("shipping"),
        [
            ZrPackTrimInputAsset::new("scenes/main.zscene")
                .with_dependency("textures/hero.png")
                .with_label("shipping"),
            ZrPackTrimInputAsset::new("textures/hero.png"),
            ZrPackTrimInputAsset::new("textures/loading.png").with_label("shipping"),
        ],
    );

    assert_eq!(report.included_assets, ["scenes/main.zscene"]);
    assert_eq!(report.trimmed_asset_count(), 2);
    assert_eq!(report.trimmed_assets[0].path, "textures/hero.png");
    assert_eq!(
        report.trimmed_assets[0].reason,
        ZrPackTrimReason::AssetFilterMismatch("shipping".to_string())
    );
    assert_eq!(report.trimmed_assets[1].path, "textures/loading.png");
    assert_eq!(
        report.trimmed_assets[1].reason,
        ZrPackTrimReason::Unreferenced
    );
    assert_eq!(
        report.diagnostics,
        [
            "trimmed asset textures/hero.png: asset_filter shipping did not match",
            "trimmed asset textures/loading.png: unreferenced"
        ]
    );
}

#[test]
fn duplicate_trim_input_path_is_reported() {
    let report = ZrPackTrimPlanner::trim(
        ZrPackTrimConfig::new(["scenes/main.zscene"]),
        [
            ZrPackTrimInputAsset::new("scenes/main.zscene"),
            ZrPackTrimInputAsset::new("scenes/main.zscene"),
        ],
    );

    assert_eq!(report.included_assets, ["scenes/main.zscene"]);
    assert_eq!(report.duplicate_assets, ["scenes/main.zscene"]);
    assert!(report.has_duplicate_assets());
    assert_eq!(
        report.diagnostics,
        ["asset scenes/main.zscene is duplicated in trim input"]
    );
}
