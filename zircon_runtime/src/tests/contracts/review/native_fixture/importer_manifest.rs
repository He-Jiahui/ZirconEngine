//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn review_d13_native_fixture_importer_is_manifest_described() {
    let fixture =
        include_str!("../../../../../../zircon_plugins/native_dynamic_fixture/native/src/lib.rs");
    let plugin_toml =
        include_str!("../../../../../../zircon_plugins/native_dynamic_fixture/plugin.toml");
    let review_findings =
        include_str!("../../../../../../docs/architecture/runtime-architecture-review-m0.md");
    let package_doc =
        include_str!("../../../../../../docs/crates/zircon_runtime/plugin/package_manifest.md");
    let native_doc = include_str!(
        "../../../../../../docs/crates/zircon_runtime/plugin/native_plugin_loader/index.md"
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
    let runtime_projection = fixture
        .split("native_projection: {")
        .nth(1)
        .and_then(|projection| projection.split("editor: {").next())
        .expect("typed native runtime projection");
    for required in [
        "registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST",
        "extensions: [{",
        "point: \"runtime.asset.importer.data\"",
        "contribution: \"plugin.native_dynamic_fixture.data_json\"",
        "schema: \"zircon.runtime.asset-importer.data/1\"",
    ] {
        assert!(runtime_projection.contains(required), "{required}");
    }
    assert!(fixture.contains("zircon_plugin_sdk::declare_plugin!"));
    assert!(fixture.contains("registration_manifest: Some(NATIVE_RUNTIME_REGISTRATION_MANIFEST)"));
    let sdk_declaration =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/declaration.rs");
    assert!(sdk_declaration.contains("#[path = \"declaration/macros.rs\"]"));
    let sdk_projection =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/declaration/macros.rs");
    for required in [
        "\"[[extensions]]\\n\",",
        "\"point = \\\"\", $native_extension_point,",
        "\"contribution = \\\"\", $native_extension_contribution,",
        "\"schema = \\\"\", $native_extension_schema,",
    ] {
        assert!(sdk_projection.contains(required), "{required}");
    }
    let command_source = fixture
        .split("const RUNTIME_COMMAND_MANIFEST_TEXT: &str = concat!(")
        .nth(1)
        .and_then(|source| source.split("r#\"").nth(1))
        .and_then(|source| source.split("\"#,").next())
        .expect("actual runtime V4 command manifest");
    let commands: toml::Value = toml::from_str(command_source).expect("V4 command manifest TOML");
    assert_eq!(
        commands["schema"].as_str(),
        Some("zircon.native.command-manifest/4")
    );
    let importer_commands = commands["commands"]
        .as_array()
        .expect("declared native commands")
        .iter()
        .filter(|command| {
            command["name"].as_str() == Some("asset.import/native_dynamic_fixture.data_json")
        })
        .collect::<Vec<_>>();
    assert_eq!(importer_commands.len(), 1);
    assert_eq!(
        importer_commands[0]["payload_schema"].as_str(),
        Some("ZRIMP001")
    );
    let importer_slot = importer_commands[0]["slot"]
        .as_integer()
        .expect("V4 importer command must declare its numeric dispatch slot");
    assert_eq!(importer_slot, 3);
    assert!(fixture.contains(&format!(
        "const COMMAND_ASSET_IMPORT_SLOT: u32 = {importer_slot};"
    )));
    let dispatch = fixture
        .split_once("unsafe fn fixture_invoke_command_inner(")
        .expect("the registered callback must retain its real slot dispatcher")
        .1
        .split_once("unsafe fn fixture_import_data_json(")
        .expect("the importer handler must follow the dispatcher")
        .0;
    assert!(dispatch.contains("match command_slot {"));
    assert!(dispatch
        .contains("COMMAND_ASSET_IMPORT_SLOT => fixture_import_data_json(payload, output),"));
    assert!(fixture.contains("command_manifest: Some(RUNTIME_COMMAND_MANIFEST)"));
    assert!(fixture.contains("invoke_command: Some(fixture_invoke_command)"));
    assert!(fixture.contains("fixture_invoke_command_inner(command_slot, payload, output)"));

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
