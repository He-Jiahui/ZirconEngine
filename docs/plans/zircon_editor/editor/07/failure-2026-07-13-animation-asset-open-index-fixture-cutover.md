---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: animation-asset-open-index-fixture-cutover
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor/07
related_code:
  - zircon_editor/src/tests/editor_event/animation_runtime/
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
  - zircon_editor/src/tests/editor_event/support.rs
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/
  - zircon_editor/src/core/asset/toolkit_route.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::editor_event::animation_runtime -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked tests::workbench::reflection::action_dispatch -- --test-threads=1
---

# Editor07：动画资产打开测试夹具未迁移索引权威

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-13 | Editor09 M1 当前完整门在动画 graph/state-machine/sequence/rebind 与 Workbench reflection 共观察到 18 项失败。代表 exact 证明 `OpenAsset` 后没有动画 view；测试仍只把 `.zranim` 写到任意临时绝对路径，未把资产纳入测试 ProjectAuthority/Editor asset index。失败已交接 Editor07 domain-editor fixtures，不恢复 suffix 分派。 |
| `OPEN / typed route 前置已提供` | 2026-07-14 | Editor09 进一步修复真实生产缺口：generic view payload 已从机器路径硬切为 `AssetToolkitOpenRoute { asset_locator, open_operation }`，animation session 的 source path 由当前 `ProjectManager` 解析。route 3/3 与 indexed-open/suffix-rejection 2/2 已在当前 Windows binary 通过。Editor07 的测试必须同时迁移 ProjectAuthority、catalog index、`res://` event/command locator 与 typed payload 断言；禁止只给绝对路径注入伪 kind。 |
| `OPEN / fixture GREEN / upward exact pending` | 2026-07-14 | 第一轮真实项目迁移仍为 animation runtime 0/15，证明 catalog 已索引但测试宿主没有产品宿主的 Timeline Sequence / Animation Graph plugin contributions；日志 `.codex/tmp/editor07-animation-runtime-focused-20260714.log`。最低共享测试支撑随后改为 `ProjectAuthority -> ProjectManager source path -> EditorManager open -> EditorAssetManager catalog sync`，并通过正式 capability-gated `EditorExtensionRegistry` 注册两组 typed toolkit descriptors；字段硬切前的受管 Windows binary 中 animation runtime 15/15、reflection action 1/1 通过，日志 `.codex/tmp/editor07-animation-runtime-focused-r3-20260714.log` 与 `.codex/tmp/editor07-reflection-animation-focused-20260714.log`。三资产 typed route exact 已确认 route/operation 均正确，仅旧 domain-specific status 文本与 generic toolkit status 不一致；断言已切到新架构文本，等待 workspace 锁状态稳定后重编 exact，故本 artifact 暂不回传。 |
| `OPEN / identity protocol hard-cut GREEN / Cargo 复编等待锁同步` | 2026-07-14 | `EditorAssetEvent::OpenAsset`、`AssetCommand`、`AssetHostEvent` 与 animation command/host/normalized event 的逻辑资产字段已无兼容别名地统一为 `asset_locator` / `graph_locator` / `state_machine_locator`；新增静态编译守卫完成 RED→GREEN 1/1，日志 `.codex/tmp/editor-asset-identity-protocol-hard-cut-red-20260714.log`、`.codex/tmp/editor-asset-identity-protocol-hard-cut-green-r2-20260714.log`。最终 Cargo exact 在执行测试前被当前 workspace manifest 与 `Cargo.lock` 不一致挡住；锁文件写权限归活跃 `plugins-08-zrvm-m1-20260714` 会话，本修复未覆盖或代写其依赖状态，待 owner 同步后立即复编。 |
| `OPEN / current-source Cargo 已启动 / 下层 owner 阻断` | 2026-07-14 | `Cargo.lock` 同步后 `cargo metadata --locked --offline` 已通过；受管 Windows current-source exact 已运行，旧“等待锁同步”不再是当前门禁。编译在进入 Editor 前被 Plugins08 reflection、Text02 variable shaping 与 Runtime04 reference resolver 共 31 项下层错误截断，日志 `.codex/tmp/editor07-focused-document-current-exact-20260714.log`；失败已写入各自功能计划，本 artifact 继续保持 open，禁止用旧 binary 结果冒充字段硬切后的 current-source GREEN。 |
| `OPEN / owner 修正已出现 / current-source 复验排队` | 2026-07-14 | Plugins08 reflection 已回传 fixed，Text02 helper 可见性与 Runtime04 `AssetUri::parse(&str)` 修正也已出现在共享源码；Editor07 不代替功能 owner 关闭其行为门。字段硬切后的 animation runtime、animation-assets 与 reflection exact 正等待受管 Cargo pool，复验完成前本 artifact 保持 open。 |
| `OPEN / current-source 第二轮未进入动画测试体 / EditorUI03 阻断` | 2026-07-14 | 受管 job `9cc782db74224c43887dfe73b46a4680` 先跑共同 focused-document 编译门，证明原 Plugins08/Text02/Runtime04 31 项阻断已不再出现；当前只剩本计划自有测试 import E0432（已按 `ui::host::module::EDITOR_MANAGER_NAME` 唯一 owner 修正）与跨功能 retained paint-text `ShapedGlyph.font_instance_id` E0063。后者已追加到 [EditorUI03 retained-text failure](../../editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md)，完整日志 `.codex/tmp/editor07-focused-document-current-exact-r2-20260714.log`。animation runtime/assets/reflection 因首门失败未执行，不声明通过。 |
| `OPEN / current-source 已编译 / Render18 Shader IDE 前置阻断` | 2026-07-15 | 受管 job `c37d4e6f07f24e5d9424536d3c44b092` 已完成当前 `zircon_editor` lib-test 编译，且 `focused_document` 2/2 通过；animation runtime 0/15 在共同的 `EditorAssetManager -> Shader IDE env` 初始化前置失败，尚未进入动画逻辑。`zr_lightmap.wgsl` 调用 `zr_irradiance_volume_sample` 时未组合 Render18 的 irradiance-volume stub，已交接并返回为 [Render18 irradiance-volume Shader IDE dependency fixed](fixed-2026-07-15-irradiance-volume-shader-ide-validation-dependency.md)。完整日志 `E:/ZirconBuilds/editor07-failure-return-animation-runtime-20260715.out.log`；本 artifact 继续保持 open。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1 完整 Windows lib-test acceptance 的 animation/domain-editor 聚类
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：Editor09 已完成 `OpenAsset -> indexed AssetTypeId -> registry toolkit` 硬切；动画领域测试、session 与 reflection action 的夹具归 Editor07，不能要求资产入口恢复路径后缀猜测。

