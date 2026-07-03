use super::*;

#[test]
fn source_template_emits_signing_and_cdn_release_contracts() {
    let cases = [
        (
            ExportTargetPlatform::Android,
            [
                (
                    "platform/android/signing.properties.example",
                    "Android signing configuration contract",
                    "ZR_ANDROID_KEYSTORE_PATH",
                ),
                (
                    "platform/android/play-publish.json",
                    "Android Play publishing metadata contract",
                    "serviceAccountJson",
                ),
                (
                    "platform/android/release-bundle.ps1",
                    "Android signed release bundle script",
                    "bundleRelease",
                ),
            ],
        ),
        (
            ExportTargetPlatform::Ios,
            [
                (
                    "platform/ios/ExportOptions.plist",
                    "iOS signing and export options contract",
                    "provisioningProfiles",
                ),
                (
                    "platform/ios/app-store-connect.env.example",
                    "iOS App Store Connect credential contract",
                    "ZR_APP_STORE_CONNECT_API_KEY_ID",
                ),
                (
                    "platform/ios/archive-export.ps1",
                    "iOS archive and export script",
                    "xcodebuild",
                ),
            ],
        ),
        (
            ExportTargetPlatform::WebGpu,
            [
                (
                    "platform/webgpu/public/_headers",
                    "WebGPU browser host CDN cache headers",
                    "immutable",
                ),
                (
                    "platform/webgpu/public/zircon-export.cdn-manifest.json",
                    "WebGPU browser host CDN deployment manifest",
                    "assetIntegrity",
                ),
                (
                    "platform/webgpu/deploy-cdn.mjs",
                    "WebGPU browser host CDN deployment contract",
                    "ZR_CDN_BASE_URL",
                ),
            ],
        ),
        (
            ExportTargetPlatform::Wasm,
            [
                (
                    "platform/wasm/public/_headers",
                    "WASM browser host CDN cache headers",
                    "immutable",
                ),
                (
                    "platform/wasm/public/zircon-export.cdn-manifest.json",
                    "WASM browser host CDN deployment manifest",
                    "assetIntegrity",
                ),
                (
                    "platform/wasm/deploy-cdn.mjs",
                    "WASM browser host CDN deployment contract",
                    "ZR_CDN_BASE_URL",
                ),
            ],
        ),
    ];

    for (platform, expected_files) in cases {
        let profile_name = format!("{}-release", platform.as_str());
        let mut manifest = ProjectManifest::new(
            "Platform Release Export Test",
            AssetUri::parse("res://scenes/main.zscene").unwrap(),
            1,
        );
        manifest.export_profiles = vec![ExportProfile::new(
            profile_name.clone(),
            RuntimeTargetMode::ClientRuntime,
            platform,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate)
        .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

        let plan = ExportBuildPlan::from_project_manifest(&manifest, &profile_name).unwrap();

        for (path, purpose, expected_contents) in expected_files {
            assert_eq!(generated_file_purpose(&plan, path), purpose);
            assert!(
                generated_file(&plan, path).contains(expected_contents),
                "{platform:?} generated `{path}` should contain `{expected_contents}`"
            );
        }
    }
}

#[test]
fn generated_release_adapters_gate_real_store_and_cdn_upload_inputs() {
    let cases = [
        (
            ExportTargetPlatform::Android,
            "platform/android/release-bundle.ps1",
            [
                "ZR_GOOGLE_PLAY_SERVICE_ACCOUNT_JSON",
                "ZR_GOOGLE_PLAY_PACKAGE_NAME",
                "Invoke-RestMethod",
                "androidpublisher/v3/applications/$packageName/edits",
            ],
        ),
        (
            ExportTargetPlatform::Ios,
            "platform/ios/archive-export.ps1",
            [
                "ZR_APP_STORE_CONNECT_PRIVATE_KEY_PATH",
                "xcrun altool --upload-app",
                "--apiKey $env:ZR_APP_STORE_CONNECT_API_KEY_ID",
                "--apiIssuer $env:ZR_APP_STORE_CONNECT_ISSUER_ID",
            ],
        ),
        (
            ExportTargetPlatform::WebGpu,
            "platform/webgpu/deploy-cdn.mjs",
            [
                "createHash('sha256')",
                "brotliCompress",
                "ZR_CDN_UPLOAD_COMMAND",
                "zircon-export.integrity.json",
            ],
        ),
        (
            ExportTargetPlatform::Wasm,
            "platform/wasm/deploy-cdn.mjs",
            [
                "createHash('sha256')",
                "brotliCompress",
                "ZR_CDN_UPLOAD_COMMAND",
                "zircon-export.integrity.json",
            ],
        ),
    ];

    for (platform, path, expected_fragments) in cases {
        let profile_name = format!("{}-release-adapter", platform.as_str());
        let mut manifest = ProjectManifest::new(
            "Platform Release Adapter Test",
            AssetUri::parse("res://scenes/main.zscene").unwrap(),
            1,
        );
        manifest.export_profiles = vec![ExportProfile::new(
            profile_name.clone(),
            RuntimeTargetMode::ClientRuntime,
            platform,
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
    }
}
