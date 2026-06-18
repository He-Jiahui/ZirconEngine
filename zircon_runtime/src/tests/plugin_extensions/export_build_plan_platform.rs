use crate::asset::{AssetUri, ProjectManifest};
use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::{
    plugin::ExportBuildPlan, plugin::ExportPackagingStrategy, plugin::ExportPlatformHostKind,
    plugin::ExportPlatformPluginStrategy, plugin::ExportPlatformResourceStrategy,
    plugin::ExportProfile, plugin::ExportTargetPlatform, plugin::ProjectPluginManifest,
    plugin::ProjectPluginSelection, plugin::RuntimePluginAvailabilityEntry,
    plugin::RuntimeProfileId,
};

#[test]
fn mobile_and_web_targets_reject_native_dynamic_packaging() {
    assert!(!ExportTargetPlatform::Android.supports_native_dynamic());
    assert!(!ExportTargetPlatform::Ios.supports_native_dynamic());
    assert!(!ExportTargetPlatform::WebGpu.supports_native_dynamic());
    assert!(!ExportTargetPlatform::Wasm.supports_native_dynamic());

    let web_policy = ExportTargetPlatform::WebGpu.policy();
    assert_eq!(web_policy.host_kind, ExportPlatformHostKind::Browser);
    assert_eq!(
        web_policy.resource_strategy,
        ExportPlatformResourceStrategy::BrowserFetch
    );
    assert_eq!(
        web_policy.plugin_strategy,
        ExportPlatformPluginStrategy::StaticSourceOrVmOnly
    );

    for platform in [
        ExportTargetPlatform::Android,
        ExportTargetPlatform::Ios,
        ExportTargetPlatform::WebGpu,
        ExportTargetPlatform::Wasm,
    ] {
        let mut manifest = ProjectManifest::new(
            "Mobile Web Export Test",
            AssetUri::parse("res://scenes/main.zscene").unwrap(),
            1,
        );
        manifest.plugins = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Sound,
                true,
                true,
            )
            .with_runtime_crate("zircon_plugin_sound_runtime")
            .with_packaging(ExportPackagingStrategy::NativeDynamic)],
        };
        let profile_name = format!("native-{}", platform.as_str());
        manifest.export_profiles = vec![ExportProfile::new(
            profile_name.clone(),
            RuntimeTargetMode::ClientRuntime,
            platform,
        )
        .with_strategies([ExportPackagingStrategy::NativeDynamic])];

        let plan = ExportBuildPlan::from_project_manifest(&manifest, &profile_name).unwrap();

        assert_eq!(plan.platform_policy.target_platform, platform);
        assert_eq!(
            plan.platform_policy.plugin_strategy,
            ExportPlatformPluginStrategy::StaticSourceOrVmOnly
        );
        assert!(
            plan.native_dynamic_packages.is_empty(),
            "{platform:?} should not export native dynamic packages"
        );
        assert!(plan.has_fatal_diagnostics());
        assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic.contains(
            &format!(
                "export profile {profile_name} enables NativeDynamic but target platform {} does not support dynamic libraries",
                platform.as_str()
            )
        )));
        assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic.contains(
            &format!(
                "plugin sound uses NativeDynamic packaging but target platform {} does not support dynamic libraries",
                platform.as_str()
            )
        )));
        assert!(plan
            .generated_files
            .iter()
            .all(|file| file.path != "plugins/native_plugins.toml"));
        assert!(availability_contains(
            &plan.runtime_plugin_availability.externalized_missing,
            "sound"
        ));
        assert!(availability_contains(
            &plan.runtime_plugin_availability.missing_required,
            "sound"
        ));
    }
}

