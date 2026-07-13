---
status: completed
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
recorded_at: 2026-07-13
milestone: M1
slice: 1.4
related_code:
  - zircon_editor/src/core/asset/source_authority.rs
  - zircon_editor/src/core/asset/type_registry/
  - zircon_editor/src/core/commands/asset_write_target.rs
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/
tests:
  - zircon_editor/src/tests/editor_asset_type_registry/source_authority.rs
  - zircon_editor/src/tests/editor_asset_type_registry/consumer_projection.rs
  - zircon_editor/src/tests/commands/descriptor_when.rs
---

# Editor09 M1.4 Source authority 与写操作双层守卫产出

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 更新日期 | 完成项目、当前证据与未完成门禁 |
|---|---|---|---|---|
| M1 | 1.4 A/B source authority + command when/dispatch write guard | `COMPLETED` | 2026-07-13 | typed source/write policy、snapshot projection、creation/mutation command target metadata、`AssetWritable` when 与实际 dispatch fail-closed 拒绝已实现。Windows no-run 门自然退出 0并生成 r6 binary；完整 registry 24/24（含 project create 与 package/builtin/lib/mem direct dispatch 拒绝）及 descriptor when 1/1 通过。ProjectAuthority-backed Manager fixture 硬切回归另由当前源码 binary 取得 83/83。M1.4 功能切片及其上行失败回收完成；Editor09 M1 全测试阶段继续执行。 |

### 已落地实现

- `AssetSourceWritePolicy::{ProjectOnly, ReadOnly}` 是 definition 的 required scalar field，默认
  `ReadOnly`；registry 记录字段 owner 并拒绝第二 owner。所有 built-in definition 与 custom
  `define` 显式形成 authoring policy，serde roundtrip 不丢失。
- `AssetSourceAuthority` 只从 `ResourceLocator`/canonical source root 或显式 derived/transient
  构造。`res://` 仅在 `ProjectOnly` 下可写；package/builtin/library/derived/transient 始终只读。
- creation template 自动给其 command 关联 `asset_type` + `target_folder`；context command 默认
  `ReadOnly`，只有 `.with_mutation_access()` 才关联 `asset_type` + `asset_locator`。一个 operation
  出现冲突 target 直接返回 typed registration error。
- `EditorCommandDescriptor.asset_write_target` 可序列化；`effective_when()` 自动与
  `WhenClause::AssetWritable` 合取。`CommandEvalCtx` 携带 fail-closed write access，Browser 当前选中
  locator 负责 UI/menu projection。
- `invoke_operation` 在分派 event 前重新从真实 invocation arguments 解出 type + locator，查询当前
  materialized registry 并计算 authority。UI、menu、CLI、remote 共用该路径；只读 source、缺参、
  空参、未知 type 和非法 scheme 都失败，不能通过直接 control request 绕开 UI。
- Browser item/selection 的 source authority 来自 `AssetTypeProjectionSnapshot.source_write_policy`
  与 locator；没有添加平行可写布尔值或 URI prefix string truth。

### 当前验证证据

- 生产编译：Windows 受管 job `2da44310d00e4ca39b24a163ee7a48d2`，
  `cargo check -p zircon_editor --lib --locked --jobs 1` 退出码 0；日志
  `.codex/tmp/editor09-m1-4-production-check-r2-20260713.log`。
- Windows 受管 Cargo job `c5f0129d36d8445c94820be243d70357`：
  `cargo test -p zircon_editor --lib --no-run --locked --jobs 1` 自然退出 0，生成当前 binary
  `.codex/tmp/zircon_editor-editor09-m1-4-source-authority-r6-20260713.exe`；日志
  `.codex/tmp/editor09-m1-4-source-authority-compile-r6-20260713.log`。
- r6 binary：`tests::editor_asset_type_registry` 24 passed、0 failed，覆盖 source authority matrix、
  source policy single-owner serde、Browser project authority、project create dispatch，以及
  package/builtin/library/transient 直接 CLI invoke 拒绝和 unsupported derived scheme 拒绝；
  `asset_write_target_adds_a_serializable_writable_when_guard` 1 passed、0 failed。
- consumer projection 的最初 RED 是 `visible_assets.find(res://ui/main.zui)` 夹具把嵌套资源错误记为
  root direct asset，而生产过滤按 locator 的真实 parent `res://ui` 工作。夹具改成 canonical folder
  tree、选择 `res://ui`，并把 validate context command 明确标记 mutation；r6 全 registry 24/24 证明
  project authority 与 mutation dispatch 当前通过，没有修改生产过滤语义或绕过断言。
- scoped `rustfmt --edition 2021` 与 `git diff --check` 通过（仅既有 LF/CRLF 提示）。

### 跨计划失败交接

- Text01 font decoration 缺 display size（已修复回传）：
  [`fixed-2026-07-13-font-decoration-display-size-argument.md`](fixed-2026-07-13-font-decoration-display-size-argument.md)。
- Editor12 native plugin 测试旧 `RuntimeTargetMode` 路径（已修复回传）：
  [`fixed-2026-07-13-native-plugin-runtime-target-mode-test-path.md`](fixed-2026-07-13-native-plugin-runtime-target-mode-test-path.md)。
- Render01 `RenderFramework::register_pipeline_asset` hard cut 与 editor test double（已修复回传）：
  [`fixed-2026-07-13-render-framework-pipeline-registration-test-double-migration.md`](fixed-2026-07-13-render-framework-pipeline-registration-test-double-migration.md)。
- Frameworks02 lifecycle observer 导入硬切五处 E0432（已修复回传）：
  [`fixed-2026-07-13-runtime-module-lifecycle-observer-import-cutover.md`](fixed-2026-07-13-runtime-module-lifecycle-observer-import-cutover.md)。
- Frameworks05 `RuntimeProfileId` consumer canonical-owner 硬切（已修复回传）：
  [`fixed-2026-07-13-runtime-profile-id-consumer-cutover.md`](fixed-2026-07-13-runtime-profile-id-consumer-cutover.md)。
- Runtime02 weak `EditorManager` caller lifetime（已修复回传）：
  [`fixed-2026-07-13-editor-manager-weak-runtime-caller-lifetime.md`](fixed-2026-07-13-editor-manager-weak-runtime-caller-lifetime.md)。
- ProjectAuthority Manager 工程夹具与 renderable template schema（已修复回传）：
  [`fixed-2026-07-13-project-authority-test-fixture-cutover.md`](../../../zircon_runtime/runtime/02/fixed-2026-07-13-project-authority-test-fixture-cutover.md)。

### 未完成门禁

M1.4 已完成，不再重复开发。ProjectAuthority/Manager 上行回归已由当前源码 binary 自然取得
`83 passed / 0 failed`，本切片涉及的四条跨计划编译/生命周期阻断也均已回传 fixed。本计划接下来执行
Editor09 M1 全量 `cargo test -p zircon_editor --lib --locked -- --test-threads=1`。首次完整门已启动，
但在测试执行前被 Runtime13 `HostRegistry` generational-handle consumer 未原子迁移的 E0308/E0599
阻断并已交接；因此 M1 与 Editor09 继续 `in_progress`，在完整测试阶段形成自然 summary 前不触发
session milestone closeout。
