---
status: in_progress
owner_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
recorded_at: 2026-07-12
related_code:
  - zircon_editor/src/core/export
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export
tests:
  - cargo test -p zircon_editor --lib --locked --offline
  - zircon_editor test binary partition filters
---

# Editor 15 M1 full lib 分区验证记录

## 验证边界

M1 计划要求的 `cargo test -p zircon_editor --lib --locked --offline` 在共享 Windows target 中运行
904 秒仍未进入可见 test summary。为区分 Cargo 锁与实际测试耗时，又直接执行由同一当前源码 focused
run 生成的 `zircon_editor-0af59361f300b435.exe`，该测试二进制同样在 904 秒内未返回 harness summary。

这不是已定位的跨计划产品失败，因此本记录作为 Editor 15 测试阶段的环境与分区证据保存，不使用
`failure-*` 生命周期。分区后发现的具体功能失败分别进入其最低 owner 的 `failure-*` 交接。

## 已有对照

- `cargo test -p zircon_editor core::export::tests --lib --offline -- --nocapture`：13/13 通过。
- `cargo test -p zircon_runtime_interface --locked --offline`：240/240 通过。
- 受影响 Python 导出测试：262/262 通过。

## 产出记录与时间

| 时间 | 状态 | 产出 |
| --- | --- | --- |
| 2026-07-12 22:24 +08:00 | 未通过，已归档 | Cargo full lib 与当前测试二进制直接执行均在 904 秒无最终 summary；M1 full editor gate 保持未完成。 |
| 2026-07-12 23:10 +08:00 | 分区定位进行中 | `core::` 64/65，唯一失败已写入 Editor10 既有功能交接；`scene::` 24/24；`paint_template_nodes::` 666/694，28 项已写入 `editor_layout/15/failure-2026-07-12-retained-painter-component-contract-regressions.md`。来源计划继续拆分剩余 UI/tests，不在 Editor15 修跨功能失败。 |
| 2026-07-12 23:22 +08:00 | 文本分区已路由 | `paint_text::{font,raster,blend,sync}` 共 36/36；`paint_text::tests` 15/17，2 项精确失败已追加到既有 `editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md`。 |
| 2026-07-12 23:28 +08:00 | Retained UI 投影分区已路由 | `ui::retained_host::ui::` 94/102；6 项 component showcase 追加到既有 Editor UI 06 交接，1 项 Welcome mount 追加到既有 Editor UI 08 交接；2 项 Build/Export pane 投影由 Editor15 自有切片继续处理。 |
| 2026-07-12 23:45 +08:00 | Editor15 自有旧 fixture 已硬切，验证待新 binary | 两项 Build/Export pane 失败源于 fixture 只填旧 `target.strategies/status` 文本、未提供新权威 `ExportWizardPanelViewModel`，因此生成 unavailable plan。测试已改为分别注入 ready typed plan 与 SourceTemplate typed plan；Cargo focused 命令在共享编译拥塞下 904s 未生成新 test binary，故暂不声明通过。 |
| 2026-07-13 00:24 +08:00 | 平台门恢复，Editor no-run 仍未形成 binary | 当前 `zircon_runtime` test binary 中 Render18 SSS profile table 1/1、`ZR_EXPORT_CONTRACT_PLATFORM=windows` 平台策略 1/1；两项 handoff 已按 coordinator return 回传为 fixed。Layout owner 的当前源码 Editor no-run 受管 job `9570dd7c069e45cab667fbc843dba2fa` 最终 exit 101 且 ephemeral target 已释放、未留下 test binary；本计划不臆测未捕获的编译错误，继续等待可审计的当前源码复现。 |
| 2026-07-13 00:27 +08:00 | Layout 分区继续路由 | 旧 Editor binary 的 `ui::layouts::` 单线程分区 73/76；3 项均为 Asset Browser/Workbench chrome 投影与度量，已追加既有 Layout15 open handoff，Editor15 未增加局部修补。 |
| 2026-07-13 00:29 +08:00 | UI 独立分区通过 | 旧 Editor binary 单线程：`ui::workbench::` 50/50；animation editor 19/19；preferences 13/13；asset editor 7/7；template runtime 3/3；component registry 1/1；material editor 15/15。该证据只覆盖对应分区，不替代当前源码 full gate。 |
| 2026-07-13 00:31 +08:00 | Text draw 分区路由 | `paint_text::draw::layout` 25/28，3 项 subpixel/grapheme 失败已追加既有 Editor UI 03 handoff；与 `paint_text::tests` 的 2 项合计为 Text03 原有 5 项 owner 集合。 |
| 2026-07-13 00:44 +08:00 | Full gate 长耗时来源确认 | `ui::retained_host::app::tests` 聚合 304s 未完成；代表 exact `asset_browser_pointer_drop_applies_real_payload_to_showcase_asset_field` 单项 1/1 但耗时 54.48s。说明旧 binary 的 904s full timeout至少包含昂贵 host 初始化/测试体耗时，不能视为 harness 死锁；后续继续按功能分区并使用当前源码 binary复验。 |
| 2026-07-13 02:14 +08:00 | Editor15 自有 typed-plan 回归通过 | 当前源码 binary（01:26:25）精确执行 `build_export_wizard_panel_uses_typed_plan_instead_of_target_summary_text`：1/1，38.50s；证明 Build/Export pane 使用 typed `ExportWizardPanelViewModel`，旧 `target.strategies/status` 摘要文本不再拥有导出计划权威。 |
| 2026-07-13 02:16 +08:00 | Layout15 当前源码复验仍有 1 项 | `paint_template_nodes::` 当前已扩大为 696 项：695 passed / 1 failed / 2416 filtered out，96.24s。原 28 项已消除；唯一 tooltip 箭头色/几何采样失败已追加到 `editor_layout/15/failure-2026-07-12-retained-painter-component-contract-regressions.md`，因此不回传 fixed。 |
| 2026-07-13 02:22 +08:00 | Layout15 owner 完成修复并回传 | Layout15 owner 在最低共享层完成 28px metric/icon/state/palette/table 收敛，并将 tooltip 断言改为验证语义绘制区域而非抗锯齿边界固定像素；当前源码分区 `paint_template_nodes::` 696/696、`ui::layouts::` 76/76。canonical 交接已由 coordinator 迁回 `editor/15/fixed-2026-07-13-retained-painter-component-contract-regressions.md`；Editor15 不再保留该 open failure。 |
| 2026-07-13 02:36 +08:00 | Editor15 自有当前 binary focused 矩阵全绿 | 当前源码 01:26 binary：`core::export::tests` 13/13；Build/Export pane projection 6/6；wizard session 6/6；build-export actions 5/5；job queue 2/2；output-folder host boundary 4/4。该 binary 之后无 Editor15 export owner 源文件变化，证明 M1.1/M1.2 自有路径当前全绿；不替代其他功能 owner 与 full lib 门。 |
| 2026-07-13 02:38 +08:00 | 跨功能失败按 owner 再复现 | 同一 current-source binary 精确复验：Editor UI 03 `paint_text::tests` 15/17、`paint_text::draw::layout` 25/28；Editor UI 06 component showcase/reference/structure 合计 3/5、0/1、0/2；Editor UI 08 Welcome mount 0/1（22 != 31）。失败集合与既有 UI03/UI06/UI08 canonical `failure-*` 记录一致，Editor15 未恢复旧字体名、退役 DTO 或旧 mount 节点，只在本来源测试记录保留向上门证据。 |
| 2026-07-13 02:42 +08:00 | 旧 full run 资源停滞已终止 | 01:26 binary 的受管 full run 持续约 74.7 分钟后有 5547 个线程、CPU 采样连续为 0，且源码已在 01:33 后变化；按系统化诊断判定该进程既不是 current-source gate，也无法继续提供有效进度，已终止并将 coordinator job `96ee7a04b19a49048cd08a3cdac2f99e` 记为 exit 1/released。此项是测试执行资源失败，不计产品通过。 |
| 2026-07-13 02:46 +08:00 | 最新源码 official validator 被 Render 11/18 半迁移编译阻断 | Windows validator `cargo test -p zircon_editor --locked --verbose`（job `d42bb4a651604962a3cc678b4ef663bb`）在 `zircon_runtime` 编译阶段 exit 101：`mesh_pipeline_cache/construct.rs:82` 的 `queue` 未进入 `MeshPipelineCache::new` 签名（E0425），`forward_shadow_receiver.rs:139` 从临时 `LightmapGpuBindings` 借用 entries（E0716）。两文件在 02:38/02:39 由并行 Render 11 EL-M3 lightmap + Render 18 forward volumetric/OIT 接线改动，最低 owner 不属于 Editor15；日志 `.codex/tmp/editor15-m1-current-full-20260713-0243.log`。本计划保留 focused 全绿证据并等待功能 owner 收敛，不加兼容参数或局部绕过。 |
| 2026-07-13 03:18 +08:00 | Render11 编译阻断已由对应功能模块修复 | Runtime05/Render11 owner 完成 `MeshPipelineCache::new` queue 构造链与 `lightmap_bindings` lifetime 修复；Windows official validator job `9c0bba0554b042c2b3c5a139a8bb10a7` 成功完成 `zircon_runtime` / `zircon_editor` test-profile 编译（13m51s），原 E0425/E0716 未再出现。Render11 failure 保持 open，等待其 focused/no-run 完整回传；Editor15 不复制修复。 |
| 2026-07-13 03:47 +08:00 | 最新源码 full harness 仍由 Editor14 线程耗尽阻断 | 同一 official validator 进入当前 test binary 后再次增长到 5547 threads；终止前线程状态为 5541 `Wait/Unknown`、5 `Wait/UserRequest`、1 `Wait/EventPairLow`，CPU 60 秒仅增加 0.015625 秒，harness 无 summary。为避免继续占用系统资源终止 child，Cargo `0xffffffff`、validator/job exit 1/released；日志 `.codex/tmp/editor15-m1-post-render11-fix-20260713-0304.log`。新证据已追加到 `editor/14/failure-2026-07-12-editor-full-gate-thread-exhaustion.md`；Editor15 M1 仍不关闭，但不在导出模块修线程调度。 |
| 2026-07-13 04:16 +08:00 | 线程耗尽最低共享层已转交 Runtime11 | 同一 binary 以 `--test-threads=1 --nocapture` 诊断时从 8 增至 4091 threads，创建批次与当前机器 Runtime `TaskPoolThreadCounts.total_threads=16` 对齐；`tests::host::manager::` 单线程窄分区也达到 549 threads，最终 62 passed / 17 failed / 3035 filtered out（50.78s）。根因边界已收窄到 `CoreRuntime::new -> TaskPools::default` 与 asset worker 独立 `spawn_named_thread` 双预算/生命周期，canonical failure 已写入 `docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md`；Editor15 仅保留来源门阻塞，不加 test-only 小池、分区替代或兼容路径。 |
| 2026-07-13 04:20 +08:00 | Render11 canonical failure 已回传 fixed | 对应功能 owner 已将 `lightmap-forward-bind-group-integration-compile` 从 Render11 迁回 `editor/15/fixed-2026-07-13-lightmap-forward-bind-group-integration-compile.md`；Editor15 当前不再保留该 open failure。此前 official validator 已证明 `zircon_runtime` / `zircon_editor` test-profile 编译越过原 E0425/E0716。 |
| 2026-07-13 04:31 +08:00 | 线程耗尽最低所有权根因继续下沉 Runtime02 | 静态所有权链已闭合为 `CoreRuntimeInner.services -> ServiceEntry.instance -> EditorManager -> EditorUiHost.core -> CoreHandle.inner -> CoreRuntimeInner`；该 `Arc` 环使旧 Runtime 及其每组 16 个 task-pool worker 无法 drop，解释 manager 单线程窄分区约 34 组的 549-thread 峰值。canonical failure 已写入 `docs/plans/zircon_runtime/runtime/02/failure-2026-07-13-service-corehandle-retention-cycle.md`。Runtime11 仍拥有 task-pool/asset-worker 双预算，但等待 Runtime02 修复后重测；Editor15 M1 继续保持 `in_progress`。 |
