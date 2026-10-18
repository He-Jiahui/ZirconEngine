# Runtime Picking / Gizmos / Camera / Remote Bevy Completion Plan

## Summary

目标是把 ZirconEngine 的 viewport picking、overlay/gizmos、camera controller、remote editor protocol、diagnostics/dev tools 从 editor-private 能力推进为 runtime-owned reusable primitives。Editor 只保留输入适配、工具 UX、选择/undo/inspector 绑定；runtime 拥有可被 app、editor、plugin、remote/dev tools 复用的协议、数据结构和算法。

本计划是 2026-05-21 的更新版。当前源码已经不再是“runtime 完全缺失 picking/gizmos”：`zircon_runtime::core::framework::picking` 和 `zircon_runtime::core::framework::gizmos` 已存在，并有 runtime 单测覆盖。剩余工作应从“补零件”改成“补完成度、补 camera/remote/interface、做 editor cutover 与验收”。

## Reference Baseline

本地 Bevy 源码基线是 `dev/bevy`，Cargo 标识为 `0.19.0-dev`；公开 docs.rs 当前为 Bevy `0.18.1`，只作为公开 API 文档引用。以本地源码决定架构，以 docs.rs 决定外部 API 叙述。

### Bevy source evidence

Picking:

- `DefaultPickingPlugins` 组合 `PointerInputPlugin`、`PickingPlugin`、`InteractionPlugin`，说明 picking 是默认交互管线而不是 editor 专有工具：[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_picking/src/lib.rs:283)。
- `Pickable` 提供 per-entity hover/blocking 语义：[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_picking/src/lib.rs:196)。
- `PickingSystems` 拆出 input、backend、hover、post-hover、last 阶段：[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_picking/src/lib.rs:256)。
- Backend 只输出 unordered `PointerHits`，核心 picking 管线统一排序、过滤、hover：[backend.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_picking/src/backend.rs:60)。
- `HitData` 与 `RayMap` 把 hit 信息和 camera ray cache 从 backend 中抽出来：[backend.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_picking/src/backend.rs:100)、[backend.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_picking/src/backend.rs:180)。
- Pointer event vocabulary 覆盖 `Over/Enter/Move/Leave/Out/Press/Release/Click/Drag*/Scroll/Cancel`：[events.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_picking/src/events.rs:67)、[events.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_picking/src/events.rs:1201)。

Gizmos:

- Immediate-mode `Gizmos` 和 backing `GizmoBuffer` 分离：[gizmos.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/gizmos.rs:143)、[gizmos.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/gizmos.rs:286)。
- 基础绘制 API 从 line/ray/strip 扩展到 2D/3D primitives：[gizmos.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/gizmos.rs:408)、[gizmos.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/gizmos.rs:456)。
- `GizmoConfigStore` 和 `GizmoConfig` 提供 group、enabled、line width、render layer 等配置：[config.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/config.rs:99)、[config.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/config.rs:208)。
- Retained gizmos 通过 `GizmoAsset` / `Gizmo` 复用 buffer，适合静态 debug geometry：[retained.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/retained.rs:67)。
- Transform gizmo 有 focus/camera markers、mode/space/axis、settings/state、plugin 和 axis math：[transform_gizmo.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/transform_gizmo.rs:88)、[transform_gizmo.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/transform_gizmo.rs:138)、[transform_gizmo.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/transform_gizmo.rs:181)、[transform_gizmo.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_gizmos/src/transform_gizmo.rs:226)。

Camera controllers:

- Free camera 以 plugin + component settings + required state + controller system 组织：[free_camera.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera_controller/src/free_camera.rs:41)、[free_camera.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera_controller/src/free_camera.rs:77)、[free_camera.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera_controller/src/free_camera.rs:208)、[free_camera.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera_controller/src/free_camera.rs:248)。
- Pan camera 独立为 2D/pan/zoom controller，并可通过 pointer drag 集成：[pan_camera.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera_controller/src/pan_camera.rs:26)、[pan_camera.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera_controller/src/pan_camera.rs:44)、[pan_camera.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera_controller/src/pan_camera.rs:166)、[pan_camera.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera_controller/src/pan_camera.rs:311)。

Remote:

