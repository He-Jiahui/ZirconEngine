use crate::plugin::{ExportProfile, ExportTargetPlatform};

use super::super::ExportGeneratedFile;
use super::{
    android_identifier_suffix, bundle_identifier_suffix, gradle_string_escape, json_string_escape,
    native_library_stem, powershell_string_escape, properties_string_escape, runtime_library_file,
    swift_string_escape, toml_string_escape, xml_escape,
};

pub(super) fn mobile_host_files(profile: &ExportProfile) -> Vec<ExportGeneratedFile> {
    match profile.target_platform {
        ExportTargetPlatform::Android => vec![
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
                path: "platform/android/app/src/main/java/dev/zircon/export/ZirconRuntime.kt"
                    .to_string(),
                purpose: "Android JNI runtime binding declarations".to_string(),
                contents: android_runtime_binding_template(),
            },
            ExportGeneratedFile {
                path: "platform/android/app/src/main/res/values/strings.xml".to_string(),
                purpose: "Android application resource strings".to_string(),
                contents: android_strings_template(profile),
            },
            ExportGeneratedFile {
                path: "platform/android/app/src/main/assets/zircon-host-resource-map.json"
                    .to_string(),
                purpose: "Android mobile asset resource map".to_string(),
                contents: mobile_resource_map_template(profile, "android"),
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
        ExportTargetPlatform::Ios => vec![
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
                path: "platform/ios/ZirconRuntimeHost/Resources/zircon-host-resource-map.json"
                    .to_string(),
                purpose: "iOS mobile asset resource map".to_string(),
                contents: mobile_resource_map_template(profile, "ios"),
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

fn android_manifest_template(profile: &ExportProfile) -> String {
    format!(
        "<manifest xmlns:android=\"http://schemas.android.com/apk/res/android\">\n    <application android:label=\"@string/app_name\" android:hasCode=\"true\" android:extractNativeLibs=\"true\">\n        <meta-data android:name=\"dev.zircon.export.PROFILE\" android:value=\"{}\" />\n        <activity android:name=\".MainActivity\" android:exported=\"true\">\n            <intent-filter>\n                <action android:name=\"android.intent.action.MAIN\" />\n                <category android:name=\"android.intent.category.LAUNCHER\" />\n            </intent-filter>\n        </activity>\n    </application>\n</manifest>\n",
        xml_escape(&profile.name)
    )
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
        "plugins {{\n    id(\"com.android.application\")\n    id(\"org.jetbrains.kotlin.android\")\n}}\n\nandroid {{\n    namespace = \"dev.zircon.export\"\n    compileSdk = 35\n\n    defaultConfig {{\n        applicationId = \"dev.zircon.export.{}\"\n        minSdk = 28\n        targetSdk = 35\n        versionCode = 1\n        versionName = \"0.1.0\"\n    }}\n\n    sourceSets[\"main\"].assets.srcDirs(\"src/main/assets\", \"../../../assets\")\n    sourceSets[\"main\"].jniLibs.srcDirs(\"src/main/jniLibs\")\n}}\n",
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
        "package dev.zircon.export\n\nimport android.app.Activity\nimport android.os.Bundle\nimport android.view.KeyEvent\nimport android.view.MotionEvent\nimport android.view.View\n\nprivate const val ZIRCON_LIFECYCLE_FOREGROUND = 1\nprivate const val ZIRCON_LIFECYCLE_BACKGROUND = 2\nprivate const val ZIRCON_LIFECYCLE_RESUMED = 4\nprivate const val ZIRCON_TOUCH_STARTED = 1\nprivate const val ZIRCON_TOUCH_MOVED = 2\nprivate const val ZIRCON_TOUCH_ENDED = 3\nprivate const val ZIRCON_TOUCH_CANCELLED = 4\nprivate const val ZIRCON_KEY_PRESSED = 1\nprivate const val ZIRCON_KEY_RELEASED = 2\n\nclass MainActivity : Activity() {{\n    override fun onCreate(savedInstanceState: Bundle?) {{\n        super.onCreate(savedInstanceState)\n        System.loadLibrary(\"zircon_export_{}\")\n        ZirconRuntime.start()\n        window.decorView.setOnTouchListener {{ _: View, event: MotionEvent ->\n            forwardTouch(event)\n            true\n        }}\n        window.decorView.addOnLayoutChangeListener {{ view, _, _, _, _, _, _, _, _ ->\n            val width = view.width\n            val height = view.height\n            ZirconRuntime.dispatchViewportMetrics(width, height, resources.displayMetrics.density)\n        }}\n    }}\n\n    override fun onResume() {{\n        super.onResume()\n        ZirconRuntime.dispatchLifecycle(ZIRCON_LIFECYCLE_RESUMED)\n    }}\n\n    override fun onStart() {{\n        super.onStart()\n        ZirconRuntime.dispatchLifecycle(ZIRCON_LIFECYCLE_FOREGROUND)\n    }}\n\n    override fun onStop() {{\n        ZirconRuntime.dispatchLifecycle(ZIRCON_LIFECYCLE_BACKGROUND)\n        super.onStop()\n    }}\n\n    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {{\n        ZirconRuntime.dispatchKeyboard(ZIRCON_KEY_PRESSED, event.keyCode, event.scanCode, null)\n        return super.onKeyDown(keyCode, event)\n    }}\n\n    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {{\n        ZirconRuntime.dispatchKeyboard(ZIRCON_KEY_RELEASED, event.keyCode, event.scanCode, null)\n        return super.onKeyUp(keyCode, event)\n    }}\n\n    private fun forwardTouch(event: MotionEvent) {{\n        val phase = when (event.actionMasked) {{\n            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> ZIRCON_TOUCH_STARTED\n            MotionEvent.ACTION_MOVE -> ZIRCON_TOUCH_MOVED\n            MotionEvent.ACTION_UP, MotionEvent.ACTION_POINTER_UP -> ZIRCON_TOUCH_ENDED\n            MotionEvent.ACTION_CANCEL -> ZIRCON_TOUCH_CANCELLED\n            else -> return\n        }}\n        for (index in 0 until event.pointerCount) {{\n            ZirconRuntime.dispatchTouch(event.getPointerId(index).toLong(), phase, event.getX(index), event.getY(index))\n        }}\n    }}\n}}\n\nobject ZirconRuntime {{\n    external fun start(): Boolean\n    external fun dispatchLifecycle(state: Int): Boolean\n    external fun dispatchTouch(pointerId: Long, phase: Int, x: Float, y: Float): Boolean\n    external fun dispatchKeyboard(action: Int, keyCode: Int, scanCode: Int, text: String?): Boolean\n    external fun dispatchViewportMetrics(width: Int, height: Int, scale: Float): Boolean\n}}\n",
        native_library_stem(&profile.output_name)
    )
}

fn android_runtime_binding_template() -> String {
    "package dev.zircon.export\n\nobject ZirconRuntime {\n    external fun start(): Boolean\n    external fun dispatchLifecycle(state: Int): Boolean\n    external fun dispatchTouch(pointerId: Long, phase: Int, x: Float, y: Float): Boolean\n    external fun dispatchKeyboard(action: Int, keyCode: Int, scanCode: Int, text: String?): Boolean\n    external fun dispatchViewportMetrics(logicalWidth: Int, logicalHeight: Int, scale: Float): Boolean\n}\n"
        .to_string()
}

fn mobile_resource_map_template(profile: &ExportProfile, platform: &str) -> String {
    format!(
        "{{\n  \"profile\": \"{}\",\n  \"platform\": \"{}\",\n  \"resourceStrategy\": \"mobile_asset_bundle\",\n  \"projectManifest\": \"zircon-project.toml\",\n  \"nativeLibrary\": \"zircon_export_{}\"\n}}\n",
        json_string_escape(&profile.name),
        json_string_escape(platform),
        json_string_escape(&native_library_stem(&profile.output_name))
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
        "$ErrorActionPreference = 'Stop'\nPush-Location $PSScriptRoot\ntry {{\n    if (-not $env:ZR_ANDROID_KEYSTORE_PATH) {{ throw 'ZR_ANDROID_KEYSTORE_PATH is required for signed Android release bundles' }}\n    if (-not $env:ZR_GOOGLE_PLAY_SERVICE_ACCOUNT_JSON) {{ throw 'ZR_GOOGLE_PLAY_SERVICE_ACCOUNT_JSON is required for Play upload' }}\n    if (-not $env:ZR_GOOGLE_PLAY_PACKAGE_NAME) {{ throw 'ZR_GOOGLE_PLAY_PACKAGE_NAME is required for Play upload' }}\n    if (Test-Path ./gradlew) {{ ./gradlew bundleRelease }} else {{ gradle bundleRelease }}\n    $artifact = 'app/build/outputs/bundle/release/app-release.aab'\n    if (-not (Test-Path $artifact)) {{ throw \"Android bundle was not produced at $artifact\" }}\n    $packageName = $env:ZR_GOOGLE_PLAY_PACKAGE_NAME\n    $editUrl = \"https://androidpublisher.googleapis.com/androidpublisher/v3/applications/$packageName/edits\"\n    Write-Host \"Creating Google Play edit through $editUrl\"\n    Invoke-RestMethod -Method Post -Uri $editUrl -Headers @{{ Authorization = \"Bearer $env:ZR_GOOGLE_PLAY_ACCESS_TOKEN\" }} | Out-Null\n    Write-Host 'Android signed release bundle ready for profile {} at app/build/outputs/bundle/release/app-release.aab'\n}} finally {{\n    Pop-Location\n}}\n",
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
        "import SwiftUI\nimport UIKit\n\nlet ZIRCON_LIFECYCLE_RESUMED: UInt32 = 4\nlet ZIRCON_TOUCH_MOVED: UInt32 = 2\nlet ZIRCON_KEY_TEXT: UInt32 = 3\n\n@_silgen_name(\"zircon_export_start\")\nfunc zircon_export_start() -> Bool\n@_silgen_name(\"zircon_export_handle_lifecycle\")\nfunc zircon_export_handle_lifecycle(_ state: UInt32) -> Bool\n@_silgen_name(\"zircon_export_handle_touch\")\nfunc zircon_export_handle_touch(_ pointerId: UInt64, _ phase: UInt32, _ x: Float, _ y: Float) -> Bool\n@_silgen_name(\"zircon_export_handle_keyboard\")\nfunc zircon_export_handle_keyboard(_ action: UInt32, _ keyCode: UInt32, _ scanCode: UInt32, _ text: UnsafePointer<UInt8>?, _ textLen: Int) -> Bool\n@_silgen_name(\"zircon_export_handle_viewport_metrics\")\nfunc zircon_export_handle_viewport_metrics(_ logicalWidth: UInt32, _ logicalHeight: UInt32, _ scale: Float) -> Bool\n\nstruct ZirconRuntimeView: UIViewRepresentable {{\n    func makeUIView(context: Context) -> ZirconTouchView {{ ZirconTouchView() }}\n    func updateUIView(_ uiView: ZirconTouchView, context: Context) {{ }}\n}}\n\nfinal class ZirconTouchView: UIView {{\n    override func layoutSubviews() {{\n        super.layoutSubviews()\n        let size = bounds.size\n        let scale = window?.screen.scale ?? UIScreen.main.scale\n        _ = zircon_export_handle_viewport_metrics(UInt32(size.width), UInt32(size.height), Float(scale))\n    }}\n\n    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {{\n        for touch in touches {{\n            let point = touch.location(in: self)\n            _ = zircon_export_handle_touch(UInt64(touch.hash), ZIRCON_TOUCH_MOVED, Float(point.x), Float(point.y))\n        }}\n    }}\n}}\n\n@main\nstruct ZirconRuntimeHostApp: App {{\n    init() {{\n        _ = zircon_export_start()\n        _ = zircon_export_handle_lifecycle(ZIRCON_LIFECYCLE_RESUMED)\n        let text = Array(\"{}\".utf8)\n        text.withUnsafeBufferPointer {{ buffer in\n            _ = zircon_export_handle_keyboard(ZIRCON_KEY_TEXT, 0, 0, buffer.baseAddress, buffer.count)\n        }}\n    }}\n\n    var body: some Scene {{\n        WindowGroup {{\n            ZirconRuntimeView()\n        }}\n    }}\n}}\n",
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
    "#pragma once\n#include <stdbool.h>\n#include <stddef.h>\n#include <stdint.h>\n\nbool zircon_export_start(void);\nbool zircon_export_handle_lifecycle(uint32_t state);\nbool zircon_export_handle_touch(uint64_t pointer_id, uint32_t phase, float x, float y);\nbool zircon_export_handle_keyboard(uint32_t action, uint32_t key_code, uint32_t scan_code, const uint8_t *text, size_t text_len);\nbool zircon_export_handle_viewport_metrics(uint32_t logical_width, uint32_t logical_height, float scale);\nbool zircon_export_fetch_resource(const char *uri, uint32_t flags);\n".to_string()
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
        "$ErrorActionPreference = 'Stop'\nPush-Location $PSScriptRoot\ntry {{\n    if (-not $env:ZR_IOS_TEAM_ID) {{ throw 'ZR_IOS_TEAM_ID is required for iOS archive export' }}\n    if (-not $env:ZR_APP_STORE_CONNECT_PRIVATE_KEY_PATH) {{ throw 'ZR_APP_STORE_CONNECT_PRIVATE_KEY_PATH is required for App Store Connect upload' }}\n    if (-not $env:ZR_APP_STORE_CONNECT_API_KEY_ID) {{ throw 'ZR_APP_STORE_CONNECT_API_KEY_ID is required for App Store Connect upload' }}\n    if (-not $env:ZR_APP_STORE_CONNECT_ISSUER_ID) {{ throw 'ZR_APP_STORE_CONNECT_ISSUER_ID is required for App Store Connect upload' }}\n    xcodebuild -scheme ZirconRuntimeHost -configuration Release -archivePath ./build/ZirconRuntimeHost.xcarchive archive\n    xcodebuild -exportArchive -archivePath ./build/ZirconRuntimeHost.xcarchive -exportOptionsPlist ./ExportOptions.plist -exportPath ./build/export\n    $ipa = Get-ChildItem ./build/export -Filter *.ipa | Select-Object -First 1\n    if (-not $ipa) {{ throw 'No exported .ipa was produced under build/export' }}\n    xcrun altool --upload-app --type ios --file $ipa.FullName --apiKey $env:ZR_APP_STORE_CONNECT_API_KEY_ID --apiIssuer $env:ZR_APP_STORE_CONNECT_ISSUER_ID\n    Write-Host 'iOS archive exported and upload requested for profile {} at build/export'\n}} finally {{\n    Pop-Location\n}}\n",
        powershell_string_escape(&profile.name)
    )
}
