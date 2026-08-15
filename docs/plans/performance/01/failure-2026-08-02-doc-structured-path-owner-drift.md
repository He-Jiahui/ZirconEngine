---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-02
summary_slug: doc-structured-path-owner-drift
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/performance/01
related_code:
  - tools/check_conventions.py
  - docs/engine-architecture/runtime-editor-pluginized-export.md
  - docs/zircon_runtime/plugin/package_manifest.md
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - docs/assets-and-rendering/runtime-physics-animation-assets.md
  - docs/zircon_editor/ui/preferences.md
  - docs/engine-architecture/core-runtime-service-registry.md
  - docs/plans/zircon_plugins/13/2026-08-10-current-status-receipt-test-sprawl-return.md
  - zircon_runtime/src/core/resource/io/atomic_file.rs
  - zircon_runtime/src/platform/preferences/atomic_file.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_runtime/src/graphics/text_transport/mod.rs
  - zircon_runtime/src/text/ui_style.rs
tests:
  - python tools/check_conventions.py --repo-root . --only docs --json
  - python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root .
  - python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root .
---

# 全仓文档 structured-path 与活跃 owner 硬切漂移

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-08-02 全仓 plans、功能文档、源码与测试并行复核
- 修复责任计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 交接原因：剩余漂移横跨多个活跃编号计划；Performance01 保留全局 gate 与路由责任，各 owner 在自己的硬切接受后回传。
- 协调责任：Performance01 维护全局证据和 owner 路由；各编号计划只修复自己持有的活跃硬切路径。

## 失败现象与复现证据

`python tools/check_conventions.py --repo-root . --only docs --json` 的初始快照为 2,206 篇文档、70,127 个结构化路径、95 篇受影响文档、550 条 `missing path`。六轮 owner-aware 修复一度把 current-worktree 降到 5 条；后续活动硬切又引入 `atomic_file.rs`、UI text adapter 与 Render01 cache/test owner 漂移。2026-08-02 本轮复跑的真实快照为 2,216 篇文档、69,669 个结构化路径、10 篇受影响文档、15 条 `missing path`，相对初始净消除 535 条。Performance01 子目录自身保持 0 条；新增项未通过跨 owner 改写掩盖。

已直接修复的主要范围：

- 从 Sound 架构文档和 Bevy parity matrix 的双向索引删除 230 条已按计划退役的自研 render、DSP state、direct CPAL 和 software mixer 路径，并补入现行 Kira bridge owner；
- 修正 virtual-geometry execution ownership、text source-map、render scene/post-process、export build-plan 等唯一映射路径；
- 把 Performance01 的 glob 伪路径改成真实目录 owner，并将 scene render extract 记录更新为 sealed `LevelFrameStateSnapshot` 当前合同；
- 清理已退役的 Hub 空测试目标、Editor11 receipt/fixture 和 Virtual Geometry test-only execution-mode 代码。
- 把 Plugins01 两份 runtime-plugin manifest/export 文档的 85 条失效 leaf owner 收敛到当前 package/feature validation projection、`derived_projection.rs` 与 editor plugin module owner，并删除 6 条重复结构化路径；
- 把四份 Editor UI/preferences 文档的 36 条 retired `ui/preferences` 路径迁到当前 `core/settings` 与 retained-host owner，同时修复无效 YAML test scalars；
- 由三位 `gpt-5.6-sol` / High 并行审阅者分别收敛 Runtime animation/physics/scene、graphics/text/render 和 Editor/plugin/UI 文档；所有直接修改范围的 structured-path 缺口归零，未实现的动画 diagnostics、Editor recovery 与 managed-validation failure 保持 open；
- 重写 25 万字节的 core runtime service registry 历史流水账为当前架构索引，并修复 event/config/platform 文档、字体 composite hard cut、asset catalog folder owner、Render06 full/profiled 资源与 Text03 folder-backed source map；
- 审计 276 个 tracked current-status 测试：272 个仍存在、23,122 行、246 个直接读取 `docs/plans`，且 260 个当前已有外来修改。未覆盖这些并发变更做批量删除；已在 Plugins13 建立 [`current-status-receipt-test-sprawl`](../../zircon_plugins/13/failure-2026-08-02-current-status-receipt-test-sprawl.md) child failure，要求按真实行为价值分类后删除或迁移，Runtime15 既有 Rust receipt debt 继续由其 canonical failure 持有。
- 修正 Render01 disabled-forward volumetric failure 的 4 条 current owner 路径和 false-RED 根因，保留仍有回归价值的 mounted structure guard；同步修正两份 owner-free Asset 功能文档的 atomic owner，以及 Render18 fixed record 的 graphics text-transport owner，共直接消除 7 条新增漂移。

当前 15 条按最低 owner 边界归类如下：

