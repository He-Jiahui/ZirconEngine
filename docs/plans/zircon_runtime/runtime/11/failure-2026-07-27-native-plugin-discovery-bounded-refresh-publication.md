---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: native-plugin-discovery-bounded-refresh-publication
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/manifest_completion/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/core/runtime/tasks
tests:
  - cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib native_plugin --locked --jobs 1 -- --nocapture --test-threads=1
  - repeated watcher burst, stale-result suppression, cancellation, shutdown, and UI-frame no-I/O fixtures
---

# Runtime11: Native Plugin Discovery Bounded Refresh Publication Handoff

## 来源执行者

- Origin plan: `docs/plans/zircon_editor/editor/12-plugin-management.md`.
- Fixing plan: `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`.
- Lifecycle key: `native-plugin-discovery-bounded-refresh-publication`.
- Coordination: Runtime11 owns the bounded task, cancellation, coalescing, terminal observation, and immutable publication contract. Frameworks04 keeps ownership of the existing native-loader discovery/load projection implementation and its active leased files. Editor12 consumes only published immutable snapshots and does not create an editor-local scanner, worker thread, cache, or compatibility path.
- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：Editor12 原生插件目录投影与 UI 请求路径审计。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：同步原生发现/加载需要 Runtime11 的有界任务、取消、终态观察和不可变发布合同，不能在 Editor UI 或 Frameworks04 加载投影中临时复制。

## 失败现象与复现证据

The Editor plugin-management request path still invokes `NativePluginLoader` synchronously in multiple user-visible operations:

- `enablement/native.rs` calls `discover` while changing a project plugin selection.
- `manifest_completion/native.rs` calls `discover` while completing a project manifest.
- `native_registration/manager.rs` calls `load_discovered_editor` while collecting editor registrations.
- `status/native.rs` calls `load_discovered_all` while building the plugin status report.

The Runtime implementation confirms that these calls perform work in the caller path. `discover.rs` delegates to `NativePluginDiscoveryAuthority`; `authority.rs` initializes/scans the root, drains watcher events, parses manifests, and builds a fresh `NativePluginLoadReport` while holding the root-state mutex. `load_discovered.rs` then may probe and load native libraries on that same request path.

The authority maintains an internal generation, but `NativePluginLoadReport` exposes neither an immutable generation-bearing discovery publication nor a submit/poll/cancel contract. `discovery_generation` is a separate lookup, so a caller cannot atomically bind a report to the generation it read. This leaves status refresh, manifest completion, registration, and enablement exposed to filesystem I/O or native-load work on the UI request path.

## 最低共享层根因

Native discovery has a cache and watcher but no Runtime11-owned bounded execution/publication layer. Its snapshot is private mutable authority state and its public output is a request-local report. Consequently Editor12 has no legal nonblocking way to ask for a refresh, observe one terminal result, discard stale work, or consume an atomically published generation.

## 架构修复验收

Runtime11 and Frameworks04 must jointly introduce one Runtime-owned discovery refresh service, without changing the native loader's authority or projection ownership:

1. A caller submits a root-scoped refresh intent and returns immediately. The service coalesces repeated watcher/editor intents by canonical root and preserves only the newest relevant generation/scope; it never creates one unbounded task per event.
2. The I/O/task lane performs collection and manifest parsing away from the UI request path under count, bytes, and time budgets. Cancellation, deadline expiry, panic, and shutdown each reach the existing Runtime11 terminal-observer path exactly once.
3. Completion atomically publishes an immutable `Arc` discovery snapshot that contains the canonical root, monotonic generation, package manifests, diagnostics, and a stable input identity. Readers obtain the latest completed publication without rescanning or observing mutable loader state.
4. A stale task cannot overwrite a newer publication. A failed task preserves the last good snapshot and publishes bounded diagnostics/telemetry instead of clearing the catalog.
5. Dynamic-library activation remains an explicit Runtime/Frameworks04-controlled operation. This handoff does not permit Editor UI code to call `Library::new`, invoke plugin entries, or move arbitrary activation side effects onto a generic worker. Its completion result must be published through the same generation-bound contract when the selected activation path is ready.
6. Editor12 migrates the four synchronous request paths only after this contract exists: UI handlers submit/observe a refresh ticket and render the last published snapshot, while explicit native activation consumes the Runtime publication. No editor-local polling loop, duplicated cache, legacy re-export, or compatibility fallback is allowed.

### 验收标准

- A focused Runtime test proves submit is nonblocking and a UI-thread fixture performs no directory enumeration, manifest read, library probe, or native entry invocation while requesting or reading a refresh.
- A burst of create/modify/remove events for one root has bounded queue depth and produces one newest published generation; superseded results never replace it.
- Publication snapshots bind generation and payload atomically. Readers either receive the previous full snapshot or the next full snapshot, never a mixed report/generation pair.
- Missing roots, malformed manifests, watcher errors, cancellation, deadline expiry, task panic, and shutdown preserve last-good data and each produce one terminal outcome with bounded diagnostics.
- Frameworks04's native projection/load behavior remains covered by its focused tests; Runtime11 does not duplicate its parser, loader, or registration projection.
- After the migration, Editor12 focused tests prove status, manifest completion, native registration, and native-aware enablement read only the published snapshot on their UI request paths.
- Run the two Cargo commands declared in front matter through `tools/zircon-session.ps1` only after an immutable source snapshot and FIFO admission; add the burst/cancellation/shutdown fixture results to the fixing-plan output record.

## 禁止临时方案