#[test]
fn platform_target_policy_matches_host_resource_and_plugin_strategy() {
    let all_cases = [
        (
            ExportTargetPlatform::Windows,
            ExportPlatformHostKind::Desktop,
            ExportPlatformResourceStrategy::FilesystemBundle,
            ExportPlatformPluginStrategy::NativeDynamicAllowed,
            true,
        ),
        (
            ExportTargetPlatform::Linux,
            ExportPlatformHostKind::Desktop,
            ExportPlatformResourceStrategy::FilesystemBundle,
            ExportPlatformPluginStrategy::NativeDynamicAllowed,
            true,
        ),
        (
            ExportTargetPlatform::Macos,
            ExportPlatformHostKind::Desktop,
            ExportPlatformResourceStrategy::FilesystemBundle,
            ExportPlatformPluginStrategy::NativeDynamicAllowed,
            true,
        ),
        (
            ExportTargetPlatform::Android,
            ExportPlatformHostKind::MobileApp,
            ExportPlatformResourceStrategy::MobileAssetBundle,
            ExportPlatformPluginStrategy::StaticSourceOrVmOnly,
            false,
        ),
        (
            ExportTargetPlatform::Ios,
            ExportPlatformHostKind::MobileApp,
            ExportPlatformResourceStrategy::MobileAssetBundle,
            ExportPlatformPluginStrategy::StaticSourceOrVmOnly,
            false,
        ),
        (
            ExportTargetPlatform::WebGpu,
            ExportPlatformHostKind::Browser,
            ExportPlatformResourceStrategy::BrowserFetch,
            ExportPlatformPluginStrategy::StaticSourceOrVmOnly,
            false,
        ),
        (
            ExportTargetPlatform::Wasm,
            ExportPlatformHostKind::Browser,
            ExportPlatformResourceStrategy::BrowserFetch,
            ExportPlatformPluginStrategy::StaticSourceOrVmOnly,
            false,
        ),
        (
            ExportTargetPlatform::Headless,
            ExportPlatformHostKind::Headless,
            ExportPlatformResourceStrategy::FilesystemBundle,
            ExportPlatformPluginStrategy::NativeDynamicAllowed,
            true,
        ),
    ];
    let requested_platform = std::env::var("ZR_EXPORT_CONTRACT_PLATFORM")
        .ok()
        .map(|value| export_target_platform_from_ci_name(&value));

    for (platform, host_kind, resource_strategy, plugin_strategy, supports_native_dynamic) in
        all_cases
    {
        if requested_platform.is_some_and(|requested| requested != platform) {
            continue;
        }
        let policy = platform.policy();
        assert_eq!(policy.target_platform, platform);
        assert_eq!(policy.host_kind, host_kind);
        assert_eq!(policy.resource_strategy, resource_strategy);
        assert_eq!(policy.plugin_strategy, plugin_strategy);
        assert_eq!(policy.supports_native_dynamic, supports_native_dynamic);
        assert_eq!(platform.supports_native_dynamic(), supports_native_dynamic);
    }
}

#[test]
fn source_template_emits_headless_host_scaffold_without_platform_shell() {
    let mut manifest = ProjectManifest::new(
        "Headless Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.export_profiles = vec![ExportProfile::new(
        "server",
        RuntimeTargetMode::ServerRuntime,
        ExportTargetPlatform::Headless,
    )
    .with_runtime_profile_id(RuntimeProfileId::Server)
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "server").unwrap();

    assert_eq!(
        plan.platform_policy.host_kind,
        ExportPlatformHostKind::Headless
    );
    assert_eq!(
        generated_file_purpose(&plan, "src/main.rs"),
        "generated headless runtime entry point"
    );
    assert!(generated_file(&plan, "src/zircon_plugins.rs").contains("EntryProfile::Headless"));
    assert!(generated_file(&plan, "src/main.rs").contains("zircon_app::bootstrap_export_runtime"));
    assert!(
        generated_file(&plan, "src/zircon_plugins.rs").contains("ExportTargetPlatform::Headless")
    );
    assert!(generated_file(&plan, "Cargo.toml").contains("features = [\"target-server\"]"));
    assert!(plan
        .generated_files
        .iter()
        .all(|file| !file.path.starts_with("platform/")));
    assert!(plan
        .generated_files
        .iter()
        .all(|file| file.path != "src/lib.rs"));
}