- `RemotePlugin` 注册 remote method set，并把 protocol setup 与 HTTP transport 分离：[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/lib.rs:572)。
- `RemoteMethods` 是 method name 到 handler 的 registry：[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/lib.rs:959)。
- `BrpRequest` / `BrpResponse` / `BrpError` 是 JSON-RPC-style request/response/error DTO：[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/lib.rs:1032)、[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/lib.rs:1153)、[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/lib.rs:1198)。
- Request processing drains a mailbox and dispatches through `RemoteMethods`：[lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/lib.rs:1376)。
- Built-in methods cover world components/resources/query/mutation, registry schema, schedule graph/list, discovery：[builtin_methods.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/builtin_methods.rs:42)、[builtin_methods.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/builtin_methods.rs:95)、[builtin_methods.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/builtin_methods.rs:98)、[builtin_methods.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_remote/src/builtin_methods.rs:104)。

Diagnostics/dev tools:

- Diagnostics model is path + measurement timeline + store + deferred recording param + app registration：[diagnostic.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs:22)、[diagnostic.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs:115)、[diagnostic.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs:305)、[diagnostic.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs:348)、[diagnostic.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs:404)。
- Log diagnostics has cadence/filter state rather than ad hoc frame logging：[log_diagnostics_plugin.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs:17)。
- Diagnostics overlay consumes registered diagnostic paths and can display smoothed/value/average style statistics：[diagnostics_overlay.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_dev_tools/src/diagnostics_overlay.rs:74)、[diagnostics_overlay.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_dev_tools/src/diagnostics_overlay.rs:220)。

### Public documentation references