- Do not put filesystem scans, watcher draining, manifest parsing, ABI probing, `Library::new`, or plugin entry invocation behind an Editor UI cache or a fire-and-forget thread.
- Do not expose a mutable `NativePluginLoadReport`, split generation from report identity, or accept stale completion by timestamp heuristics.
- Do not add aliases, compatibility shims, a second scanner, an unbounded channel, periodic polling, or caller-specific bypasses.
- Do not absorb Frameworks04's active native-loader projection changes into this Runtime11 task contract.

## 修复结果与回传

Open state: Runtime11 has implemented a folder-backed bounded refresh contract with root admission, latest-wins generations, immutable last-good snapshots, explicit cancellation/deadline/shutdown terminals, typed budget errors, and terminal-observer admission that remains capped after completion. Independent review found that the Frameworks04 collector still builds public `Vec`/`String` payloads and self-reports byte counters before Runtime11 validates them; a Runtime11-owned metered streaming sink or equivalent collector-contract migration is required before these values can claim a real RSS bound. The Frameworks04 authority collector and Editor12 request-path migration remain outside this Runtime11 source scope; no Cargo acceptance, fixed artifact, or lifecycle return is claimed by this handoff.

The managed focused reservation `c868389ecd794d728f3cc8c7143ef67d` was consumed as job `87a16b87c245479daffe95b5a3d727fe` / run `636fe30611ce45fb96f203222f1a27b8` and naturally terminated `exit 101` at 2026-07-29 13:41 CST. `cargo +1.94.1 test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1 -- --nocapture --test-threads=1` never reached test execution: the `zircon_runtime` lib-test compile stopped on Runtime08 `SceneBindingGenerations` visibility (`E0365` + `E0603` x2), Runtime11 operation-test visibility/type inference (`E0422` x2, `E0624` x2, `E0282`), and Text09 `text/parallel/raster_pool.rs:447` moved-options `E0382`. The operation-owned diagnostics have been repaired in current source; Runtime08 and Text09 are separate fixing owners. This terminal is compile-blocker evidence only, not a Native Refresh RED or GREEN result; any retry requires a fresh FIFO reservation after those lower compile blockers return.

Current architecture state: Frameworks04 node `1129296` (`native-discovery-metered-collector-contract`) owns the required before-allocation collector/sink migration. Runtime11 retains task admission, cancellation, terminal ticketing, and immutable publication, but cannot claim a real resource bound from post-hoc collector counters. Text09 remains the active lower lib-test compile owner; no native focused retry, fixed artifact, or commit is claimed before both lower returns and fresh FIFO admission.

2026-08-27 source progress: composition-order and Unreal review corrected the discovery owner model. Discovery can run before Core composition and therefore remains a process-lifetime authority; `NativePluginHostHandle` owns product-generation dynamic-library lifetime. Runtime11/Frameworks04 now expose an explicit root-resolution setup phase, a prepared-root nonblocking refresh ticket, and a no-filesystem immutable last-good snapshot read through `plugin::native::discovery`. Blocking wait remains crate-private, dynamic activation remains explicit, and async admission reclaims terminal authority entries. The focused static contract audit passes 6/6 and the broader host adapter audit introduces no new failure beyond its two existing structure drifts. Editor12 has not migrated its four request paths, managed Cargo has not run, and the process-default three-pool materialization still requires the Windows profile recorded in `docs/plans/optimize/zircon_runtime/11/2026-08-27-native-plugin-discovery-authority-research.md`; this failure therefore remains open and no fixed artifact or accepted milestone is claimed.

2026-08-27 Editor migration gate: current source confirms that `ProjectPreflightReceipt` is intentionally data-only and cannot own a live refresh ticket or plugin/runtime capability. The required owner is a companion project/session activation ledger: preflight resolves the prepared native root, the admitted session owns ticket cancellation and terminal observation, status/manifest/enablement read only its latest complete discovery snapshot, and native registration separately consumes the selected generation through explicit load/activation while retaining the product-generation `NativePluginHostHandle`. A discovery snapshot is not a loaded-plugin report. Migrating any of the four call sites before this ledger exists would create a cold-empty catalog fallback or bypass generation-bound activation, so no partial Editor call-site edit is admitted by this handoff.

## 产出记录与时间

### 2026-07-27 11:52 CST

- 状态：`resolving_failure`，交接记录已建立，保持 `open`。
- 已完成：以当前源审计确认四类 Editor 用户请求同步进入 native discovery/load；将修复责任路由到 Runtime11，并明确 Frameworks04 的加载投影所有权不转移。
- 证据：`NativePluginDiscoveryAuthority` 已有 root cache/watcher/generation，但 discovery report 缺少 generation-bound immutable publication，且 `load_discovered_*` 在调用方路径继续执行。
- 后续：Runtime11 落地有界刷新/发布合同后，Editor12 取得新的精确租约，迁移四个 UI 调用点并以受管 Cargo、独立审查和 `fixed-*` 生命周期回传验收。

### 2026-07-27 11:59 CST

- 状态：协调器已导入 open 节点 `926663`，生命周期键为 `native-plugin-discovery-bounded-refresh-publication`；本记录继续保持 `resolving_failure`，未使用 `blocked`。
- 验证：failure-handoff 技能校验器对本文件结果为 `local_errors=0`；协调器 `failure open` 已返回本节点和八条精确相关路径。
- 范围：全仓审计仍有既有 cycle/schema 诊断，均非本记录引入且不在本会话租约内；本次未修改其余 failure artifact。
