use crate::{plugin::ExportProfile, RuntimeTargetMode};

use super::ExportGeneratedFile;

pub(super) fn platform_host_files(
    profile: &ExportProfile,
    has_native_dynamic_plugins: bool,
) -> Vec<ExportGeneratedFile> {
    let policy = profile.target_platform.policy();
    match policy.host_kind {
        crate::plugin::ExportPlatformHostKind::Desktop => vec![ExportGeneratedFile {
            path: "src/main.rs".to_string(),
            purpose: "generated desktop runtime entry point".to_string(),
            contents: super::main_template::main_template(profile, has_native_dynamic_plugins),
        }],
        crate::plugin::ExportPlatformHostKind::MobileApp => mobile_host_files(profile),
        crate::plugin::ExportPlatformHostKind::Browser => browser_host_files(profile),
    }
}

fn mobile_host_files(profile: &ExportProfile) -> Vec<ExportGeneratedFile> {
    match profile.target_platform {
        crate::plugin::ExportTargetPlatform::Android => vec![
            runtime_library_file(profile, "Android mobile asset host"),
            ExportGeneratedFile {
                path: "platform/android/settings.gradle.kts".to_string(),
                purpose: "Android Gradle settings manifest".to_string(),
                contents: android_settings_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/build.gradle.kts".to_string(),
                purpose: "Android Gradle root build manifest".to_string(),
                contents: android_root_gradle_template(),
            },
            ExportGeneratedFile {
                path: "platform/android/gradle.properties".to_string(),
                purpose: "Android Gradle packaging properties".to_string(),
                contents: android_gradle_properties_template(),
            },
            ExportGeneratedFile {
                path: "platform/android/app/build.gradle.kts".to_string(),
                purpose: "Android application packaging manifest".to_string(),
                contents: android_app_gradle_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/app/src/main/AndroidManifest.xml".to_string(),
                purpose: "Android application host manifest".to_string(),
                contents: android_manifest_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/app/src/main/java/dev/zircon/export/MainActivity.kt"
                    .to_string(),
                purpose: "Android Kotlin runtime host launcher".to_string(),
                contents: android_activity_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/app/src/main/res/values/strings.xml".to_string(),
                purpose: "Android application resource strings".to_string(),
                contents: android_strings_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/app/src/main/jniLibs/README.md".to_string(),
                purpose: "Android native library placement contract".to_string(),
                contents: android_jni_readme_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/package-export.ps1".to_string(),
                purpose: "Android release packaging script".to_string(),
                contents: android_package_script_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/signing.properties.example".to_string(),
                purpose: "Android signing configuration contract".to_string(),
                contents: android_signing_properties_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/play-publish.json".to_string(),
                purpose: "Android Play publishing metadata contract".to_string(),
                contents: android_play_publish_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/release-bundle.ps1".to_string(),
                purpose: "Android signed release bundle script".to_string(),
                contents: android_release_bundle_script_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/README.md".to_string(),
                purpose: "Android release packaging instructions".to_string(),
                contents: android_readme_template(profile),
            },
        ],
        crate::plugin::ExportTargetPlatform::Ios => vec![
            runtime_library_file(profile, "iOS mobile asset host"),
            ExportGeneratedFile {
                path: "platform/ios/Package.swift".to_string(),
                purpose: "iOS Swift package manifest".to_string(),
                contents: ios_package_swift_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/ios/ZirconRuntimeHost/Resources/Info.plist".to_string(),
                purpose: "iOS application host property list".to_string(),
                contents: ios_info_plist_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/ios/ZirconRuntimeHost/Sources/ZirconRuntimeHostApp.swift"
                    .to_string(),
                purpose: "iOS Swift runtime host launcher".to_string(),
                contents: ios_host_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/ios/ZirconRuntimeHost/Resources/zircon-export.bundle.toml"
                    .to_string(),
                purpose: "iOS bundled resource manifest pointer".to_string(),
                contents: ios_resource_pointer_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/ios/ZirconRuntimeHost/Linking/module.modulemap".to_string(),
                purpose: "iOS Rust static library module map".to_string(),
                contents: ios_module_map_template(),
            },
            ExportGeneratedFile {
                path: "platform/ios/ZirconRuntimeHost/Linking/zircon_runtime_native.h".to_string(),
                purpose: "iOS Rust static library C header".to_string(),
                contents: ios_native_header_template(),
            },
            ExportGeneratedFile {
                path: "platform/ios/package-export.ps1".to_string(),
                purpose: "iOS release packaging script".to_string(),
                contents: ios_package_script_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/ios/ExportOptions.plist".to_string(),
                purpose: "iOS signing and export options contract".to_string(),
                contents: ios_export_options_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/ios/app-store-connect.env.example".to_string(),
                purpose: "iOS App Store Connect credential contract".to_string(),
                contents: ios_app_store_connect_env_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/ios/archive-export.ps1".to_string(),
                purpose: "iOS archive and export script".to_string(),
                contents: ios_archive_export_script_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/ios/README.md".to_string(),
                purpose: "iOS release packaging instructions".to_string(),
                contents: ios_readme_template(profile),
            },
        ],
        _ => Vec::new(),
    }
}

