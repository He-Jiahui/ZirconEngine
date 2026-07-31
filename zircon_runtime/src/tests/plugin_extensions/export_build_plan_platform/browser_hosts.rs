use super::*;

#[test]
fn generated_browser_hosts_instantiate_wasm_exports_and_gate_asset_origins() {
    let cases = [
        (
            ExportTargetPlatform::WebGpu,
            "webgpu",
            "platform/webgpu/src/zircon_webgpu_host.js",
            [
                "WebAssembly.instantiateStreaming(fetch(manifest.wasmModule), zirconExportImports)",
                "const zirconRuntimeExports = wasmInstance.exports;",
                "zirconRuntimeExports.zircon_export_start?.();",
                "zirconRuntimeExports.zircon_export_handle_touch?.(BigInt(pointerId), phase, x, y);",
                "if (!url.pathname.startsWith(new URL(manifest.allowedAssetRoot, location.href).pathname))",
                "throw new Error(`Blocked Zircon resource fetch outside ${manifest.allowedAssetRoot}: ${uri}`);",
            ],
        ),
        (
            ExportTargetPlatform::Wasm,
            "wasm",
            "platform/wasm/src/zircon_wasm_host.js",
            [
                "WebAssembly.instantiateStreaming(fetch(manifest.wasmModule), zirconExportImports)",
                "const zirconRuntimeExports = wasmInstance.exports;",
                "zirconRuntimeExports.zircon_export_start?.();",
                "zirconRuntimeExports.zircon_export_handle_touch?.(BigInt(pointerId), phase, x, y);",
                "if (!url.pathname.startsWith(new URL(manifest.allowedAssetRoot, location.href).pathname))",
                "throw new Error(`Blocked Zircon resource fetch outside ${manifest.allowedAssetRoot}: ${uri}`);",
            ],
        ),
    ];

    for (platform, host_name, path, expected_fragments) in cases {
        let profile_name = format!("{}-wasm-glue", platform.as_str());
        let mut manifest = ProjectManifest::new(
            "Browser Glue Test",
            AssetUri::parse("res://scenes/main.zscene").unwrap(),
            1,
        );
        manifest.export_profiles = vec![ExportProfile::new(
            profile_name.clone(),
            RuntimeTargetMode::ClientRuntime,
            platform,
            RuntimeProfileId::Client2d,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate)
        .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

        let plan = ExportBuildPlan::from_project_manifest(&manifest, &profile_name).unwrap();
        let generated = generated_file(&plan, path);

        for expected_fragment in expected_fragments {
            assert!(
                generated.contains(expected_fragment),
                "{platform:?} generated `{path}` should contain `{expected_fragment}`"
            );
        }

        assert!(generated_file(
            &plan,
            &format!("platform/{host_name}/public/zircon-export.manifest.json")
        )
        .contains("\"allowedAssetRoot\": \"./assets/\""));
    }
}
