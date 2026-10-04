---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-09-27
summary_slug: required-native-plugin-admission-uses-discovery-metadata
origin_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
fixing_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
origin_child_dir: docs/plans/optimize/zircon_app/08
fixing_child_dir: docs/plans/optimize/zircon_app/08
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/product_composition/request.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_app/src/entry/tests/export_bootstrap.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -ManifestPath zircon_plugins/Cargo.toml -Package zircon_plugin_virtual_geometry_dist -NoDefaultFeatures -Features dist -SkipTest -TargetDir $AssignedCargoTarget -LinkMode static
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -ManifestPath zircon_plugins/Cargo.toml -Package zircon_plugin_virtual_geometry_dist -NoDefaultFeatures -Features dist -LibTests -TestFilter virtual_geometry_dist_ -TestThreads 1 -NoCapture -LinkMode static
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -LibTests -TestFilter required_native_plugin_with_discovered_manifest_but_missing_library_is_rejected -TestThreads 1 -NoCapture -LinkMode static
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -LibTests -TestFilter bootstrap_accepts_required_native_dynamic_plugin_from_export_load_manifest -IgnoredTests -TestThreads 1 -NoCapture -TargetDir $AssignedCargoTarget -LinkMode static
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -LibTests -TestFilter native_export_runtime_bootstrap_merges_linked_and_native_reports -TestThreads 1 -NoCapture -LinkMode static
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -LibTests -TestFilter required_native_export_plugin_rejects_discovery_without_a_library -TestThreads 1 -NoCapture -LinkMode static
---

# App08: required NativeDynamic admission accepts discovered metadata without a loaded library

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 来源执行切片：Runtime55 Foundation config-path upward App08 profile replay.
- 修复责任计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 交接原因：App composition request decides product native admission; the native loader retains discovery projection for other catalog consumers.

## 失败现象与复现证据

At `bc02eefafead65dbf5050482110e8175250a5e77`, `ProductCompositionRequest::prepare` loads a native export root and accepts a required `NativeDynamic` selection when `runtime_plugin_registration_reports` contains its package ID. `NativePluginLiveHost::load_reported_plugins_result` builds those reports from discovery before loading libraries. A valid manifest with a missing DLL therefore has a registration report and no loaded runtime plugin.

The existing App08 test `bootstrap_accepts_required_native_dynamic_plugin_from_export_load_manifest` writes only a short manifest, selects the default `LibraryEmbed` packaging, then claims successful NativeDynamic bootstrap while asserting that no native plugin is loaded and that the library is missing. This is a misleading positive fixture. The existing `required_native_plugin_rejection_prevents_product_preparation` has no discovered manifest and does not cover this failure.

The exact current generated VirtualGeometry manifest is embedded by `include_str!` from `zircon_plugins/virtual_geometry/plugin.toml` into both App test fixtures, so the coordinator can seal it as a compile-time input. Its source SHA-256 was `f452827664f629a12789a26665556720e943503af07322e058dc311442fd6829` at this repair boundary. The negative fixture stages those bytes without a dist DLL, requires metadata projection, and expects `NativePluginAdmission` before Core bootstrap. This test has been added but has **not** executed. No current Cargo failure or pass is claimed. Read-only root diagnosis is in `.codex/tmp/failure-roll-01a0df1a-app08-native-dynamic-admission-triage.json`.

## 最低共享层根因

Discovery registration is catalog metadata, not evidence that an artifact passed authority checks, loaded into the live Runtime host, and exposed a valid runtime entry. `loaded_plugin_ids` alone also does not prove a valid entry: the loader can retain a library after a requested-entry error. App08 must require the same-call loaded ID and a present, non-`Invalid` runtime behavior validation report, in addition to matching registration metadata. `Degraded` remains admitted with diagnostics because the actual VirtualGeometry dist entry is stateless with a valid registration manifest but omits optional unload and command callbacks. An `Invalid`, absent, or unavailable runtime entry does not satisfy a *required* NativeDynamic selection. Loader projection semantics stay owned by Plugins01.