#[test]
fn source_template_emits_mobile_and_browser_host_scaffolds() {
    let cases = [
        (
            ExportTargetPlatform::Android,
            "src/lib.rs",
            "platform/android/app/src/main/AndroidManifest.xml",
            "platform/android/app/src/main/java/dev/zircon/export/MainActivity.kt",
            "Android mobile asset host",
        ),
        (
            ExportTargetPlatform::Ios,
            "src/lib.rs",
            "platform/ios/ZirconRuntimeHost/Resources/Info.plist",
            "platform/ios/ZirconRuntimeHost/Sources/ZirconRuntimeHostApp.swift",
            "iOS mobile asset host",
        ),
        (
            ExportTargetPlatform::WebGpu,
            "src/lib.rs",
            "platform/webgpu/index.html",
            "platform/webgpu/src/zircon_webgpu_host.js",
            "WebGPU browser host",
        ),
        (
            ExportTargetPlatform::Wasm,
            "src/lib.rs",
            "platform/wasm/index.html",
            "platform/wasm/src/zircon_wasm_host.js",
            "WASM browser host",
        ),
    ];

    for (platform, runtime_entry, shell_file, launcher_file, host_label) in cases {
        let profile_name = format!("{}-source", platform.as_str());
        let mut manifest = ProjectManifest::new(
            "Platform Host Export Test",
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

        assert_eq!(plan.platform_policy.target_platform, platform);
        assert_eq!(
            generated_file_purpose(&plan, runtime_entry),
            format!("generated {host_label} runtime library entry point")
        );
        assert!(generated_file(&plan, runtime_entry).contains("zircon_export_start"));
        assert!(generated_file(&plan, runtime_entry).contains(platform.as_str()));
        assert!(!plan
            .generated_files
            .iter()
            .any(|file| file.path == "src/main.rs"));
        assert!(generated_file(&plan, shell_file).contains(&profile_name));
        assert!(!generated_file(&plan, launcher_file).is_empty());
        assert!(generated_file(&plan, "src/zircon_plugins.rs").contains("project_plugins"));
        assert!(generated_file(&plan, "assets/zircon-project.toml")
            .contains("Platform Host Export Test"));
        assert!(plan
            .generated_files
            .iter()
            .all(|file| file.path != "plugins/native_plugins.toml"));
    }
}

#[test]
fn source_template_emits_package_manifests_for_mobile_and_browser_hosts() {
    let cases = [
        (
            ExportTargetPlatform::Android,
            [
                "platform/android/settings.gradle.kts",
                "platform/android/app/build.gradle.kts",
                "platform/android/app/src/main/jniLibs/README.md",
                "platform/android/package-export.ps1",
            ],
            [
                "Android Gradle settings manifest",
                "Android application packaging manifest",
                "Android native library placement contract",
                "Android release packaging script",
            ],
        ),
        (
            ExportTargetPlatform::Ios,
            [
                "platform/ios/Package.swift",
                "platform/ios/ZirconRuntimeHost/Resources/zircon-export.bundle.toml",
                "platform/ios/ZirconRuntimeHost/Linking/zircon_runtime_native.h",
                "platform/ios/package-export.ps1",
            ],
            [
                "iOS Swift package manifest",
                "iOS bundled resource manifest pointer",
                "iOS Rust static library C header",
                "iOS release packaging script",
            ],
        ),
        (
            ExportTargetPlatform::WebGpu,
            [
                "platform/webgpu/package.json",
                "platform/webgpu/vite.config.mjs",
                "platform/webgpu/public/zircon-export.manifest.json",
                "platform/webgpu/package-export.mjs",
            ],
            [
                "WebGPU browser host package manifest",
                "WebGPU browser host dev and release server config",
                "WebGPU browser host fetch manifest",
                "WebGPU browser host release packaging script",
            ],
        ),
        (
            ExportTargetPlatform::Wasm,
            [
                "platform/wasm/package.json",
                "platform/wasm/vite.config.mjs",
                "platform/wasm/public/zircon-export.manifest.json",
                "platform/wasm/package-export.mjs",
            ],
            [
                "WASM browser host package manifest",
                "WASM browser host dev and release server config",
                "WASM browser host fetch manifest",
                "WASM browser host release packaging script",
            ],
        ),
    ];

    for (platform, paths, purposes) in cases {
        let profile_name = format!("{}-package", platform.as_str());
        let mut manifest = ProjectManifest::new(
            "Platform Package Export Test",
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

        for (path, purpose) in paths.into_iter().zip(purposes) {
            assert_eq!(generated_file_purpose(&plan, path), purpose);
            assert!(
                !generated_file(&plan, path).trim().is_empty(),
                "{platform:?} generated `{path}` should have contents"
            );
        }
    }
}

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
fn generated_mobile_and_browser_hosts_translate_platform_callbacks_to_runtime_abi_events() {
    let cases = [
        (
            ExportTargetPlatform::Android,
            "platform/android/app/src/main/java/dev/zircon/export/MainActivity.kt",
            [
                "dispatchLifecycle(ZIRCON_LIFECYCLE_RESUMED)",
                "dispatchTouch(event.getPointerId(index).toLong(), phase, event.getX(index), event.getY(index))",
                "dispatchKeyboard(ZIRCON_KEY_PRESSED, event.keyCode, event.scanCode, null)",
                "dispatchViewportMetrics(width, height, resources.displayMetrics.density)",
            ],
        ),
        (
            ExportTargetPlatform::Ios,
            "platform/ios/ZirconRuntimeHost/Sources/ZirconRuntimeHostApp.swift",
            [
                "zircon_export_handle_lifecycle(ZIRCON_LIFECYCLE_RESUMED)",
                "zircon_export_handle_touch(UInt64(touch.hash), ZIRCON_TOUCH_MOVED",
                "zircon_export_handle_keyboard(ZIRCON_KEY_TEXT, 0, 0",
                "zircon_export_handle_viewport_metrics(UInt32(size.width), UInt32(size.height), Float(scale))",
            ],
        ),
        (
            ExportTargetPlatform::WebGpu,
            "platform/webgpu/src/zircon_webgpu_host.js",
            [
                "zirconExportDispatchLifecycle('resumed')",
                "zirconExportDispatchPointer(event.pointerId, 'moved', event.clientX, event.clientY)",
                "zirconExportDispatchKeyboard('pressed', event.code, event.key)",
                "zirconExportFetchResource(uri, { streaming = false } = {})",
            ],
        ),
        (
            ExportTargetPlatform::Wasm,
            "platform/wasm/src/zircon_wasm_host.js",
            [
                "zirconExportDispatchLifecycle('resumed')",
                "zirconExportDispatchPointer(event.pointerId, 'moved', event.clientX, event.clientY)",
                "zirconExportDispatchKeyboard('pressed', event.code, event.key)",
                "zirconExportFetchResource(uri, { streaming = false } = {})",
            ],
        ),
    ];

    for (platform, path, expected_fragments) in cases {
        let profile_name = format!("{}-host-callbacks", platform.as_str());
        let mut manifest = ProjectManifest::new(
            "Platform Callback Export Test",
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

#[test]
fn generated_platform_hosts_include_repo_owned_binding_and_resource_glue() {
    let cases = [
        (
            ExportTargetPlatform::Android,
            [
                (
                    "platform/android/app/src/main/java/dev/zircon/export/ZirconRuntime.kt",
                    [
                        "external fun start(): Boolean",
                        "external fun dispatchLifecycle(state: Int): Boolean",
                        "external fun dispatchTouch(pointerId: Long, phase: Int, x: Float, y: Float): Boolean",
                        "external fun dispatchKeyboard(action: Int, keyCode: Int, scanCode: Int, text: String?): Boolean",
                        "external fun dispatchViewportMetrics(logicalWidth: Int, logicalHeight: Int, scale: Float): Boolean",
                    ],
                ),
                (
                    "platform/android/app/src/main/assets/zircon-host-resource-map.json",
                    [
                        "\"resourceStrategy\": \"mobile_asset_bundle\"",
                        "\"projectManifest\": \"zircon-project.toml\"",
                        "\"profile\": \"android-glue\"",
                        "\"platform\": \"android\"",
                        "\"nativeLibrary\": \"zircon_export_android_glue\"",
                    ],
                ),
            ],
        ),
        (
            ExportTargetPlatform::Ios,
            [
                (
                    "platform/ios/ZirconRuntimeHost/Linking/zircon_runtime_native.h",
                    [
                        "bool zircon_export_fetch_resource(const char *uri, uint32_t flags);",
                        "bool zircon_export_handle_viewport_metrics(uint32_t logical_width, uint32_t logical_height, float scale);",
                        "bool zircon_export_handle_keyboard(uint32_t action, uint32_t key_code, uint32_t scan_code, const uint8_t *text, size_t text_len);",
                        "bool zircon_export_handle_touch(uint64_t pointer_id, uint32_t phase, float x, float y);",
                        "bool zircon_export_handle_lifecycle(uint32_t state);",
                    ],
                ),
                (
                    "platform/ios/ZirconRuntimeHost/Resources/zircon-host-resource-map.json",
                    [
                        "\"resourceStrategy\": \"mobile_asset_bundle\"",
                        "\"projectManifest\": \"zircon-project.toml\"",
                        "\"profile\": \"ios-glue\"",
                        "\"platform\": \"ios\"",
                        "\"nativeLibrary\": \"zircon_export_ios_glue\"",
                    ],
                ),
            ],
        ),
    ];

    for (platform, expected_files) in cases {
        let profile_name = format!("{}-glue", platform.as_str());
        let mut manifest = ProjectManifest::new(
            "Platform Glue Test",
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

        for (path, expected_fragments) in expected_files {
            let generated = generated_file(&plan, path);
            for expected_fragment in expected_fragments {
                assert!(
                    generated.contains(expected_fragment),
                    "{platform:?} generated `{path}` should contain `{expected_fragment}`"
                );
            }
        }
    }
}

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

fn generated_file<'a>(plan: &'a ExportBuildPlan, path: &str) -> &'a str {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}

fn generated_file_purpose(plan: &ExportBuildPlan, path: &str) -> String {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.purpose.clone())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}

fn availability_contains(entries: &[RuntimePluginAvailabilityEntry], plugin_id: &str) -> bool {
    entries.iter().any(|entry| entry.id == plugin_id)
}

fn export_target_platform_from_ci_name(value: &str) -> ExportTargetPlatform {
    match value {
        "windows" => ExportTargetPlatform::Windows,
        "linux" => ExportTargetPlatform::Linux,
        "macos" => ExportTargetPlatform::Macos,
        "android" => ExportTargetPlatform::Android,
        "ios" => ExportTargetPlatform::Ios,
        "web_gpu" => ExportTargetPlatform::WebGpu,
        "wasm" => ExportTargetPlatform::Wasm,
        "headless" => ExportTargetPlatform::Headless,
        other => panic!("unknown export target platform {other}"),
    }
}
