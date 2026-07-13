---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: ui-asset-v2-projection-drift
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
related_code:
  - zircon_editor/src/tests/ui/ui_asset_editor
  - zircon_editor/src/tests/ui/asset_browser
  - zircon_editor/src/ui/layouts/views/asset_browser
  - zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs
  - zircon_editor/src/core/asset/toolkit_route.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - cargo test -p zircon_editor --lib --locked ui_asset_editor -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked asset_browser -- --test-threads=1
---

# Editor UI 05：UI Asset V2 / Asset Browser 投影漂移失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 当前源码完整门与 UI Asset / Asset Browser 聚类复核
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 交接原因：canonical `.zui` identity、V2 document、authoring projection 与 Asset Browser host 合同均由
  EditorUI05 持有；Editor kernel 与 Editor09 只能报告上行失败，不能恢复旧 synthetic identity、旧 loader
  或在资产类型注册表旁建立第二份 UI kind 真源。

## 失败现象与复现证据

Editor M1 当前源码 08:31 binary 的完整门禁中，UI Asset/Asset Browser 聚类仍为 12 项失败，分布于 bootstrap assets、editor layouts、reflection、host dispatch、asset-browser view 与按钮绘制。独立 exact `ui_asset_editor_v2_projection_asset_self_hosts_shell_regions` 为 0/1（0.01s）：实际当前 identity 是 `res://ui/editor/ui_asset_editor.zui`，旧断言仍要求 `editor.ui_asset_editor.projection.v2`。

该聚类归 Editor UI 05 的资产身份、V2 projection 与 Asset Browser host 合同。后续必须逐项裁决旧 synthetic projection id 与当前 canonical `res://...zui` identity；禁止恢复旧 `.ui.toml`/`.v2.ui.toml`、projection-v2 别名或双 loader。

2026-07-13 Editor09 M1 静态 hard-cut 复核进一步确认生产路径仍存在
`v2_document_to_legacy_projection_document`、`legacy_projection_document_to_v2_document`、
`legacy_asset_kind` 与 `legacy_asset_kind_for_v2`，证明当前不只是测试 identity 漂移，而是 V2 `.zui`
authoring 仍通过旧 `UiAssetDocument`/`UiAssetKind` 平行模型往返。该事实属于本交接，不另建重复 failure。

## 最低共享层根因

当前 runtime `UiV2AssetDocument`/`.zui` 已是磁盘与运行时 canonical schema，但 Editor authoring session、
route kind、imports 与若干 host consumers 仍依赖旧 `UiAssetDocument` 树形 DTO。双向 projection 把 V2 flat
node graph 转回旧树再编辑并重新展平，保留两套 kind/node/style truth；旧 synthetic projection identity
测试又与 canonical `res://...zui` 身份并存。最低 owner 是 EditorUI05 的 UI Asset editor/session 与
runtime V2 consumer 收口，不是通用 Editor asset registry。

## 架构修复验收

- `.zui` authoring、route、imports、session save/refresh 直接消费一个 canonical V2 document/model；删除
  `*_legacy_projection_*` 双向转换与旧 kind 映射，不保留 alias、wrapper 或双 loader。
- Asset Browser、bootstrap projection 与 host dispatch 使用 canonical `res://...zui` identity，旧
  `editor.*.projection.v2` synthetic id 不再出现在生产或测试断言。
- 原 UI Asset/Asset Browser focused filters 自然通过，再向上重跑完整 Editor lib-test；不得用只更新
  失败断言掩盖生产双模型。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止恢复 `.ui.toml` / `.v2.ui.toml`、projection-v2 id、旧 kind parser fallback 或 V2→legacy→V2
  长期桥；迁移必须硬切到单一 `.zui`/V2 owner。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor UI 05 / Editor M1 | UI Asset V2 canonical identity 与 Asset Browser projection | `未通过-12项待功能owner处理` | 2026-07-11 | 全量门禁归类 12 项；V2 self-host exact 0/1，实际 `res://ui/editor/ui_asset_editor.zui`、旧期望 `editor.ui_asset_editor.projection.v2`，证明至少一项是硬切后旧身份断言漂移。 |
| Editor UI 05 / Editor M1 | 当前源码完整门禁复核 | `未通过-失败集合未变化` | 2026-07-11 | 08:31 当前源码 binary 完整执行 2930 项为 2763/133/34（2258.13s）；与 06:17 门禁逐项比较，133 个失败名 added=0、removed=0，本计划 12 项归属不变。同一 binary 的 V2 self-host exact 0/1（0.01s），仍是 canonical `.zui` identity 对已退役 projection-v2 id。 |
| Editor UI 05 / Editor03+08 M1 | 当前全量门 UI Asset/Asset Browser 回归复现 | `未通过-继续由功能owner处理` | 2026-07-12 | 受管 job `520d85713df249afae31661a7697ad07` 再次复现 UI Asset reference/promotion、theme tooling、workspace watcher、bootstrap projection 与 Asset Browser layout/scroll 失败；代表项包括 `editor_manager_promotes_selected_ui_asset_component_to_external_widget_asset`、`ui_asset_editor_v2_projection_asset_self_hosts_shell_regions`、`asset_browser_projection_maps_bootstrap_asset_into_mount_nodes`。完整失败名保存在 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`；本记录继续要求 canonical `.zui` 单路修复，不允许回退旧 projection id、旧 loader 或双格式兼容。 |
| Editor UI 05 / Editor09 M1 | 当前源码完整门停滞前复现 | `未通过-继续由功能owner处理` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 在第 1755 项外部停滞前再次记录 UI Asset editor host、theme tooling、Asset Browser bootstrap、V2 self-host 与 UI Asset route 失败；日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。生产双向 legacy projection 静态事实也仍在本记录内，禁止另建兼容通道。 |
| Editor UI 05 / Editor09 M1 | generic toolkit route 硬切后的 domain route 收束 | `未通过-domain owner 后续处理` | 2026-07-14 | Editor09 已删除 generic `{ path, operation_id }` payload，新增 canonical `AssetToolkitOpenRoute`，UI asset restore 可从该 route 经 ProjectManager 解析 source。当前 `UiAssetEditorRoute.asset_id` 与 direct host APIs 仍允许裸字符串/物理路径，且 V2↔legacy 双模型未修；这些剩余项继续由 EditorUI05 一次性硬切，禁止把旧 `path` fallback 加回 generic host。 |

## 修复结果与回传

- 状态：`open / 待修复`；先分别跑 UI Asset 与 Asset Browser 组，再向上重跑 Editor M1。
