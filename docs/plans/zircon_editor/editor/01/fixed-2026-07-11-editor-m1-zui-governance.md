---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: editor-m1-zui-governance
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_layout/15
resolved_at: 2026-07-11
related_code:
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/activity_drawer_window.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/assets/ui/editor/windows/asset_window.zui
  - zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.zui
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/slot.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_shell.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/assets/ui/editor
  - zircon_runtime/assets/ui
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked zui_asset_governance -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked tests::host::retained_menu_pointer::visual_screenshot::blend_space_workspace::blend_space_workspace_adapts_between_compact_and_wide_windows -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib ui::tests::v2_asset::file_cache::ui_v2_file_cache_resolves_builtin_asset_id_widget_imports --locked -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib --locked -- --test-threads=1
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --offline --jobs 1
  - zircon_editor-1ca47919e17744f1.exe tests::host::template_runtime:: --test-threads=1
---

# Editor Layout 15：Editor M1 ZUI 治理失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 Windows 全量失败聚类与 V2 公共契约闭环测试阶段
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 交接原因：失败集中在 Editor Layout 15 所有的 production `.zui` 资产与共享 governance expectation，来源计划的 Editor kernel 不拥有这些资产规则。

## 失败现象与复现证据

Editor 架构会话的独立诊断 binary 中 `zui_asset_governance` 为 68/71。布局 owner 最新官方单线程完整门禁为 2761 passed / 133 failed / 34 ignored，且该切片自有 focused tests 全绿，因此不能把 133 项整体归因于本计划，必须在当前源码重新运行治理组并逐项建立 owner 映射。

复现命令：

```text
cargo test -p zircon_editor --lib --locked zui_asset_governance -- --test-threads=1
cargo test -p zircon_editor --lib --locked -- --test-threads=1
```

## 最低共享层根因

68/71 的三个直接根因已经逐项收敛到最低共享层：

- 51 个 production `.zui` 仍使用逻辑短 ID，而 production identity 合同要求 ID 与文件派生的 `res://` locator 相同；继续维护 alias 会形成第二套身份真相。
- `child_mount.slot.layout` 已被 compiler、Taffy、responsive MUI 和 surface builder 作为正式结构合同消费，但 governance allowlist 尚未承认该 table。
- viewport gizmo 在 `style.self` 中重复写入只属于 props 的 `text_tone`/`foreground_color`，违反共享 style-self schema。

完成 locator 硬切后又暴露三项同层结构漂移：`activity_drawer_window.zui` 仍留在旧 `editor/host` 路径，L4 allowlist 未把组件投影边界 `Slot` 视为合法结构节点，以及 builtin template registry 仍用 24 个逻辑键为同一 ZUI 文件建立第二身份。独立审查进一步证明 alias table/helper/宽松测试仍允许别名回归，且 child-slot schema 只扫描 component、遗漏已有 `slot.layout` 的 view 文档。它们均在共享资产、runtime registry、fixture 和 governance owner 修复，没有向 loader、单一资产或截图路径增加例外。

## 架构修复验收

- 当前源码的 `zui_asset_governance` 失败逐项映射到 authored `.zui` 或共享 governance owner。
- 先修生产资产或共享规则，并为每类根因增加最低层回归。
- governance 组全绿后再向上运行完整单线程 Editor 门禁并记录精确结果。

## 禁止临时方案

- 禁止恢复旧 `.ui.toml` / `.v2.ui.toml`、旧 `kind = "layout"` 或兼容 loader 路线。
- 禁止为单个截图、单个测试或单一资产增加绕过 governance 的特殊分支。

## 修复结果与回传