- [Bevy picking module docs](https://docs.rs/bevy/latest/bevy/picking/index.html)
- [DefaultPickingPlugins docs](https://docs.rs/bevy/latest/bevy/picking/struct.DefaultPickingPlugins.html)
- [Bevy gizmos module docs](https://docs.rs/bevy/latest/bevy/gizmos/index.html)
- [bevy_camera_controller docs](https://docs.rs/bevy_camera_controller/latest/bevy_camera_controller/)
- [FreeCamera docs](https://docs.rs/bevy/latest/bevy/camera_controller/free_camera/struct.FreeCamera.html)
- [bevy_remote docs](https://docs.rs/bevy_remote/latest/bevy_remote/)
- [BrpRequest docs](https://docs.rs/bevy/latest/bevy/remote/struct.BrpRequest.html)
- [Bevy diagnostic module docs](https://docs.rs/bevy/latest/bevy/diagnostic/index.html)
- [Bevy dev_tools module docs](https://docs.rs/bevy/latest/bevy/dev_tools/index.html)

## Current Zircon Status

Runtime primitives already present:

- `zircon_runtime::core::framework::picking` exports `PointerId`, `PointerLocation`, `PointerInput`, `PointerHits`, `HitData`, `HitTarget`, `Pickable`, `RayMap`, `PickingBackend`, `PrimitivePickingBackend`, `PickingEventState`, `PickingPointerEvent` from [mod.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/picking/mod.rs:1).
- Picking tests cover ray generation, ray map filtering, hit sorting, Pickable hover/blocking, primitive backend output, hover map, hover transition, click/release previous-hover behavior, drag/drop/scroll sequence, and cancel cleanup in [mod.rs](E:/Git/ZirconEngine/zircon_runtime/src/tests/picking/mod.rs:1).
- `zircon_runtime::core::framework::gizmos` exports `GizmoBuffer`, `GizmoCommand`, config, retained gizmo and render overlay extract bridge from [mod.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/gizmos/mod.rs:1).
- Gizmo tests cover buffer ordering/clear, disabled config, config defaults, retained buffer independence, overlay conversion, color policy, append into render overlay, shape command expansion, and retained-after-immediate ordering in [mod.rs](E:/Git/ZirconEngine/zircon_runtime/src/tests/gizmos/mod.rs:1).
- Runtime diagnostics already has `DiagnosticPath`, `DiagnosticMeasurement`, `DiagnosticStore`, snapshots and producers under [mod.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/diagnostics/mod.rs:1).

Still missing or not converged:

- No runtime-owned `core::framework::camera_controller`; current reusable camera work is still mostly render camera DTO plus private dynamic API controller at [camera_controller.rs](E:/Git/ZirconEngine/zircon_runtime/src/dynamic_api/camera_controller.rs:1).
- No runtime-owned `core::framework::remote`; current `core::framework::net::rpc` is diagnostics/report-oriented, not a Bevy Remote Protocol equivalent.
- `zircon_runtime_interface` has rich UI/resource contracts but no stable ABI DTO set for runtime picking/gizmo/camera/remote snapshots.
- Editor viewport still owns large portions of pointer route, overlay router, handle registry, transform handle math and camera navigation under [viewport](E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/mod.rs:1).
- Runtime picking/gizmos exist but are not yet wired as the single source of truth for editor selection, transform handles, dev overlays, remote inspection, or examples.

## Architecture Target

Runtime-owned public primitives:

- `zircon_runtime::core::framework::picking`: canonical pointer/ray/hit/event pipeline for UI, scene, editor handles, debug overlays and remote tools.
- `zircon_runtime::core::framework::gizmos`: immediate and retained debug geometry, config groups, overlay extract and later transform gizmo rendering.
- `zircon_runtime::core::framework::camera_controller`: reusable free/pan/orbit controllers with settings/state/input/action separation.
- `zircon_runtime::core::framework::remote`: protocol registry, request/response/error DTOs, method handlers, watch tokens, transport-neutral dispatch.
- `zircon_runtime::core::diagnostics`: Bevy-like diagnostic registry/history/smoothing, dev overlay feeds and producer registration.
- `zircon_runtime_interface`: ABI-stable snapshots/commands only. Rich Rust-only runtime APIs remain in `zircon_runtime`.

Naming rule: only actual network transport uses `server`; protocol and method layers use `RemoteProtocol`, `RemoteMethodRegistry`, `RemoteRequest`, `RemoteTransport`.

## Milestones

### M0 - Evidence and Gap Freeze

Status: update in progress.

Implementation:

- Freeze the source-reference matrix above as the minimum Bevy evidence set.
- Add a Zircon gap matrix with current runtime picking/gizmos status, missing camera/remote/interface status and editor ownership hotspots.
- Mark already-landed runtime picking/gizmos as “validate and cut over”, not “build from scratch”.
- Add source-guard tests or docs assertions that future planning does not reintroduce editor-only ownership for picking/gizmo/camera.

Testing stage:

- Docs-only validation with grep/source assertions if available.
- `cargo fmt --all --check`.

Exit criteria:

- Plan has Bevy source refs, public docs refs, Zircon current-owner refs and milestone gates.
- Every target capability has one runtime owner, one interface boundary rule and one editor cutover rule.

### M1 - Runtime Picking Contract Hardening

Status: partly implemented; hardening and integration remain.

Implementation:

- Audit current `framework::picking` against Bevy `Pickable`, `PointerHits`, `HitData`, `RayMap`, `PickingSystems` and pointer event vocabulary.
- Add explicit schedule/stage documentation: input collection, ray map rebuild, backend collection, hit merge/sort, hover resolve, event dispatch, post-event observers.
- Ensure `PointerHits` backend order and target priority semantics are stable and documented.
- Add multi-pointer, multi-camera and multi-viewport boundary cases beyond the current focused tests.
- Add a typed `PickingPipelineReport` for dev tools and diagnostics.

Testing stage:

- `cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1`.
- Boundary tests for same pointer across two viewports, two pointers in one viewport, inactive camera, non-hoverable but blocking targets, non-blocking pass-through, and backend priority ties.

Exit criteria:

- Runtime picking contract can be consumed without editor types.
- Editor-private route priority is representable as runtime `HitTarget`/priority, not duplicated logic.

### M2 - Picking Backend Integration and Editor Route Adapter

Status: runtime primitive backend exists; editor cutover remains.

Implementation:

- Introduce an adapter that converts existing editor renderable/scene-gizmo/handle candidates into runtime `PointerHits`.
- Keep editor viewport input as source of `PointerInput` only; runtime picking owns hit sorting and event dispatch.
- Make current editor route priority `HandleAxis > SceneGizmo > Renderable` a runtime test, then delete duplicate editor sorting once adapter is stable.
- Add optional picking debug feed built from `PickingPipelineReport`.

Testing stage:

- `cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1`.
- `cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1`.
- Regression tests for select entity, hover scene gizmo, drag handle and click-through non-blocking overlays.

Exit criteria:

- Editor `ViewportOverlayPointerRouter` becomes an input/output bridge.
- No editor-only picking algorithm remains as the authoritative route resolver.

### M3 - Runtime Gizmo API Hardening

Status: partly implemented; Bevy-like config/render integration needs completion.

Implementation:

- Audit `framework::gizmos` against Bevy immediate `Gizmos`, `GizmoBuffer`, config groups, retained `GizmoAsset` and overlay extraction.
- Add missing common shapes only if they are needed by editor/runtime examples: arrow, arc, grid, frustum, capsule.
- Add config group store if multiple independent debug tool families need independent enable/line/render-layer policy.
- Keep `render::overlay` as extract/output layer, not as a growing editor dump.
- Add diagnostics counters: command count, extracted line count, skipped-disabled count, retained/immediate split.

Testing stage:

- `cargo test -p zircon_runtime --lib gizmos --locked --color never --jobs 1`.
- Snapshot tests for config group enable/disable, line width, render layer, depth bias, color override, retained transform and clear timing.

Exit criteria:

- Runtime systems, editor and dev tools can draw gizmos without depending on editor handle code.
- Overlay DTO is an extract target, not the canonical gizmo API.

### M4 - Runtime Transform Gizmo Primitive

Status: missing as runtime primitive; editor handle tools exist.

Implementation:

- Add `framework::gizmos::transform` or `framework::transform_gizmo` only after module-boundary review.
- Model settings/state from Bevy: mode translate/rotate/scale, space world/local, axis X/Y/Z/View, snap increments, screen-scale policy, camera selection and focus target.
- Consume M1/M2 picking events for hover, press, drag and release.
- Output transform delta commands, not editor undo commands. Editor adapts deltas into undoable operations.
- Keep rendering independent of Slint; output runtime gizmo/overlay commands.

Testing stage:

- Axis hover math, ray-plane intersection and ring distance tests.
- Translate/rotate/scale snap tests.
- Local/world space transform tests.
- Editor undo adapter tests after cutover.

Exit criteria:

- Move/rotate/scale handle math lives in runtime.
- Editor `HandleToolRegistry` is reduced to mode selection, selection binding and undo adapter.

### M5 - Reusable Camera Controllers

Status: missing as public runtime primitive.

Implementation:

- Add `zircon_runtime::core::framework::camera_controller`.
- Provide `FreeCameraController`, `PanCameraController` and `OrbitCameraController`.
- Split each controller into settings, state, input, output delta and apply step.
- Migrate private dynamic runtime controller to use the public controller.
- Support keyboard/mouse/touch-compatible actions, scroll zoom/speed, damping/friction, viewport constraints, cursor grab/confine intent and focus gating.
- Keep Bevy divergence explicit: Zircon may not use Bevy ECS components, but it must preserve the same settings/state/system separation.

Testing stage:

- Free camera movement, yaw/pitch clamp, scroll speed, friction decay.
- Pan/orbit viewport-to-world conversion.
- Dynamic API smoke test proving old frame output does not regress after migration.
- Editor viewport camera navigation regression.

Exit criteria:

- Runtime demo, editor viewport and remote/dev tools can reuse the same camera controller logic.
- No private dynamic API camera controller is extended further.

### M6 - Diagnostics and Dev Tools Completion

Status: diagnostics store exists; overlay/dev-tool consumers need completion.

Implementation:

- Audit current `core::diagnostics` against Bevy diagnostic path/measurement/store/history/average/smoothed/enable model.
- Add missing history limit, smoothing or producer registration behavior only where current code lacks it.
- Add dev feeds: FPS overlay, diagnostics overlay, picking debug overlay, render debug overlay and infinite grid as consumers of runtime primitives.
- Picking debug reads `PickingPipelineReport`; gizmo/render debug reads `GizmoExtractReport`; overlays read `DiagnosticStore`.
- Do not let dev overlays mutate simulation state.

Testing stage:

- `cargo test -p zircon_runtime --lib runtime_diagnostics --locked --color never --jobs 1`.
- Overlay feed tests for enable/disable, missing diagnostic path, smoothed/average value selection and no-simulation-mutation guarantee.

Exit criteria:

- Dev tools observe runtime primitives; they do not duplicate picking/gizmo/camera logic.

### M7 - Remote Protocol Core

Status: missing.

Implementation:

- Add `zircon_runtime::core::framework::remote`.
- Define `RemoteRequest`, `RemoteResponse`, `RemoteError`, `RemoteId`, `RemoteMethodRegistry`, `RemoteMethodHandler`, `RemoteSession`, `RemoteWatchToken`.
- Match Bevy’s split: protocol registry and request processing independent from HTTP transport.
- Initial built-ins:
  - `world.list_components`
  - `world.get_components`
  - `world.query`
  - `world.spawn_entity`
  - `world.despawn_entity`
  - `world.insert_components`
  - `world.remove_components`
  - `world.mutate_components`
  - `world.reparent_entities`
  - `resource.list`
  - `resource.get`
  - `resource.insert`
  - `resource.remove`
  - `resource.mutate`
  - `registry.schema`
  - `schedule.list`
  - `schedule.graph`
  - `rpc.discover`
- Map unsupported methods to structured `RemoteError` with stable code and diagnostic path.
- Keep `framework::net::rpc` as transport/diagnostic/report layer or migrate it under remote; do not leave two competing RPC systems.

Testing stage:

- JSON round-trip for request/response/error.
- Method discovery and duplicate registration tests.
- World/resource read-only methods first; mutation methods behind explicit capability settings.
- Watch token tests for add/change/remove once runtime change tracking is stable.

Exit criteria:

- A remote editor can inspect and mutate supported runtime world/resource/schema through protocol methods, without dynamic-session private commands.

### M8 - Runtime Interface / ABI DTOs

Status: missing for these primitives.

Implementation:

- Add stable DTOs to `zircon_runtime_interface` only for cross-ABI needs:
  - picking event snapshot and hit snapshot
  - gizmo command/extract snapshot
  - camera controller command/state snapshot
  - remote request/response/error frame
- Keep ABI DTOs versioned and smaller than rich Rust APIs.
- Add conversion functions in runtime, not in editor.
- Preserve existing UI/resource interface contracts; do not mix generic runtime picking with UI-only surface picking unless explicitly adapted.

Testing stage:

- ABI layout/version tests.
- Serialization/backward compatibility tests.
- Dynamic session remote command smoke test.

Exit criteria:

- Runtime primitives can cross crate, plugin and dynamic-library boundaries through stable DTOs.

### M9 - Editor Cutover

Status: largest remaining integration risk.

Implementation:

- Convert editor viewport input to runtime `PointerInput` and camera-controller input.
- Replace editor route sorting with runtime picking outputs.
- Replace move/rotate/scale handle math with runtime transform gizmo deltas.
- Replace editor overlay data construction with runtime gizmo buffers and retained gizmos.
- Keep editor-owned behavior only where it is truly editor UX: selection model, undo grouping, toolbar state, inspector focus, Slint view model.
- Hard cutover after tests: delete stale editor-only route/math modules rather than leaving long-term facades.

Testing stage:

- `cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1`.
- Regression workflows: select entity, hover gizmo, drag move/rotate/scale, undo/redo, camera pan/orbit/free move, scene gizmo click, renderable click, non-blocking overlay click-through.

Exit criteria:

- Editor no longer owns picking/gizmo/camera controller core algorithms.
- Editor is a consumer/adapter of runtime primitives.

### M10 - Bevy Completion Acceptance

Status: final milestone.

Implementation:

- Add examples/smoke tests aligned to Bevy capability categories:
  - simple picking
  - renderable/mesh picking
  - picking debug overlay
  - many pointers / many backends
  - immediate gizmos
  - retained gizmos
  - transform gizmo
  - free camera
  - pan/orbit camera
  - remote protocol discovery/query/mutation
  - diagnostics/FPS overlay
- Update docs for runtime primitives, editor cutover and intentional divergences from Bevy.

Testing stage:

- `cargo check -p zircon_runtime_interface --lib --locked --color never --jobs 1`
- `cargo test -p zircon_runtime --lib picking --locked --color never --jobs 1`
- `cargo test -p zircon_runtime --lib gizmos --locked --color never --jobs 1`
- `cargo test -p zircon_runtime --lib runtime_diagnostics --locked --color never --jobs 1`
- `cargo test -p zircon_runtime --lib remote --locked --color never --jobs 1`
- `cargo test -p zircon_editor --lib viewport --locked --color never --jobs 1`
- `cargo test --workspace --locked --color never --jobs 1`

Exit criteria:

- Public runtime primitives cover Bevy first-party picking, gizmos, camera controller, remote, diagnostics and dev-tools capability categories at the level listed in this plan.
- Editor tools run on runtime primitives.
- Remote editor uses remote protocol rather than editor-only or dynamic-session-private commands.
- Docs and tests record Bevy source refs, Zircon intentional divergence and acceptance evidence.

## Recommended Execution Order

1. M0: freeze evidence and update docs/status.
2. M1-M2: harden picking and cut editor route to runtime picking.
3. M3-M4: harden gizmos and land transform gizmo primitive.
4. M5: land camera controllers and replace private dynamic controller.
5. M6: make diagnostics/dev tools observers of those primitives.
6. M7-M8: add remote protocol and ABI DTOs after runtime contracts are stable.
7. M9-M10: editor cutover and full Bevy-completion acceptance.

## Risks

- Existing runtime picking/gizmos may be concurrent work from another session. Before modifying those modules, re-read recent `.codex/sessions` and coordinate ownership.
- Remote protocol depends on Zircon reflection/world/resource maturity. Start with list/query/read methods, then mutation/watch.
- Transform gizmo cutover can break editor undo semantics. Runtime should output deltas; editor should own undo grouping.
- Runtime interface must not become a dumping ground for rich internal APIs. ABI DTOs need versioning and strict scope.
- Do not keep long-term compatibility shims after editor cutover unless a dynamic ABI version requires them and has an explicit removal condition.
