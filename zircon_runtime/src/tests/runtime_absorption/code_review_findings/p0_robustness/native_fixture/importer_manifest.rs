#[test]
fn review_d13_native_fixture_importer_is_manifest_described() {
    let fixture = include_str!(
        "../../../../../../../zircon_plugins/native_dynamic_fixture/native/src/lib.rs"
    );
    let plugin_toml =
        include_str!("../../../../../../../zircon_plugins/native_dynamic_fixture/plugin.toml");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let package_doc =
        include_str!("../../../../../../../docs/zircon_runtime/plugin/package_manifest.md");
    let native_doc = include_str!(
        "../../../../../../../docs/zircon_runtime/plugin/native_plugin_loader/index.md"
    );

    for required in [
        "\"runtime.asset.importer.native_dynamic_fixture.data_json\"",
        "[[asset_importers]]",
        "id = \"native_dynamic_fixture.data_json\"",
        "plugin_id = \"native_dynamic_fixture\"",
        "source_extensions = [\"json\"]",
        "output_kind = \"Data\"",
        "required_capabilities = [\"runtime.asset.importer.native_dynamic_fixture.data_json\"]",
    ] {
        assert!(
            plugin_toml.contains(required),
            "native fixture plugin.toml should describe importer manifest anchor `{required}`"
        );
    }
    for required in [
        "runtime.asset.importer.native_dynamic_fixture.data_json",
        "[[extensions]]",
        "point = \"runtime.asset.importer.data\"",
        "contribution = \"plugin.native_dynamic_fixture.data_json\"",
        "schema = \"zircon.runtime.asset-importer.data/1\"",
        "command=asset.import/native_dynamic_fixture.data_json;payload=ZRIMP001",
        "\"asset.import/native_dynamic_fixture.data_json\" =>",
    ] {
        assert!(
            fixture.contains(required),
            "native fixture runtime registration/command surface should contain `{required}`"
        );
    }

    for stale_gap in [
        "plugin.toml` 无 `[[asset_importers]]`",
        "registration manifest `extensions` 也未声明 `runtime.importer` 贡献",
        "importer 能力未进可发现清单",
    ] {
        assert!(
            !review_findings.contains(stale_gap),
            "D13 native fixture importer review text should not keep stale gap `{stale_gap}`"
        );
    }
    for doc_anchor in [
        "D13 native_dynamic_fixture importer self-description",
        "native_dynamic_fixture_importer_manifest_self_description_static_passed_cargo_deferred",
        "review_d13_native_fixture_importer_is_manifest_described",
        "runtime.asset.importer.native_dynamic_fixture.data_json",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || package_doc.contains(doc_anchor)
                || native_doc.contains(doc_anchor),
            "D13 native fixture importer docs should record `{doc_anchor}`"
        );
    }
}