## 失败现象与复现证据

完整门日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log` 在人工终止停滞测试前已记录：

- `tests::editor_event::animation_runtime::{graph,rebind,sequence,state_machine}` 共 15 项失败；
- `tests::editor_event::runtime::animation_assets` 2 项失败；
- Workbench reflection animation-track action 1 项失败。

代表 exact `animation_graph_ignores_connections_from_missing_source_nodes` 为 0/1，panic 为
`graph editor view should stay open`；reflection exact 同样为 `animation sequence view should remain open`。
测试先将 `.zranim` 写入任意 `%TEMP%` 绝对路径，再直接派发 `OpenAsset`。当前生产入口会先调用
`asset_type_id_for_locator`；未被索引的 locator 明确返回 `Asset type is not indexed ...`，因此没有 toolkit
view。该信号发生在上层动画 mutation 之前。

## 最低共享层根因

Editor07 的共享 `EventRuntimeHarness`/动画 fixture 仍隐含“文件后缀即可决定 editor toolkit”的退役合同；
Editor09 M1 已删除该平行分派真源，要求 ProjectAuthority 下的资产索引提供 typed identity。18 项上层动画
断言因同一个未迁移 fixture 前置条件失败，并非 18 个独立 graph/state-machine 算法缺陷。2026-07-14
的更深复核同时证明旧 payload 把 `res://` 当作 OS path；最低共享 route 已由 Editor09 改为 typed locator，
Editor07 已把 animation event 的 `graph_locator/state_machine_locator` 参数、session identity 比较及全部
domain fixture 统一迁移为 locator；`asset_path/graph_path/state_machine_path` 不再作为跨层逻辑身份协议，
物理源路径仅在 `ProjectManager` 解析 locator 后留在 session 内部用于实际读写。

## 架构修复验收

- 动画测试资产在测试 ProjectAuthority 下创建，并通过 canonical index/registry fixture 获得
  `AssetTypeId`；不得直接向生产 state 注入伪造 kind 字符串。
- `OpenAsset` 与 graph/state-machine/sequence/rebind command 均使用 canonical `res://` locator；view
  payload 断言反序列化 `AssetToolkitOpenRoute`，不再断言 `payload["path"]`。
- animation runtime、runtime animation-assets 与 reflection action 原组自然通过。
- 增加反向守卫：未索引 locator 继续被明确拒绝，生产路径不存在 suffix fallback 或第二 toolkit map。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止在 `OpenAsset` 恢复 `.zranim`/`.zui` 后缀 match，禁止 test-only 注册旁路或在动画事件处理器绕过资产索引。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
