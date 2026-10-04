---
handoff_kind: failure
status: open
created_at: 2026-09-28
summary_slug: portable-surface-v9-abi-contract
origin_plan: docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/optimize/zircon_runtime/57
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_shape.rs
  - zircon_runtime_interface/src/runtime_build_set/interface_spec_v1.json
  - zircon_runtime_interface/build.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_editor/src/core/gateway/session/gateway.rs
  - zircon_editor/src/core/gateway/session/contract.rs
  - zircon_editor/src/core/gateway/session/viewport.rs
  - zircon_editor/src/core/gateway/session/tests.rs
  - zircon_editor/src/core/gateway/handle.rs
  - tools/build/zircon_build_abi.py
  - tools/build/zircon_build_runtime_manifest.py
  - tools/build/zircon_build_staging_manifest.py
  - tools/tests/test_zircon_build_abi.py
  - tools/tests/test_zircon_build_runtime_manifest.py
  - tools/tests/test_zircon_build_staging_manifest.py
tests:
  - 'cargo +1.94.1 test -p zircon_runtime_interface --no-default-features --locked --lib tests::abi_safety_contracts:: -- --test-threads=1'
  - 'cargo +1.94.1 test -p zircon_runtime_interface --no-default-features --locked --lib runtime_api::abi::api_shape_tests:: -- --test-threads=1'
  - 'cargo +1.94.1 test -p zircon_runtime --no-default-features --locked --lib dynamic_api::tests::viewport:: -- --test-threads=1'
  - 'cargo +1.94.1 test -p zircon_app --no-default-features --locked --lib entry::runtime_library::tests:: -- --test-threads=1'
  - 'cargo +1.94.1 test -p zircon_editor --no-default-features --locked --lib core::gateway::session::tests:: -- --test-threads=1'
  - python -m unittest tools.tests.test_zircon_build_abi tools.tests.test_zircon_build_runtime_manifest tools.tests.test_zircon_build_staging_manifest
---

# Runtime10: versioned portable surface ABI contract

## 来源执行者

- 来源计划： docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
- 来源执行切片： Runtime57 portable host surface identity and lifetime gate.
- 修复责任计划： docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
- 交接原因： Runtime10 owns the public versioned ABI table and contract.
- Origin: Runtime57's portable host surface identity and lifetime gate, recorded in [its open failure](../../../optimize/zircon_runtime/57/failure-2026-09-28-export-portable-surface-target-host-lifecycle.md).
- Fixing owner: Runtime10 owns the public dynamic API table, InterfaceSpec, ABI DTO layout, entry symbol and host/runtime migration. Runtime57 remains responsible for host identity, lifecycle and App/Graphics integration; Runtime90 owns physical RHI acquisition and present.
- This child record carries the cross-plan link. Numbered plan definitions remain read-only.

## 失败现象与复现证据

The current `#[repr(C)]` `ZrRuntimeNativeSurfaceTargetV1` has only `abi_version`, `kind`, `window_handle` and `display_handle`. Its constructors represent NONE and WIN32. `ZrRuntimeBindViewportSurfaceRequestV1` embeds that target by value, and the V8 table's bind slot accepts the V1 request by value. There is no qualified Android native-window, iOS view/layer or browser canvas target, nor a host-owned surface generation in this public request.

The current InterfaceSpec fixes runtime API version 8 and `zircon_runtime_get_api_v8`. The App loader checks the exact table version and size. Editor `SessionGateway` stores `ZrRuntimeApiV8` directly and its gateway contract passes the V1 bind request, so Editor is also a direct migration consumer. At the 2026-09-28 read-only boundary, `viewport.rs` had SHA-256 `23de5d37bbd22a876ddbfa2fac2944cf14afb2254550f4a7f7feb67e9a4664dc`, `api_table.rs` `61d3bc871934e5e02e4652d26e0aa3ae82c96e50e9eeac5a1529ad7cd34c1b95`, and `interface_spec_v1.json` `db3c639db8e6a8385dad4959d178b0c95894b51ae87531f46d87ced6955e1fa4`. This is a static contract finding; no Cargo or portable product gate has passed for this failure.

## 最低共享层根因

The published V8 by-value bind request cannot acquire typed portable host objects or reject a stale generation without changing the meaning or layout of the V1 request. Reinterpreting its two raw Win32 handle fields as opaque tokens would silently change the existing ABI. The version gate and all current producers and consumers therefore need a coordinated new API table version before Runtime57 can accept the portable lifecycle.

## 架构修复验收

- Define a V9 bind request and typed target contract with host-owned identity and generation. Specify validity, ownership, release and stale-generation rejection at the ABI boundary. Keep the V8 layout and semantics immutable during the migration.
- Move the current InterfaceSpec/catalog, generated version and entry symbol, runtime export, App loader/session adapter, Editor `SessionGateway` and its bind contract, and the ABI/runtime/staging manifest builders to the new version together. Run their focused Python regressions, add exact table layout/version tests and lower-layer target/identity tests, and retain negative coverage for old or malformed layouts.
- The frontmatter Cargo commands are test selectors for coordinator-managed Windows submissions, not standalone invocations. Freeze one source snapshot; let the coordinator allocate each physical target below `D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets`; retain `--locked`; and require nonzero selected test counts for interface, runtime, App and Editor gates. Then rerun Runtime57's bind/resize/suspend/rebind/release gate and the originating Plugins09 mobile/browser generated-host and real-surface acceptance. Each target must show at least two nonblank presented frames, resize or DPI change, pause or visibility change, and clean shutdown before the upstream failure can close.
- Bind managed validation receipts, independent Critical/Important/Moderate-zero review, canonical `failure return` and closeout to the exact source and configuration used. No static check or queued receipt counts as dynamic acceptance.

## 禁止临时方案

Do not mutate or reinterpret the V8 request in place, encode borrowed native or JavaScript objects as fake Win32 integers, return success without a bound surface, duplicate the host owner in the runtime, or weaken the ABI shape and upstream product gates.

## 修复结果与回传

Open state: Runtime10 ABI repair and managed validation pending. The Runtime57 and Plugins09 dependent gates remain open; no Cargo, product run, review, return, commit or WeCom pass is claimed here.