- 先前 5 条不变：Runtime04 两份 open failure 共 3 条，Shader03 open failure 1 条，Editor07 fixed record 1 条；它们的精确现行 owner 映射已在各自活动 Session 中保留；
- `foundation/persistence/atomic_file.rs` 硬切剩余 8 条，分布在 Runtime02/04 failure、zmeta 与 config-manager 功能文档；current source 已统一调用 `core/resource/io/atomic_file`。Asset migration/registry 两份 owner-free 文档已直接同步，Runtime02 与两份功能文档为 foreign-dirty，Runtime04 record 则由其活动 owner 回写；
- UI text hard cut 剩余 2 条，分别位于 Runtime04 与 Shader06 records。Render18 已改到 `graphics/text_transport/mod.rs`；Shader06 可做相同精确替换，Runtime04 的旧 adapter 同时覆盖 `text/ui_style.rs` 与 `graphics/text_transport/mod.rs`，不能缩成单一 owner；
- Render01 failure 的 4 条错误短路径已恢复为当前 `graphics/scene/scene_renderer/...` owner，根因已改为 rustfmt 折行触发的单行锚点 false RED。测试仍由 `gpu.rs` 挂载并约束字段、构造和 binding 引用，不属于过时 receipt；failure 保持 open，等待受管精确测试与原 Text01 向上门禁。

结构化路径以外，本轮还发现 shared atomic hard cut 的运行合同缺口：`core/resource/io/atomic_file.rs` 在新文件创建和 Unix rename 后没有同步父目录，而 `platform/preferences/atomic_file.rs` 另行执行该同步；因此现有功能文档宣称共享 owner 提供完整 crash-safe/platform durability 过强。当前新路径为 21 处引用、分布于 18 个文件，并包含 `zircon_editor` 的 workspace persistence consumer；Runtime04 的 canonical atomic failure 仍按 20 个 Runtime consumers 验收，漏掉 Editor rollback/write-failure 测试和 Editor package boundary。该问题必须由 Runtime04 在共享 owner 层修复并消除 Platform 重复，或先把文档合同收窄为原子可见性，不能用路径修正掩盖。

## 最低共享层根因

功能文档和计划的 structured metadata 长期枚举 leaf 文件，却没有随 hard cutover 一起进入 owner 的完成条件。模块拆分或吸收后，正文、`related_code`、`implementation_files` 和历史 output record 分别漂移；随后为让链接暂时存在而猜测同名替代路径，又会把错误 owner 固化。最低共享修复不是批量字符串替换，而是把“代码硬切完成”与“当前态文档索引同步、handoff 回传、docs gate 归零”绑定为同一 owner 验收。

## 已完成的 Session 路由

本轮已把高置信映射或未实现缺口写入相关 Session reason：Editor01/03/04/06/07/08/09/11/12/14/16、Plugins01/04/13、Runtime04/08/15、Shader03/06、Text02、Frameworks01/05、Sound01 和 Hub07。Editor04/06/14/16、Plugins04 动画 diagnostics、Hub07 managed validation、Runtime04 atomic durability 与 Text02 managed upward validation 明确保持 open。无活动 owner 的 Render06、Text01、Text03 由短生命周期 `gpt-5.6-sol` / High maintenance Session 修复唯一 metadata 映射；它们不改变业务计划状态或已有 failure 生命周期。

## 架构修复验收

- 每个活跃 owner 在其硬切接受后同步功能文档、主计划和 failure/fixed records，结构化字段只引用真实当前文件或目录；
- `python tools/check_conventions.py --repo-root . --only docs --json` 达到 `violation_count = 0`；
- failure handoff validator 与 plan-output audit/self-test 全部通过；
- 需要动态证据的 open failure 取得 managed gate 结果后再执行 failure return，不用静态推断代替运行证据。

## 禁止临时方案

- 不创建空文件、兼容 facade、alias 或无消费者测试来恢复旧路径；
- 不把已删除 leaf 随意改成某个同名目录来骗过 `Test-Path`，必须先证明当前 owner 和正文语义一致；
- 不跨越活跃 Session lease 改写其计划状态或把 open failure 标为 fixed；
- 不删除历史正文中的明确 dated receipt，但 current-state 段落和 structured metadata 必须指向当前实现。

## 修复结果与回传

Open state: `15 条 / 10 文档的跨 owner structured-path 漂移仍待对应硬切 owner 收敛`。本轮已完成 Performance01 owner-clean 修复、Session 路由和证据降噪；Runtime02 等 foreign-dirty owner 未被跨写，Render01 metadata 已修正但其 managed gate 仍待新的 primary。全仓 docs gate 尚未通过，shared atomic durability 合同也尚未闭合，不得声称该 failure fixed。

## 2026-08-11 current-source continuation

当前共享源码继续发生大量 owner hard cut，fresh
`python tools/check_conventions.py --repo-root . --only docs --json` 快照已变为 176 篇受影响文档、
539 条 violation；上面的 `15 条 / 10 文档` 仅保留为 2026-08-02 历史快照，不能再作为当前
验收数据。本轮只修正了直接相关且 owner 唯一的 config-manager metadata：删除已退役的
`foundation/persistence{,/atomic_file}.rs`，改指
`zircon_runtime/src/core/resource/io/atomic_file.rs`，没有对其余 foreign-dirty 文档做批量猜测替换。

同一 continuation 已在 Runtime04 shared owner 内补齐目录创建、new-target、Unix replacement/
backup cleanup 与 Windows committed-target 的 durability 责任，并删除 Platform Preferences 的
重复 committed-target sync。源码静态门与独立二次审查均无发现，但 managed Rust 测试在 Cargo
启动前被 unmanaged-artifact preflight 拒绝。因此 atomic durability 的源码合同已前向收敛，
运行验收仍由 Runtime04 canonical handoff 保持 open；本全仓 structured-path failure 也继续
保持 `status: open`，不得用该局部修复声称 docs gate fixed。
