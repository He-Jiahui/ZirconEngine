use crate::core::framework::project::{ExportProfile, ExportTargetPlatform};

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
    let mut source = String::with_capacity(9_216);
    source.push_str(
        r#"package dev.zircon.export

import android.app.Activity
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View

private const val ZIRCON_LIFECYCLE_FOREGROUND = 1
private const val ZIRCON_LIFECYCLE_BACKGROUND = 2
private const val ZIRCON_LIFECYCLE_RESUMED = 4
private const val ZIRCON_LIFECYCLE_SUSPENDED = 8
private const val ZIRCON_TOUCH_STARTED = 1
private const val ZIRCON_TOUCH_MOVED = 2
private const val ZIRCON_TOUCH_ENDED = 3
private const val ZIRCON_TOUCH_CANCELLED = 4
private const val ZIRCON_KEY_PRESSED = 1
private const val ZIRCON_KEY_RELEASED = 2

private data class PendingTouchMove(
    val pointerId: Long,
    val x: Float,
    val y: Float,
    val deltaX: Float,
    val deltaY: Float,
    val queuedAtNanos: Long,
)

private data class PendingViewportMetrics(
    val width: Int,
    val height: Int,
    val scale: Float,
    val queuedAtNanos: Long,
)

class MainActivity : Activity() {
    private lateinit var frameView: View
    private val pendingTouchMoves = linkedMapOf<Long, PendingTouchMove>()
    private val lastTouchPositions = linkedMapOf<Long, Pair<Float, Float>>()
    private var pendingViewportMetrics: PendingViewportMetrics? = null
    private var frameInputScheduled = false
    private var isInputActive = false
    private val frameInputCallback = Runnable {
        if (frameInputScheduled) {
            frameInputScheduled = false
            measureInputMainThread { flushFrameInputState() }
        }
    }

    private var inputEventsReceived = 0L
    private var inputEventsCoalesced = 0L
    private var inputEventsDispatched = 0L
    private var inputRawDeltaX = 0.0
    private var inputRawDeltaY = 0.0
    private var maxInputQueueAge = 0.0
    private var inputAbiWallTime = 0.0
    private var inputMainThreadWallTime = 0.0

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        System.loadLibrary("zircon_export_"#,
    );
    source.push_str(&native_library_stem(&profile.output_name));
    source.push_str(
        r#"")

        ZirconRuntime.start()
        frameView = window.decorView
        frameView.setOnTouchListener { _: View, event: MotionEvent ->
            measureInputMainThread { forwardTouch(event) }
            true
        }
        frameView.addOnLayoutChangeListener { view, _, _, _, _, _, _, _, _ ->
            measureInputMainThread {
                val width = view.width
                val height = view.height
                queueViewportMetrics(width, height, resources.displayMetrics.density)
            }
        }
    }

    override fun onResume() {
        super.onResume()
        measureInputMainThread { resumeInput() }
    }

    override fun onPause() {
        measureInputMainThread { suspendInput() }
        super.onPause()
    }

    override fun onStart() {
        super.onStart()
        ZirconRuntime.dispatchLifecycle(ZIRCON_LIFECYCLE_FOREGROUND)
    }

    override fun onStop() {
        measureInputMainThread { flushFrameInput() }
        ZirconRuntime.dispatchLifecycle(ZIRCON_LIFECYCLE_BACKGROUND)
        super.onStop()
    }

    override fun onDestroy() {
        measureInputMainThread { flushFrameInput() }
        ZirconRuntime.shutdown()
        super.onDestroy()
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        measureInputMainThread {
            flushFrameInput()
            inputEventsReceived += 1
            measureInputAbi {
                ZirconRuntime.dispatchKeyboard(ZIRCON_KEY_PRESSED, event.keyCode, event.scanCode, null)
            }
            inputEventsDispatched += 1
        }
        return super.onKeyDown(keyCode, event)
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        measureInputMainThread {
            flushFrameInput()
            inputEventsReceived += 1
            measureInputAbi {
                ZirconRuntime.dispatchKeyboard(ZIRCON_KEY_RELEASED, event.keyCode, event.scanCode, null)
            }
            inputEventsDispatched += 1
        }
        return super.onKeyUp(keyCode, event)
    }

    fun inputTelemetry(): Map<String, Double> = mapOf(
        "inputEventsReceived" to inputEventsReceived.toDouble(),
        "inputEventsCoalesced" to inputEventsCoalesced.toDouble(),
        "inputEventsDispatched" to inputEventsDispatched.toDouble(),
        "inputRawDeltaX" to inputRawDeltaX,
        "inputRawDeltaY" to inputRawDeltaY,
        "maxInputQueueAge" to maxInputQueueAge,
        "inputAbiWallTime" to inputAbiWallTime,
        "inputMainThreadWallTime" to inputMainThreadWallTime,
    )

    private fun forwardTouch(event: MotionEvent) {
        if (!isInputActive) {
            return
        }
        when (event.actionMasked) {
            MotionEvent.ACTION_MOVE -> queueTouchMoves(event)
            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> {
                dispatchTouchEdge(event, event.actionIndex, ZIRCON_TOUCH_STARTED)
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_POINTER_UP -> {
                dispatchTouchEdge(event, event.actionIndex, ZIRCON_TOUCH_ENDED)
            }
            MotionEvent.ACTION_CANCEL -> dispatchCancelledTouches(event)
        }
    }

    private fun queueTouchMoves(event: MotionEvent) {
        val now = System.nanoTime()
        for (index in 0 until event.pointerCount) {
            val pointerId = event.getPointerId(index).toLong()
            val x = event.getX(index)
            val y = event.getY(index)
            val previousPosition = lastTouchPositions[pointerId]
            val pending = pendingTouchMoves[pointerId]
            inputEventsReceived += 1
            if (pending != null) {
                inputEventsCoalesced += 1
            }
            pendingTouchMoves[pointerId] = PendingTouchMove(
                pointerId = pointerId,
                x = x,
                y = y,
                deltaX = (pending?.deltaX ?: 0.0f) + x - (previousPosition?.first ?: x),
                deltaY = (pending?.deltaY ?: 0.0f) + y - (previousPosition?.second ?: y),
                queuedAtNanos = pending?.queuedAtNanos ?: now,
            )
            lastTouchPositions[pointerId] = Pair(x, y)
        }
        scheduleFrameInput()
    }

    private fun dispatchTouchEdge(event: MotionEvent, index: Int, phase: Int) {
        flushFrameInput()
        val pointerId = event.getPointerId(index).toLong()
        inputEventsReceived += 1
        dispatchTouchEvent(event, index, phase)
        inputEventsDispatched += 1
        if (phase == ZIRCON_TOUCH_ENDED) {
            lastTouchPositions.remove(pointerId)
        } else {
            lastTouchPositions[pointerId] = Pair(event.getX(index), event.getY(index))
        }
    }

    private fun dispatchCancelledTouches(event: MotionEvent) {
        flushFrameInput()
        for (index in 0 until event.pointerCount) {
            val pointerId = event.getPointerId(index).toLong()
            inputEventsReceived += 1
            dispatchTouchEvent(event, index, ZIRCON_TOUCH_CANCELLED)
            inputEventsDispatched += 1
            lastTouchPositions.remove(pointerId)
        }
    }

    private fun dispatchTouchEvent(event: MotionEvent, index: Int, phase: Int) {
        measureInputAbi {
            ZirconRuntime.dispatchTouch(event.getPointerId(index).toLong(), phase, event.getX(index), event.getY(index))
        }
    }

    private fun dispatchPendingTouchMove(pending: PendingTouchMove, now: Long) {
        measureInputAbi {
            ZirconRuntime.dispatchTouch(pending.pointerId, ZIRCON_TOUCH_MOVED, pending.x, pending.y)
        }
        inputRawDeltaX += pending.deltaX
        inputRawDeltaY += pending.deltaY
        inputEventsDispatched += 1
        maxInputQueueAge = maxOf(
            maxInputQueueAge,
            (now - pending.queuedAtNanos).toDouble() / 1_000_000.0,
        )
    }

    private fun queueCurrentViewportMetrics() {
        if (!frameView.isLaidOut || frameView.isLayoutRequested) {
            return
        }
        queueViewportMetrics(frameView.width, frameView.height, resources.displayMetrics.density)
    }

    private fun queueViewportMetrics(width: Int, height: Int, scale: Float) {
        if (!isInputActive || width <= 0 || height <= 0) {
            return
        }
        val now = System.nanoTime()
        inputEventsReceived += 1
        if (pendingViewportMetrics != null) {
            inputEventsCoalesced += 1
        }
        pendingViewportMetrics = PendingViewportMetrics(
            width = width,
            height = height,
            scale = scale,
            queuedAtNanos = pendingViewportMetrics?.queuedAtNanos ?: now,
        )
        scheduleFrameInput()
    }

    private fun scheduleFrameInput() {
        if (frameInputScheduled) {
            return
        }
        frameInputScheduled = true
        frameView.postOnAnimation(frameInputCallback)
    }

    private fun flushFrameInput() {
        if (frameInputScheduled) {
            frameView.removeCallbacks(frameInputCallback)
            frameInputScheduled = false
        }
        flushFrameInputState()
    }

    private fun flushFrameInputState() {
        val now = System.nanoTime()
        val moveIterator = pendingTouchMoves.entries.iterator()
        while (moveIterator.hasNext()) {
            val pending = moveIterator.next().value
            moveIterator.remove()
            dispatchPendingTouchMove(pending, now)
        }
        val metrics = pendingViewportMetrics ?: return
        pendingViewportMetrics = null
        measureInputAbi {
            ZirconRuntime.dispatchViewportMetrics(metrics.width, metrics.height, metrics.scale)
        }
        inputEventsDispatched += 1
        maxInputQueueAge = maxOf(
            maxInputQueueAge,
            (now - metrics.queuedAtNanos).toDouble() / 1_000_000.0,
        )
    }

    private fun suspendInput() {
        if (!isInputActive) {
            return
        }
        flushFrameInput()
        isInputActive = false
        for ((pointerId, position) in lastTouchPositions.entries.sortedBy { it.key }) {
            inputEventsReceived += 1
            measureInputAbi {
                ZirconRuntime.dispatchTouch(pointerId, ZIRCON_TOUCH_CANCELLED, position.first, position.second)
            }
            inputEventsDispatched += 1
        }
        lastTouchPositions.clear()
        pendingTouchMoves.clear()
        pendingViewportMetrics = null
        measureInputAbi {
            ZirconRuntime.dispatchLifecycle(ZIRCON_LIFECYCLE_SUSPENDED)
        }
    }

    private fun resumeInput() {
        if (isInputActive) {
            return
        }
        isInputActive = true
        measureInputAbi {
            ZirconRuntime.dispatchLifecycle(ZIRCON_LIFECYCLE_RESUMED)
        }
        queueCurrentViewportMetrics()
    }

    private inline fun measureInputAbi(operation: () -> Unit) {
        val startedAt = System.nanoTime()
        operation()
        inputAbiWallTime += (System.nanoTime() - startedAt).toDouble() / 1_000_000.0
    }

    private inline fun measureInputMainThread(operation: () -> Unit) {
        val startedAt = System.nanoTime()
        operation()
        inputMainThreadWallTime += (System.nanoTime() - startedAt).toDouble() / 1_000_000.0
    }
}
"#,
    );
    source
}