## 架构修复验收

- App08 request checks matching registration, actual same-call loaded ID, and a non-`Invalid` live Runtime entry for every enabled required NativeDynamic selection. A missing export root rejects the required selection even if the caller injects a discovery report. Caller supplied NativeDynamic plugin or feature reports are excluded from product availability even when the selection is optional; only reports from this request's admitted live host may enter the product. Discovered reports remain available from the loader API; preserve `Degraded` diagnostics.
- The missing-DLL negative uses the exact generated manifest through a compile-time include, explicitly selects NativeDynamic for ClientRuntime, verifies discovery projected the candidate, and must execute under managed Cargo with its fixture under the assigned drive-root `cargo-targets` target. The compile-time input scanner resolves both includes to the same manifest path. It also checks that injected discovery metadata cannot bypass the required gate without an export root or create optional native availability with or without an export root.
- Build the real `zircon_plugin_virtual_geometry_dist.dll` into one coordinator-assigned development-profile target, then run the ignored profile positive against that same `-TargetDir`. The test reads only `debug/zircon_plugin_virtual_geometry_dist.dll` beneath its managed `CARGO_TARGET_DIR`, prints `APP08_NATIVE_DIST_SHA256`, verifies staged bytes have that same digest, captures trusted-local authority for the staged package, and requires an actually loaded runtime ID, a present non-`Invalid` entry and its runtime registration manifest. Before accepting the positive ticket, compare its printed digest with the recorded DLL SHA-256 from the exact upstream managed build ticket; retain both receipts and reject any mismatch. This uses no daemon ambient environment variables. The direct `export_bootstrap` consumer now checks that a missing optional DLL contributes diagnostics but no native availability/module, while a required NativeDynamic selection fails. VirtualGeometry GPU rendering parity remains the separately owned Plugins17 gate.
- Run the lower dist entry tests, focused negative, original profile test, direct export consumer test, and the affected App08 profile batch through managed Windows `--locked` validation. Confirm that exact filters execute their named tests. Build products and caches must physically reside under a coordinator-allocated `D:/cargo-targets`, `E:/cargo-targets`, or `F:/cargo-targets` root.

## 禁止临时方案

- Do not count a discovered manifest, a fake registration report, an empty live host, or a skipped DLL as positive native admission.
- Do not remove loader discovery metadata, weaken artifact authority, silently recapture changed bytes, or weaken the product acceptance assertions.
- Do not treat structural checks or pending validation receipts as a dynamic pass.

## 修复结果与回传

Open: `source_repaired_real_dll_and_managed_validation_pending`. Stable App08 Session `failure-roll-01a0df1a-app08-config-path-r1` owns the request, profile test, export test and this record after exact transfer fingerprints `b68d0ba066a7fb86e8a5d366cf7b4ff16e0a0ababf969482c5ebc65c91ad4698`, `750e457a8f30c82f2c1b0e9386a78b503766c881605fe5198368a456d94b0b36`, and `a0295f746078a4d56441d9992a6277d5b1a76dc7894a3e0eec46e1e7436121b2`. The request preimage is `.codex/tmp/failure-roll-01a0df1a-app08-native-request-before.rs` at SHA-256 `309d4ed50af02fcafeefaa31475f91cd2d78fea4b7b8b7b91b1a269ac5e2e2d9`; the profile and export preimages are `.codex/tmp/failure-roll-01a0df1a-app08-native-profile-before.rs` and `.codex/tmp/failure-roll-01a0df1a-app08-native-export-before.rs`.

The original profile positive and export consumer have source changes but lack execution evidence. Static review of snapshot 5086 found C0/I0/M1; its manifest-closure Moderate item was repaired with compile-time includes and needs review on the next exact snapshot. No managed ticket, Cargo pass, `failure return`, formal closeout review, closeout, commit, or WeCom result is claimed. Runtime55's config-path producer is a separate failure and does not close this App08 admission lifecycle.