- 根因：production asset identity、builtin runtime registry identity、child-slot metadata schema、style-self schema、Activity Drawer owner path 与 L4 structural allowlist 六处共享合同发生漂移。
- 架构修复：244 个 production `.zui` 全部满足 file-derived locator；24 个 builtin registry key 及其生产/测试直接消费者硬切为同一 locator；alias table、helper 与允许 alias 的治理分支全部删除；Activity Drawer 从 `editor/host` 硬切到 `editor/components/workbench/shell`；`slot.layout` 仅接受 table 且检查覆盖 component/view；viewport gizmo 删除无效 style-self；L4 允许组件投影边界 `Slot`。
- 验证：审查后最新 Editor test binary 中 governance 72/72、builtin descriptors 10/10、pane body 11/11、bootstrap 11/11、floating layout route 1/1；template runtime 48/50，两个失败为既有 dual-host/style-override 基线。2026-07-12 的独立复验先准确捕获 Blend Space 新资产把治理拉回 70/72：五个轴声明混用了 schema 外 `weight`，并把 `Stretch` 与显式尺寸约束叠加。修复后侧栏统一为有界 `Fixed`，中心与采样画布统一为无尺寸约束 `Stretch`，治理恢复 72/72，900/1260 自适应几何 1/1。审查前完整单线程 Editor lib 为 2761 passed / 133 failed / 34 ignored（2497.76s），治理项无失败。
- 外部门禁：`template_assets` 为 15/21，6 项失败属于 host invalidation/pointer/DTO/welcome 基线；`integration_contracts` 在运行前被 `workbench_autolayout.rs` 4 个既有 E0061 阻断。两者均不回退本治理修复，也不据此声明 Editor M1 全量通过。
- 回传：本 ZUI governance 故障已修复并回迁 Editor 01；Editor M1 可移除该失败 owner，但仍须继续处理完整门禁中的 133 个其他失败和 integration-contracts 的 API 漂移。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| S15 / Editor M1 | Current-source ZUI governance 重建复验 | `已修复-治理组71/71` | 2026-07-11 | 早期 owner binary 完整执行为 70/71，唯一失败是 `l4_surfaces_contain_no_inline_primitive_structures` 对 `activity_drawer_window.zui` 七个 `Slot` 节点的 L4 合同裁决。Layout owner 完成共享资产/治理收束后，06:17 当前 Editor binary 执行 `zui_asset_governance --test-threads=1 --nocapture` 为 71 passed / 0 failed / 0 ignored，耗时 138.63s。未恢复旧 `.ui.toml` / `.v2.ui.toml`、旧 `kind = "layout"`，也未增加单文件路径豁免。 |
| S15 / Editor M1 | Fixed handoff 独立复验 | `已修复-治理组71/71` | 2026-07-11 | 使用 06:17 且晚于相关源码的 Editor test binary 重新执行全部 `zui_asset_governance`，结果为 71 passed / 0 failed / 0 ignored / 2857 filtered out，耗时 133.20s。 |
| S15 / Editor M1 | Activity Drawer 测试 fixture 硬切 | `已修复-旧ID与旧路径0命中` | 2026-07-11 | Runtime V2 file-cache fixture 从旧 `editor.host` / `ui/editor/host` 迁到 `editor.workbench.shell` / `components/workbench/shell`，保留 asset-ID 与 resource-path 双索引语义；旧 ID/路径生产与测试扫描 0 命中，fully-qualified file-cache exact 为 1 passed / 0 failed（0.11s）。 |
| S15 / Editor M1 | Current-source 向上门禁复验 | `治理已修复-审查后72/72-Editor-M1总门禁仍开放` | 2026-07-11 | 09:01 binary `F:\cargo-targets\zircon-ui-state-priority-0711\debug\deps\zircon_editor-1ca47919e17744f1.exe`：governance 72/72（120.61s）、builtin descriptors 10/10、pane body 11/11、bootstrap 11/11、floating route 1/1；template runtime 48/50，两个失败为既有基线。24 个旧 registry ID、alias mechanism、伪 `.zui.*` locator 与旧 Drawer path 均 0 命中；244 个 production `.zui` 为 0 locator mismatch。审查前完整单线程为 2761 passed / 133 failed / 34 ignored（2497.76s）。 |
| S15 / Editor M1 | Blend Space 共享轴 schema 回归修复 | `已修复-治理72/72-双档几何通过` | 2026-07-12 | 对 canonical fixed handoff 做当前源码复验时，governance 先为 70/72：`workbench_extension_blend_space_workspace.zui` 的 left/center/right/sample-canvas/preview-card 五个宽度轴引入 schema 外 `weight`，且 `Stretch` 同时声明 min/preferred/max。按硬切规则删除私有方言：left/right/preview 采用有界 `Fixed`，center/sample-canvas 采用纯 `Stretch`；同时把预览卡固定节奏收敛为 120/136/180，使 900 宽下 sample canvas 仍占主导。两个原失败 exact 各 1/1，完整 `zui_asset_governance` 为 72 passed / 0 failed（123.06s），`blend_space_workspace_adapts_between_compact_and_wide_windows` 为 1/1（37.74s）。字体发现 HUD exact 1/1（49.24s）与 plugin-provider exact 1/1（0.49s）亦通过，证明本轮只修复布局治理回归，没有破坏另外两条 Editor M1 handoff。 |
| S15 / Editor M1 | 当前源码重建门禁 | `外部阻断-Editor15测试导入漂移` | 2026-07-12 | Windows 托管 Cargo job `8d056be2d1c24b7ab4c08853a201c068` 在共享 target 上重建当前源码，Layout/ZUI 改动本身未产生诊断；编译被当前 Editor 15 typed-export 会话所有的 `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs:360` 单一 E0433 阻断：测试使用 `ExportProcessError` 但未导入。该文件与 `export_build` 目录均由 `20260712-editor15-typed-export` 持有租约，本记录不越权吸收；因此采用此前晚于 Layout painter 代码的 3074-test binary 完成资产加载型 governance、双档几何、HUD 字体与 provider 精确复验，不把本轮 Cargo 重建冒充 green。 |
| S15 / Editor M1 | 2026-07-12 canonical fixed handoff 末次复验 | `已修复-governance 72/72且双档几何1/1通过` | 2026-07-12 | Windows 托管 Editor binary（21:47）完整执行 `zui_asset_governance` 为 72 passed / 0 failed / 0 ignored（130.61s，3,036 filtered），并执行 Blend Space compact/wide geometry exact 为 1/1（49.87s，3,107 filtered）。当前 production 只保留 `.zui` locator identity、共享 Slot/schema 与 L4 composition 合同；未恢复 `.ui.toml`、旧 kind、alias loader、路径豁免或测试专用布局旁路。 |