fn android_runtime_binding_template() -> String {
    "package dev.zircon.export\n\nobject ZirconRuntime {\n    external fun start(): Boolean\n    external fun shutdown(): Boolean\n    external fun dispatchLifecycle(state: Int): Boolean\n    external fun dispatchTouch(pointerId: Long, phase: Int, x: Float, y: Float): Boolean\n    external fun dispatchKeyboard(action: Int, keyCode: Int, scanCode: Int, text: String?): Boolean\n    external fun dispatchViewportMetrics(logicalWidth: Int, logicalHeight: Int, scale: Float): Boolean\n}\n"
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
    let mut source = String::with_capacity(9_216);
    source.push_str(
        r#"import QuartzCore
import SwiftUI
import UIKit

let ZIRCON_LIFECYCLE_RESUMED: UInt32 = 4
let ZIRCON_LIFECYCLE_SUSPENDED: UInt32 = 8
let ZIRCON_TOUCH_STARTED: UInt32 = 1
let ZIRCON_TOUCH_MOVED: UInt32 = 2
let ZIRCON_TOUCH_ENDED: UInt32 = 3
let ZIRCON_TOUCH_CANCELLED: UInt32 = 4
let ZIRCON_KEY_TEXT: UInt32 = 3

@_silgen_name("zircon_export_start")
func zircon_export_start() -> Bool
@_silgen_name("zircon_export_shutdown")
func zircon_export_shutdown() -> Bool
@_silgen_name("zircon_export_handle_lifecycle")
func zircon_export_handle_lifecycle(_ state: UInt32) -> Bool
@_silgen_name("zircon_export_handle_touch")
func zircon_export_handle_touch(_ pointerId: UInt64, _ phase: UInt32, _ x: Float, _ y: Float) -> Bool
@_silgen_name("zircon_export_handle_keyboard")
func zircon_export_handle_keyboard(_ action: UInt32, _ keyCode: UInt32, _ scanCode: UInt32, _ text: UnsafePointer<UInt8>?, _ textLen: Int) -> Bool
@_silgen_name("zircon_export_handle_viewport_metrics")
func zircon_export_handle_viewport_metrics(_ logicalWidth: UInt32, _ logicalHeight: UInt32, _ scale: Float) -> Bool

private struct PendingTouchMove {
    let touchId: UInt64
    let point: CGPoint
    let deltaX: CGFloat
    let deltaY: CGFloat
    let queuedAt: CFTimeInterval
}

private struct ActiveTouch {
    let touchId: UInt64
    let point: CGPoint
}

private final class ZirconDisplayLinkTarget: NSObject {
    weak var owner: ZirconTouchView?

    init(owner: ZirconTouchView) {
        self.owner = owner
    }

    @objc func tick(_ displayLink: CADisplayLink) {
        owner?.flushScheduledFrameInput()
    }
}

final class ZirconRuntimeApplicationDelegate: NSObject, UIApplicationDelegate {
    func applicationWillTerminate(_ application: UIApplication) {
        _ = zircon_export_shutdown()
    }
}

struct ZirconRuntimeView: UIViewRepresentable {
    func makeUIView(context: Context) -> ZirconTouchView { ZirconTouchView(frame: .zero) }
    func updateUIView(_ uiView: ZirconTouchView, context: Context) { }
}

final class ZirconTouchView: UIView {
    private var pendingTouchMoves: [ObjectIdentifier: PendingTouchMove] = [:]
    private var pendingTouchOrder: [ObjectIdentifier] = []
    private var touchIds: [ObjectIdentifier: UInt64] = [:]
    private var activeTouches: [ObjectIdentifier: ActiveTouch] = [:]
    private var nextTouchId: UInt64 = 1
    private var pendingViewportMetricsAt: CFTimeInterval?
    private var displayLink: CADisplayLink?
    private lazy var displayLinkTarget = ZirconDisplayLinkTarget(owner: self)
    private var lifecycleObservers: [NSObjectProtocol] = []
    private var isInputActive = true

    private var inputEventsReceived: UInt64 = 0
    private var inputEventsCoalesced: UInt64 = 0
    private var inputEventsDispatched: UInt64 = 0
    private var inputRawDeltaX: Double = 0
    private var inputRawDeltaY: Double = 0
    private var maxInputQueueAge: Double = 0
    private var inputAbiWallTime: Double = 0
    private var inputMainThreadWallTime: Double = 0

    override init(frame: CGRect) {
        super.init(frame: frame)
        configureInput()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        configureInput()
    }

    private func configureInput() {
        isMultipleTouchEnabled = true
        registerLifecycleObservers()
    }

    deinit {
        displayLink?.invalidate()
        for observer in lifecycleObservers {
            NotificationCenter.default.removeObserver(observer)
        }
    }

    var inputTelemetry: [String: Double] {
        [
            "inputEventsReceived": Double(inputEventsReceived),
            "inputEventsCoalesced": Double(inputEventsCoalesced),
            "inputEventsDispatched": Double(inputEventsDispatched),
            "inputRawDeltaX": inputRawDeltaX,
            "inputRawDeltaY": inputRawDeltaY,
            "maxInputQueueAge": maxInputQueueAge,
            "inputAbiWallTime": inputAbiWallTime,
            "inputMainThreadWallTime": inputMainThreadWallTime,
        ]
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        measureInputMainThread { queueViewportMetrics() }
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        measureInputMainThread {
            dispatchTouchEdges(touches, phase: ZIRCON_TOUCH_STARTED)
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        measureInputMainThread { queueTouchMoves(touches) }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        measureInputMainThread {
            dispatchTouchEdges(touches, phase: ZIRCON_TOUCH_ENDED)
        }
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        measureInputMainThread {
            dispatchTouchEdges(touches, phase: ZIRCON_TOUCH_CANCELLED)
        }
    }

    private func queueTouchMoves(_ touches: Set<UITouch>) {
        guard isInputActive else {
            return
        }
        let now = CACurrentMediaTime()
        for (touch, identifier, touchId) in orderedTouches(touches) {
            let point = touch.location(in: self)
            let previousPoint = touch.previousLocation(in: self)
            let pending = pendingTouchMoves[identifier]
            inputEventsReceived += 1
            if pending != nil {
                inputEventsCoalesced += 1
            } else {
                pendingTouchOrder.append(identifier)
            }
            pendingTouchMoves[identifier] = PendingTouchMove(
                touchId: touchId,
                point: point,
                deltaX: (pending?.deltaX ?? 0) + point.x - previousPoint.x,
                deltaY: (pending?.deltaY ?? 0) + point.y - previousPoint.y,
                queuedAt: pending?.queuedAt ?? now
            )
            activeTouches[identifier] = ActiveTouch(touchId: touchId, point: point)
        }
        scheduleFrameInput()
    }

    private func dispatchTouchEdges(_ touches: Set<UITouch>, phase: UInt32) {
        guard isInputActive else {
            return
        }
        flushFrameInput()
        for (touch, identifier, touchId) in orderedTouches(touches) {
            let point = touch.location(in: self)
            inputEventsReceived += 1
            measureInputAbi {
                _ = zircon_export_handle_touch(touchId, phase, Float(point.x), Float(point.y))
            }
            inputEventsDispatched += 1
            if phase == ZIRCON_TOUCH_ENDED || phase == ZIRCON_TOUCH_CANCELLED {
                activeTouches.removeValue(forKey: identifier)
                touchIds.removeValue(forKey: identifier)
            } else {
                activeTouches[identifier] = ActiveTouch(touchId: touchId, point: point)
            }
        }
    }

    private func registerLifecycleObservers() {
        let center = NotificationCenter.default
        lifecycleObservers.append(
            center.addObserver(
                forName: UIApplication.willResignActiveNotification,
                object: nil,
                queue: .main
            ) { [weak self] _ in
                guard let self else { return }
                self.measureInputMainThread { self.suspendInput() }
            }
        )
        lifecycleObservers.append(
            center.addObserver(
                forName: UIApplication.didBecomeActiveNotification,
                object: nil,
                queue: .main
            ) { [weak self] _ in
                guard let self else { return }
                self.measureInputMainThread { self.resumeInput() }
            }
        )
    }

    private func touchObjectSortKey(_ touch: UITouch) -> UInt {
        UInt(bitPattern: Unmanaged.passUnretained(touch).toOpaque())
    }

    private func touchId(for touch: UITouch) -> UInt64 {
        let identifier = ObjectIdentifier(touch)
        if let existing = touchIds[identifier] {
            return existing
        }
        precondition(nextTouchId < UInt64.max, "Zircon touch identifier space exhausted")
        let allocated = nextTouchId
        nextTouchId += 1
        touchIds[identifier] = allocated
        return allocated
    }

    private func orderedTouches(
        _ touches: Set<UITouch>
    ) -> [(touch: UITouch, identifier: ObjectIdentifier, touchId: UInt64)] {
        let ordered = touches.sorted { left, right in
            let leftIdentifier = ObjectIdentifier(left)
            let rightIdentifier = ObjectIdentifier(right)
            switch (touchIds[leftIdentifier], touchIds[rightIdentifier]) {
            case let (leftId?, rightId?):
                return leftId < rightId
            case (_?, nil):
                return true
            case (nil, _?):
                return false
            case (nil, nil):
                return touchObjectSortKey(left) < touchObjectSortKey(right)
            }
        }
        return ordered.map { touch in
            (touch, ObjectIdentifier(touch), touchId(for: touch))
        }
    }

    private func suspendInput() {
        guard isInputActive else { return }
        flushFrameInput()
        isInputActive = false
        for active in activeTouches.values.sorted(by: { $0.touchId < $1.touchId }) {
            inputEventsReceived += 1
            measureInputAbi {
                _ = zircon_export_handle_touch(
                    active.touchId,
                    ZIRCON_TOUCH_CANCELLED,
                    Float(active.point.x),
                    Float(active.point.y)
                )
            }
            inputEventsDispatched += 1
        }
        activeTouches.removeAll(keepingCapacity: true)
        touchIds.removeAll(keepingCapacity: true)
        pendingTouchMoves.removeAll(keepingCapacity: true)
        pendingTouchOrder.removeAll(keepingCapacity: true)
        pendingViewportMetricsAt = nil
        displayLink?.isPaused = true
        measureInputAbi {
            _ = zircon_export_handle_lifecycle(ZIRCON_LIFECYCLE_SUSPENDED)
        }
    }

    private func resumeInput() {
        guard !isInputActive else { return }
        isInputActive = true
        measureInputAbi {
            _ = zircon_export_handle_lifecycle(ZIRCON_LIFECYCLE_RESUMED)
        }
        queueViewportMetrics()
    }

    private func queueViewportMetrics() {
        guard isInputActive else {
            return
        }
        inputEventsReceived += 1
        if pendingViewportMetricsAt != nil {
            inputEventsCoalesced += 1
        } else {
            pendingViewportMetricsAt = CACurrentMediaTime()
        }
        scheduleFrameInput()
    }

    private func scheduleFrameInput() {
        if displayLink == nil {
            let link = CADisplayLink(
                target: displayLinkTarget,
                selector: #selector(ZirconDisplayLinkTarget.tick(_:))
            )
            link.add(to: .main, forMode: .common)
            link.isPaused = true
            displayLink = link
        }
        displayLink?.isPaused = false
    }

    fileprivate func flushScheduledFrameInput() {
        measureInputMainThread { flushFrameInput() }
    }

    private func flushFrameInput() {
        displayLink?.isPaused = true
        let now = CACurrentMediaTime()
        for identifier in pendingTouchOrder {
            flushPendingTouchMove(identifier, now: now)
        }
        pendingTouchOrder.removeAll(keepingCapacity: true)
        guard let queuedAt = pendingViewportMetricsAt else {
            return
        }
        pendingViewportMetricsAt = nil
        let size = bounds.size
        let scale = window?.screen.scale ?? UIScreen.main.scale
        measureInputAbi {
            _ = zircon_export_handle_viewport_metrics(UInt32(size.width), UInt32(size.height), Float(scale))
        }
        inputEventsDispatched += 1
        maxInputQueueAge = max(maxInputQueueAge, (now - queuedAt) * 1_000)
    }

    private func flushPendingTouchMove(_ identifier: ObjectIdentifier, now: CFTimeInterval) {
        guard let pending = pendingTouchMoves.removeValue(forKey: identifier) else {
            return
        }
        measureInputAbi {
            _ = zircon_export_handle_touch(
                pending.touchId,
                ZIRCON_TOUCH_MOVED,
                Float(pending.point.x),
                Float(pending.point.y)
            )
        }
        inputRawDeltaX += Double(pending.deltaX)
        inputRawDeltaY += Double(pending.deltaY)
        inputEventsDispatched += 1
        maxInputQueueAge = max(maxInputQueueAge, (now - pending.queuedAt) * 1_000)
    }

    private func measureInputAbi(_ operation: () -> Void) {
        let startedAt = CACurrentMediaTime()
        operation()
        inputAbiWallTime += (CACurrentMediaTime() - startedAt) * 1_000
    }

    private func measureInputMainThread(_ operation: () -> Void) {
        let startedAt = CACurrentMediaTime()
        operation()
        inputMainThreadWallTime += (CACurrentMediaTime() - startedAt) * 1_000
    }
}

@main
struct ZirconRuntimeHostApp: App {
    @UIApplicationDelegateAdaptor(ZirconRuntimeApplicationDelegate.self) private var applicationDelegate

    init() {
        _ = zircon_export_start()
        _ = zircon_export_handle_lifecycle(ZIRCON_LIFECYCLE_RESUMED)
        let text = Array(""#,
    );
    source.push_str(&swift_string_escape(&profile.output_name));
    source.push_str(
        r#"".utf8)
        text.withUnsafeBufferPointer { buffer in
            _ = zircon_export_handle_keyboard(ZIRCON_KEY_TEXT, 0, 0, buffer.baseAddress, buffer.count)
        }
    }

    var body: some Scene {
        WindowGroup {
            ZirconRuntimeView()
        }
    }
}
"#,
    );
    source
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
    "#pragma once\n#include <stdbool.h>\n#include <stddef.h>\n#include <stdint.h>\n\nbool zircon_export_start(void);\nbool zircon_export_shutdown(void);\nbool zircon_export_handle_lifecycle(uint32_t state);\nbool zircon_export_handle_touch(uint64_t pointer_id, uint32_t phase, float x, float y);\nbool zircon_export_handle_keyboard(uint32_t action, uint32_t key_code, uint32_t scan_code, const uint8_t *text, size_t text_len);\nbool zircon_export_handle_viewport_metrics(uint32_t logical_width, uint32_t logical_height, float scale);\nbool zircon_export_fetch_resource(const char *uri, uint32_t flags);\n".to_string()
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