fn browser_host_files(profile: &ExportProfile) -> Vec<ExportGeneratedFile> {
    let (host_name, script_name, script_contents, readme_title) = match profile.target_platform {
        crate::plugin::ExportTargetPlatform::WebGpu => (
            "webgpu",
            "src/zircon_webgpu_host.js",
            webgpu_host_script_template(profile),
            "WebGPU browser host",
        ),
        crate::plugin::ExportTargetPlatform::Wasm => (
            "wasm",
            "src/zircon_wasm_host.js",
            wasm_host_script_template(profile),
            "WASM browser host",
        ),
        _ => return Vec::new(),
    };
    vec![
        runtime_library_file(profile, readme_title),
        ExportGeneratedFile {
            path: format!("platform/{host_name}/index.html"),
            purpose: format!("{readme_title} HTML shell"),
            contents: browser_index_template(profile, script_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/{script_name}"),
            purpose: format!("{readme_title} JavaScript launcher"),
            contents: script_contents,
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/package.json"),
            purpose: format!("{readme_title} package manifest"),
            contents: browser_package_json_template(profile, host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/vite.config.mjs"),
            purpose: format!("{readme_title} dev and release server config"),
            contents: browser_vite_config_template(host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/public/zircon-export.manifest.json"),
            purpose: format!("{readme_title} fetch manifest"),
            contents: browser_fetch_manifest_template(profile, host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/public/_headers"),
            purpose: format!("{readme_title} CDN cache headers"),
            contents: browser_cdn_headers_template(),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/public/zircon-export.cdn-manifest.json"),
            purpose: format!("{readme_title} CDN deployment manifest"),
            contents: browser_cdn_manifest_template(profile, host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/package-export.mjs"),
            purpose: format!("{readme_title} release packaging script"),
            contents: browser_package_script_template(host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/deploy-cdn.mjs"),
            purpose: format!("{readme_title} CDN deployment contract"),
            contents: browser_deploy_cdn_script_template(host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/README.md"),
            purpose: format!("{readme_title} release packaging instructions"),
            contents: browser_readme_template(profile, readme_title),
        },
    ]
}

fn runtime_library_file(profile: &ExportProfile, host_label: &str) -> ExportGeneratedFile {
    ExportGeneratedFile {
        path: "src/lib.rs".to_string(),
        purpose: format!("generated {host_label} runtime library entry point"),
        contents: runtime_library_template(profile, host_label),
    }
}

fn runtime_library_template(profile: &ExportProfile, host_label: &str) -> String {
    let entry_profile = entry_profile(profile.target_mode);
    let target_platform = profile.target_platform.as_str();
    format!(
        "mod zircon_plugins;\n\nuse zircon_app::{{EntryConfig, EntryProfile, EntryRunner}};\n\n/// Starts the Zircon runtime from a generated {host_label} scaffold.\npub fn zircon_export_bootstrap() -> Result<(), Box<dyn std::error::Error>> {{\n    let config = EntryConfig::new(EntryProfile::{entry_profile})\n        .with_target_mode(zircon_plugins::target_mode())\n        .with_project_plugins(zircon_plugins::project_plugins())\n        .with_export_profile(zircon_plugins::export_profile());\n    let _core = EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations(\n        config,\n        zircon_plugins::runtime_plugin_registrations(),\n        zircon_plugins::runtime_plugin_feature_registrations(),\n    )?;\n    Ok(())\n}}\n\n#[no_mangle]\npub extern \"C\" fn zircon_export_start() -> bool {{\n    zircon_export_bootstrap().is_ok()\n}}\n\n#[cfg(target_os = \"android\")]\n#[no_mangle]\npub extern \"system\" fn Java_dev_zircon_export_ZirconRuntime_start(\n    _env: *mut core::ffi::c_void,\n    _class: *mut core::ffi::c_void,\n) -> bool {{\n    zircon_export_start()\n}}\n\npub const ZIRCON_EXPORT_TARGET_PLATFORM: &str = \"{target_platform}\";\n"
    )
}

fn android_manifest_template(_profile: &ExportProfile) -> String {
    "<manifest xmlns:android=\"http://schemas.android.com/apk/res/android\">\n    <application android:label=\"@string/app_name\" android:hasCode=\"true\" android:extractNativeLibs=\"true\">\n        <activity android:name=\".MainActivity\" android:exported=\"true\">\n            <intent-filter>\n                <action android:name=\"android.intent.action.MAIN\" />\n                <category android:name=\"android.intent.category.LAUNCHER\" />\n            </intent-filter>\n        </activity>\n    </application>\n</manifest>\n"
        .to_string()
}

fn android_settings_template(profile: &ExportProfile) -> String {
    format!(
        "pluginManagement {{\n    repositories {{\n        google()\n        mavenCentral()\n        gradlePluginPortal()\n    }}\n}}\ndependencyResolutionManagement {{ repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS); repositories {{ google(); mavenCentral() }} }}\nrootProject.name = \"{}\"\ninclude(\":app\")\n",
        gradle_string_escape(&profile.output_name)
    )
}

fn android_root_gradle_template() -> String {
    "plugins {\n    id(\"com.android.application\") version \"8.6.1\" apply false\n    id(\"org.jetbrains.kotlin.android\") version \"2.0.20\" apply false\n}\n"
        .to_string()
}

fn android_gradle_properties_template() -> String {
    "android.useAndroidX=true\nandroid.nonTransitiveRClass=true\nkotlin.code.style=official\n"
        .to_string()
}

fn android_app_gradle_template(profile: &ExportProfile) -> String {
    format!(
        "plugins {{\n    id(\"com.android.application\")\n    id(\"org.jetbrains.kotlin.android\")\n}}\n\nandroid {{\n    namespace = \"dev.zircon.export\"\n    compileSdk = 35\n\n    defaultConfig {{\n        applicationId = \"dev.zircon.export.{}\"\n        minSdk = 28\n        targetSdk = 35\n        versionCode = 1\n        versionName = \"0.1.0\"\n    }}\n\n    sourceSets[\"main\"].assets.srcDirs(\"../../../assets\")\n    sourceSets[\"main\"].jniLibs.srcDirs(\"src/main/jniLibs\")\n}}\n",
        android_identifier_suffix(&profile.output_name)
    )
}

fn android_strings_template(profile: &ExportProfile) -> String {
    format!(
        "<resources>\n    <string name=\"app_name\">{}</string>\n</resources>\n",
        xml_escape(&profile.output_name)
    )
}

fn android_activity_template(profile: &ExportProfile) -> String {
    format!(
        "package dev.zircon.export\n\nimport android.app.Activity\nimport android.os.Bundle\n\nclass MainActivity : Activity() {{\n    override fun onCreate(savedInstanceState: Bundle?) {{\n        super.onCreate(savedInstanceState)\n        System.loadLibrary(\"zircon_export_{}\")\n        ZirconRuntime.start()\n    }}\n}}\n\nobject ZirconRuntime {{\n    external fun start(): Boolean\n}}\n",
        native_library_stem(&profile.output_name)
    )
}

fn android_readme_template(profile: &ExportProfile) -> String {
    format!(
        "# Android Export Host\n\nProfile `{}` targets Android through a Gradle app scaffold, a mobile asset bundle, and static or VM plugin packaging. Build the generated Rust `cdylib` for each Android ABI, copy each `libzircon_export_*.so` under `platform/android/app/src/main/jniLibs/<abi>/`, then run `platform/android/package-export.ps1` or `./gradlew assembleRelease` from `platform/android`. The Gradle app packages `assets/zircon-project.toml` through its `main.assets` source set and launches `zircon_export_start` from `MainActivity`.\n",
        profile.name
    )
}

fn android_jni_readme_template(profile: &ExportProfile) -> String {
    format!(
        "# Android Native Libraries\n\nPlace compiled libraries named `libzircon_export_{}.so` under ABI folders such as `arm64-v8a/` and `x86_64/`. The generated Gradle manifest includes this directory as `jniLibs`, so release packaging embeds the Rust runtime library beside the mobile asset bundle.\n",
        native_library_stem(&profile.output_name)
    )
}

fn android_package_script_template(profile: &ExportProfile) -> String {
    format!(
        "$ErrorActionPreference = 'Stop'\nPush-Location $PSScriptRoot\ntry {{\n    if (Test-Path ./gradlew) {{ ./gradlew assembleRelease }} else {{ gradle assembleRelease }}\n    Write-Host 'Android export package ready for profile {} at app/build/outputs/apk/release'\n}} finally {{\n    Pop-Location\n}}\n",
        powershell_string_escape(&profile.name)
    )
}

fn android_signing_properties_template(profile: &ExportProfile) -> String {
    format!(
        "# Copy this file to signing.properties and fill values from your release secret store.\nprofile={}\nstoreFile=${{ZR_ANDROID_KEYSTORE_PATH}}\nstorePassword=${{ZR_ANDROID_KEYSTORE_PASSWORD}}\nkeyAlias=${{ZR_ANDROID_KEY_ALIAS}}\nkeyPassword=${{ZR_ANDROID_KEY_PASSWORD}}\n",
        properties_string_escape(&profile.name)
    )
}

fn android_play_publish_template(profile: &ExportProfile) -> String {
    format!(
        "{{\n  \"profile\": \"{}\",\n  \"track\": \"internal\",\n  \"packageName\": \"dev.zircon.export.{}\",\n  \"serviceAccountJson\": \"${{ZR_GOOGLE_PLAY_SERVICE_ACCOUNT_JSON}}\",\n  \"artifact\": \"app/build/outputs/bundle/release/app-release.aab\"\n}}\n",
        json_string_escape(&profile.name),
        json_string_escape(&android_identifier_suffix(&profile.output_name))
    )
}

fn android_release_bundle_script_template(profile: &ExportProfile) -> String {
    format!(
        "$ErrorActionPreference = 'Stop'\nPush-Location $PSScriptRoot\ntry {{\n    if (-not $env:ZR_ANDROID_KEYSTORE_PATH) {{ throw 'ZR_ANDROID_KEYSTORE_PATH is required for signed Android release bundles' }}\n    if (Test-Path ./gradlew) {{ ./gradlew bundleRelease }} else {{ gradle bundleRelease }}\n    Write-Host 'Android signed release bundle ready for profile {} at app/build/outputs/bundle/release/app-release.aab'\n}} finally {{\n    Pop-Location\n}}\n",
        powershell_string_escape(&profile.name)
    )
}

fn ios_package_swift_template(profile: &ExportProfile) -> String {
    format!(
        "// swift-tools-version: 5.10\nimport PackageDescription\n\nlet package = Package(\n    name: \"{}\",\n    platforms: [.iOS(.v16)],\n    products: [\n        .executable(name: \"ZirconRuntimeHost\", targets: [\"ZirconRuntimeHost\"]),\n    ],\n    targets: [\n        .executableTarget(\n            name: \"ZirconRuntimeHost\",\n            resources: [.process(\"Resources\")],\n            linkerSettings: [.unsafeFlags([\"-L./ZirconRuntimeHost/Linking\", \"-lzircon_export_{}\"])]\n        ),\n    ]\n)\n",
        swift_string_escape(&profile.output_name),
        native_library_stem(&profile.output_name)
    )
}

fn ios_info_plist_template(profile: &ExportProfile) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n    <key>CFBundleDisplayName</key>\n    <string>{}</string>\n    <key>CFBundleIdentifier</key>\n    <string>dev.zircon.export.{}</string>\n</dict>\n</plist>\n",
        xml_escape(&profile.output_name),
        bundle_identifier_suffix(&profile.output_name)
    )
}

fn ios_host_template(profile: &ExportProfile) -> String {
    format!(
        "import SwiftUI\n\n@_silgen_name(\"zircon_export_start\")\nfunc zircon_export_start() -> Bool\n\n@main\nstruct ZirconRuntimeHostApp: App {{\n    init() {{\n        _ = zircon_export_start()\n    }}\n\n    var body: some Scene {{\n        WindowGroup {{\n            Text(\"{}\")\n        }}\n    }}\n}}\n",
        swift_string_escape(&profile.output_name)
    )
}

fn ios_readme_template(profile: &ExportProfile) -> String {
    format!(
        "# iOS Export Host\n\nProfile `{}` targets iOS through a Swift Package host, bundled resources, and static or VM plugin packaging. Build the generated Rust library as `libzircon_export_{}.a` for the desired iOS architectures, place it under `platform/ios/ZirconRuntimeHost/Linking/`, copy `assets/zircon-project.toml` into `ZirconRuntimeHost/Resources/`, then run `platform/ios/package-export.ps1` to build the Swift package.\n",
        profile.name,
        native_library_stem(&profile.output_name)
    )
}

fn ios_resource_pointer_template(profile: &ExportProfile) -> String {
    format!(
        "profile = \"{}\"\nproject_manifest = \"zircon-project.toml\"\nresource_strategy = \"mobile_asset_bundle\"\n",
        toml_string_escape(&profile.name)
    )
}

fn ios_module_map_template() -> String {
    "module ZirconRuntimeNative {\n    header \"zircon_runtime_native.h\"\n    export *\n}\n"
        .to_string()
}

fn ios_native_header_template() -> String {
    "#pragma once\n#include <stdbool.h>\n\nbool zircon_export_start(void);\n".to_string()
}

fn ios_package_script_template(profile: &ExportProfile) -> String {
    format!(
        "$ErrorActionPreference = 'Stop'\nPush-Location $PSScriptRoot\ntry {{\n    swift build -c release\n    Write-Host 'iOS Swift package built for profile {}'\n}} finally {{\n    Pop-Location\n}}\n",
        powershell_string_escape(&profile.name)
    )
}

fn ios_export_options_template(profile: &ExportProfile) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n    <key>method</key>\n    <string>app-store-connect</string>\n    <key>teamID</key>\n    <string>$(ZR_IOS_TEAM_ID)</string>\n    <key>signingStyle</key>\n    <string>manual</string>\n    <key>provisioningProfiles</key>\n    <dict>\n        <key>dev.zircon.export.{}</key>\n        <string>$(ZR_IOS_PROVISIONING_PROFILE)</string>\n    </dict>\n</dict>\n</plist>\n",
        xml_escape(&bundle_identifier_suffix(&profile.output_name))
    )
}

fn ios_app_store_connect_env_template(profile: &ExportProfile) -> String {
    format!(
        "# Copy this file to app-store-connect.env and load it from your CI secret store.\nZR_IOS_PROFILE_NAME={}\nZR_IOS_TEAM_ID=\nZR_IOS_PROVISIONING_PROFILE=\nZR_APP_STORE_CONNECT_API_KEY_ID=\nZR_APP_STORE_CONNECT_ISSUER_ID=\nZR_APP_STORE_CONNECT_PRIVATE_KEY_PATH=\n",
        properties_string_escape(&profile.name)
    )
}

fn ios_archive_export_script_template(profile: &ExportProfile) -> String {
    format!(
        "$ErrorActionPreference = 'Stop'\nPush-Location $PSScriptRoot\ntry {{\n    if (-not $env:ZR_IOS_TEAM_ID) {{ throw 'ZR_IOS_TEAM_ID is required for iOS archive export' }}\n    xcodebuild -scheme ZirconRuntimeHost -configuration Release -archivePath ./build/ZirconRuntimeHost.xcarchive archive\n    xcodebuild -exportArchive -archivePath ./build/ZirconRuntimeHost.xcarchive -exportOptionsPlist ./ExportOptions.plist -exportPath ./build/export\n    Write-Host 'iOS archive exported for profile {} at build/export'\n}} finally {{\n    Pop-Location\n}}\n",
        powershell_string_escape(&profile.name)
    )
}

fn browser_package_json_template(profile: &ExportProfile, host_name: &str) -> String {
    format!(
        "{{\n  \"name\": \"zircon-export-{}-{}\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {{\n    \"dev\": \"vite --host 127.0.0.1\",\n    \"build\": \"vite build\",\n    \"preview\": \"vite preview --host 127.0.0.1\",\n    \"package:export\": \"node package-export.mjs\"\n  }},\n  \"devDependencies\": {{\n    \"@vitejs/plugin-basic-ssl\": \"latest\",\n    \"vite\": \"latest\"\n  }}\n}}\n",
        json_string_escape(host_name),
        json_string_escape(&native_library_stem(&profile.output_name))
    )
}

fn browser_vite_config_template(host_name: &str) -> String {
    format!(
        "import {{ defineConfig }} from 'vite';\n\nexport default defineConfig({{\n  base: './',\n  publicDir: 'public',\n  build: {{\n    outDir: 'dist/{}',\n    emptyOutDir: true,\n    target: 'es2022',\n    assetsInlineLimit: 0\n  }},\n  server: {{\n    headers: {{\n      'Cross-Origin-Opener-Policy': 'same-origin',\n      'Cross-Origin-Embedder-Policy': 'require-corp'\n    }}\n  }}\n}});\n",
        javascript_string_escape(host_name)
    )
}

fn browser_fetch_manifest_template(profile: &ExportProfile, host_name: &str) -> String {
    format!(
        "{{\n  \"profile\": \"{}\",\n  \"target\": \"{}\",\n  \"resourceStrategy\": \"browser_fetch\",\n  \"projectManifest\": \"../../assets/zircon-project.toml\",\n  \"wasmModule\": \"./zircon_export_{}.wasm\"\n}}\n",
        json_string_escape(&profile.name),
        json_string_escape(host_name),
        json_string_escape(&native_library_stem(&profile.output_name))
    )
}

fn browser_cdn_headers_template() -> String {
    "/*\n  Cross-Origin-Opener-Policy: same-origin\n  Cross-Origin-Embedder-Policy: require-corp\n  Cache-Control: public, max-age=300\n/assets/*\n  Cache-Control: public, max-age=31536000, immutable\n/*.wasm\n  Cache-Control: public, max-age=31536000, immutable\n  Content-Type: application/wasm\n"
        .to_string()
}

fn browser_cdn_manifest_template(profile: &ExportProfile, host_name: &str) -> String {
    format!(
        "{{\n  \"profile\": \"{}\",\n  \"target\": \"{}\",\n  \"baseUrl\": \"${{ZR_CDN_BASE_URL}}\",\n  \"immutableAssetPath\": \"assets/\",\n  \"compression\": [\"br\", \"gzip\"],\n  \"assetIntegrity\": \"sha256 manifest generated by CI before publish\"\n}}\n",
        json_string_escape(&profile.name),
        json_string_escape(host_name)
    )
}

fn browser_package_script_template(host_name: &str) -> String {
    format!(
        "import {{ mkdir, copyFile }} from 'node:fs/promises';\nimport {{ dirname, join }} from 'node:path';\n\nconst output = join('dist', '{}');\nawait mkdir(join(output, 'assets'), {{ recursive: true }});\nawait copyFile('../../assets/zircon-project.toml', join(output, 'assets', 'zircon-project.toml'));\nconsole.log(`Browser export assets staged in ${{output}}`);\n",
        javascript_string_escape(host_name)
    )
}

fn browser_deploy_cdn_script_template(host_name: &str) -> String {
    format!(
        "import {{ access }} from 'node:fs/promises';\nimport {{ join }} from 'node:path';\n\nconst baseUrl = process.env.ZR_CDN_BASE_URL;\nif (!baseUrl) {{\n  throw new Error('ZR_CDN_BASE_URL is required before publishing the {} export');\n}}\nconst output = join('dist', '{}');\nawait access(output);\nconsole.log(`CDN publish contract ready for ${{output}} -> ${{baseUrl}}`);\nconsole.log('Upload with immutable cache headers from public/_headers and update zircon-export.cdn-manifest.json with final hashes.');\n",
        javascript_string_escape(host_name),
        javascript_string_escape(host_name)
    )
}

fn browser_readme_template(profile: &ExportProfile, title: &str) -> String {
    format!(
        "# {title}\n\nProfile `{}` targets browser resources through fetch and static or VM plugin packaging. Run `npm install`, `npm run build`, and `npm run package:export` from this folder after compiling the Rust `cdylib` to `zircon_export_{}.wasm`. The generated `public/zircon-export.manifest.json` records the fetch contract, and the Vite config keeps COOP/COEP headers enabled for WebGPU and threaded WASM hosts.\n",
        profile.name,
        native_library_stem(&profile.output_name)
    )
}

fn browser_index_template(profile: &ExportProfile, script_name: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n    <meta charset=\"utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    <title>{}</title>\n</head>\n<body>\n    <canvas id=\"zircon-canvas\"></canvas>\n    <script type=\"module\" src=\"./{}\"></script>\n</body>\n</html>\n",
        html_escape(&profile.output_name),
        script_name
    )
}

fn webgpu_host_script_template(profile: &ExportProfile) -> String {
    format!(
        "const canvas = document.querySelector('#zircon-canvas');\nconst manifest = await fetch('./zircon-export.manifest.json').then((response) => response.json());\nif (!navigator.gpu) {{\n    throw new Error('WebGPU is unavailable for Zircon export profile {}');\n}}\nconst adapter = await navigator.gpu.requestAdapter();\nif (!adapter) {{\n    throw new Error('WebGPU adapter is unavailable for Zircon export profile {}');\n}}\nwindow.zirconExportHost = {{\n    target: 'web_gpu',\n    canvas,\n    adapter,\n    manifest,\n    resourceManifest: './assets/zircon-project.toml',\n}};\n",
        javascript_string_escape(&profile.name),
        javascript_string_escape(&profile.name)
    )
}

fn wasm_host_script_template(_profile: &ExportProfile) -> String {
    "const canvas = document.querySelector('#zircon-canvas');\nconst manifest = await fetch('./zircon-export.manifest.json').then((response) => response.json());\nconst wasmModule = await WebAssembly.compileStreaming(fetch(manifest.wasmModule));\nwindow.zirconExportHost = {\n    target: 'wasm',\n    canvas,\n    manifest,\n    wasmModule,\n    resourceManifest: './assets/zircon-project.toml',\n};\n"
        .to_string()
}

fn entry_profile(target_mode: RuntimeTargetMode) -> &'static str {
    match target_mode {
        RuntimeTargetMode::ClientRuntime => "Runtime",
        RuntimeTargetMode::ServerRuntime => "Headless",
        RuntimeTargetMode::EditorHost => "Editor",
    }
}

fn native_library_stem(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' | '_' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' => '_',
            _ => '_',
        })
        .collect()
}

fn bundle_identifier_suffix(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' | '_' => '.',
            _ => '-',
        })
        .collect()
}

fn android_identifier_suffix(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            '-' | '_' => '.',
            _ => '.',
        })
        .collect::<String>()
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(".")
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html_escape(value: &str) -> String {
    xml_escape(value)
}

fn swift_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn javascript_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn json_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn gradle_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn powershell_string_escape(value: &str) -> String {
    value.replace('`', "``").replace('\'', "''")
}

fn properties_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n")
}

fn toml_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
