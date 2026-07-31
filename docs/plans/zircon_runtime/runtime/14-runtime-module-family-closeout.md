---
related_code:
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/sequence.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/animation/scene_hook.rs
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
  - zircon_runtime/src/engine_module/engine_module.rs
  - zircon_runtime/src/engine_module/service_factory.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_markdown.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/zircon_runtime/animation/runtime.md
  - docs/zircon_runtime/navigation/runtime.md
  - docs/zircon_runtime/diagnostic_log/mod.md
  - docs/zircon_runtime/engine_module/relationship.md
status: in_progress
last_refined: 2026-07-19
---

# 14 运行时模块族完备性收尾（animation / navigation / diagnostic_log / engine_module）

2026-07-19 current-source owner sync：`module_family_boundary` 报告 `expected_family_count = 4`、`animation = 28`、`navigation = 12`、`diagnostic_log = 7`、`engine_module = 8`，全部 missing/file-count/risk 列表为空。navigation 新增的 `operation/{mod,handler,registration}.rs` 是 folder-backed shared-operation integration owner，不改变 fallback runtime manager、crate-root seat 或 editor/plugin 边界。下方 9-file 记录保留为 operation integration 落地前的历史证据；当前镜像只由本计划、Runtime14 当前编号记录和 `docs/zircon_runtime/navigation/runtime.md` 维护，不再要求 Runtime15 历史归档复制最新计数。

Current audit anchors: `root_seat_guard_present = true`, `animation_status_json_guard_present = true`, `animation_status_json_anchor_count = 8`, `missing_animation_status_json_anchors = []`, `module_family_guard_anchor_count = 7`, `missing_module_family_guard_anchors = []`, `cargo_gate_anchor_count = 5`, `missing_cargo_gate_anchors = []`, `risks = []`, and `runtime_14_module_family_mirror_docs_match_structure_audit_counts`.

扫尾型计划：lib.rs 根模块中尚无任何子计划认领的四个"小族"逐一裁决——架构对照、厚度核查、归属定位。每族工作量小，但放任不管会成为下一轮审计的"游离散件"（02 处理 core/ 根散件的同构问题在 crate 根重演）。

## 现状与证据（2026-06-13 实仓盘点）

- **animation/**（28 个 `.rs` 文件）：`manager/`、`sequence/`、`clip_event.rs`、`scene_hook/`、`module.rs`——有真实动画运行时（clip 事件、序列、场景钩子、graph/state machine、模块注册）。Runtime 14 M0.1 已在 `docs/zircon_runtime/animation/runtime.md` 对照 bevy_animation/Fyrox/godot，裁决 root motion 为 backlog，GPU skinning 与编辑器工具为非目标，并澄清 morph 已有 asset/scene property/sequence/graphics CPU baseline，但不是 animation manager 专属求解器。
- **navigation/**（9 个 `.rs` 文件）：不是极薄注册层；`runtime.rs` 仍是 manager owner，`runtime/{baked_mesh,world_scan,avoidance,state,math,tests}.rs` 负责 baked navmesh、A* pathfinding、sample/raycast、world agent tick、obstacle/agent avoidance 与 focused fallback runtime tests。Runtime 14 M0.2 已在 `docs/zircon_runtime/navigation/runtime.md` 裁决：保留 crate-root fallback runtime，Recast/editor/baking 仍归 `zircon_plugins/navigation`。
- **diagnostic_log/**（7 个 `.rs` 文件）：`sink.rs`（含 `disabled_file_sink_skips_directory_candidates` 等测试）、`level/settings/platform/timestamp/diagnostics.rs`——**与 `core/diagnostics/`（store/collect/snapshot/profiling）构成双诊断面**。Runtime 14 M0.3 已在 `docs/zircon_runtime/diagnostic_log/mod.md` 裁决：`diagnostic_log` 保留进程文本日志面；`core::diagnostics` 保留数值诊断面；`diagnostic_log/diagnostics.rs` 是唯一 snapshot-to-log 桥接点。
- **engine_module/**（8 个 `.rs` 文件，含 `descriptors/names.rs`）：engine 级模块/服务声明层位于 crate 根，与 `core::runtime` 的 ModuleDescriptor/ServiceFactory/registration/lifecycle 分层共存。Runtime 14 M0.4 已在 `docs/zircon_runtime/engine_module/relationship.md` 裁决：保留 declared layering，不合并、不删除。
- **foundation/**：02-M1 已裁决"装配壳，重叠为空"——本计划不重复，仅在 engine_module 裁决时引用其判例。
- 参考锚点（2026-06-13 实测核验，动工前先读——index 公约 §7.9）：
  - bevy_animation（clip/graph/player）— `dev/bevy/crates/bevy_animation/src`（执行时核验：`ls dev/bevy/crates/ | grep animation`）
  - Fyrox 动画与混合 — `dev/Fyrox/fyrox-impl/src/scene/animation/`（执行时核验确切路径：Glob `dev/Fyrox/**/animation/**`）
  - Godot AnimationPlayer/AnimationTree — `dev/godot/scene/animation/`（执行时核验）
  - Godot navigation module layering — `dev/godot/modules/navigation_2d` 与 `dev/godot/modules/navigation_3d`（2026-06-13 已核验；计划初稿的 `modules/navigation` 路径错误）
  - bevy log/诊断分置（tracing 管道 vs diagnostic 数值）— `dev/bevy/crates/{bevy_log,bevy_diagnostic}/src`

## 目标

1. 四族各出一份"厚度与归属判词"：该族应有的职责边界、当前厚度是否合理、crate 根席位去留。
2. animation 对照差距表（与 04/08 同方法论）：clip/序列/事件/混合的有无逐项裁决"有意取舍 / 债"。
3. 双诊断面（diagnostic_log vs core::diagnostics）分工文档化 + 桥接点单点声明。
4. engine_module 与 core::runtime 模块抽象的关系裁决（合并/分层/删除三选一，硬切换执行）。

## 非目标

- 不实现新动画特性（混合树等若判"债"列 backlog 排期）；不动 navigation 插件的 Recast 行为；不重做 02 已裁决的 foundation。
- 渲染骨架（动画的 GPU skinning 上载等）归 render 计划。

### 全局硬约束（继承总计划 §4）

- 不新增 crate；硬切换不留兼容层；非网络语义 server 命名是 blocker（注意 Godot "NavigationServer" 词汇不得照搬命名）。

## 执行前检查清单

1. `git status --porcelain -- zircon_runtime/src/animation/ zircon_runtime/src/navigation/ zircon_runtime/src/diagnostic_log/ zircon_runtime/src/engine_module/`；活动会话避让。
2. 事实重核：`ls` 四族目录；Grep `engine_module::`，path `zircon_runtime/src zircon_app/src`（engine_module 调用面）；Grep `diagnostic_log::`，path 同（日志面调用方）。
3. 基线记录：`cargo test -p zircon_runtime --lib animation --locked` 与 `--lib diagnostic_log --locked` 通过数。

## 里程碑

### M0 四族判词（纯审计 + 裁决）

- 切片 0.1 animation 对照差距表：对照三引擎锚点逐项（clip 采样/事件/序列编排/混合/状态机/根运动），落 `docs/zircon_runtime/animation/`（执行时核验镜像文档）。DoD：差距表每行有"有意取舍/债"判词。
- 切片 0.2 navigation 厚度判词：runtime 侧"仅注册 + 服务投影"是否定稿为目标态；若是，加一行结构守卫断言 navigation/ 不长出行为文件（文件数白名单）。DoD：判词 + 守卫测试名落册。
- 切片 0.3 双诊断面分工：日志管道（diagnostic_log：文本/sink/level）vs 数值诊断（core::diagnostics：计数/snapshot）分工文档化；`diagnostic_log/diagnostics.rs` 的桥接角色单点声明；crate 根席位判词（保留 or 并入 core）。DoD：分工文档 + 桥接点声明，与 07 的计数通道口径互引。
- 切片 0.4 engine_module 关系裁决：与 `core::runtime` 的 ModuleDescriptor/ServiceFactory/EngineService 逐类型对照（调用面实测：Grep 结果入册），三选一判词（合并入 core::runtime / declared 分层 / 删除）。若判合并或删除，迁移切片入 M1。DoD：判词 + 调用面清单。

### M1 裁决执行（按 M0 判词裁剪，硬切换）

- 切片模式（每项独立提交）：engine_module 合并/删除的 `git mv`+调用方迁移；diagnostic_log 归属迁移（若判并入）；navigation 守卫落地；animation 债项 backlog 化（不实现，列条目带验收口径）。
- 验收：受影响族的聚焦测试 + `cargo test -p zircon_runtime --lib --locked` 全量无回归；结构守卫（02 的 root_entries.rs 同族）同步更新 crate 根口径。
- DoD：四族在 crate 根的席位与判词一致；无未裁决残留。

### 测试阶段（milestone-first）

- M0：纯审计，`git status --porcelain` 仅 docs。
- M1：`cargo test -p zircon_runtime --lib animation --locked`、`--lib navigation --locked`、`--lib diagnostic_log --locked`、`--lib engine_module --locked`（按词过滤）+ 全量 lib 回归；波及装配时 `cargo test -p zircon_app --locked`。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`14/2026-07-09-runtime-module-family-closeout-output-records.md`](14/2026-07-09-runtime-module-family-closeout-output-records.md)

## Code Review 建议 (2026-07-30)

### 与代码现状不符，需修订

- §「现状与证据」（2026-06-13 快照）navigation 一段「**navigation/**（9 个 `.rs` 文件）……`runtime/{baked_mesh,world_scan,avoidance,state,math,tests}.rs`」与当前代码不符，且与顶部 2026-07-19 sync 的 `navigation = 12` 自相矛盾。实测 `zircon_runtime/src/navigation/` 现有 `mod.rs`、`module.rs`、`runtime.rs`、`repath_budget.rs`、`operation/` 目录，`navigation/runtime/` 下有 `avoidance.rs`、`baked_mesh.rs`+`baked_mesh/`、`math.rs`、`state.rs`、`tests.rs`、`world_scan.rs`。建议把该段的「9 个文件」与文件清单更新为当前形态，并把新增的 `operation/`（folder-backed shared-operation integration owner）与 `repath_budget.rs` 纳入 M0.2 厚度判词，避免它们成为下一轮审计的「游离散件」——这正是本计划要防的问题。
- §「现状与证据」的四族文件计数需要与顶部 sync（animation = 28、navigation = 12、diagnostic_log = 7、engine_module = 8）对齐核对：`diagnostic_log/` 实测含 `diagnostics.rs`+`diagnostics/`、`level.rs`+`level/`、`sink.rs`+`sink/`、`mod.rs`、`platform.rs`、`settings.rs`、`timestamp.rs`；`engine_module/` 实测含 `mod.rs`、`engine_module.rs`、`engine_service.rs`、`service_factory.rs`、`contexts.rs`、`tests.rs`、`descriptors/`。body 里「engine_module（8 个 `.rs` 文件，含 `descriptors/names.rs`）」需核对 `descriptors/` 子目录当前是否仍为 `names.rs`（related_code 只列了 `engine_module/engine_module.rs`、`engine_module/service_factory.rs`，未列 `engine_service.rs`/`contexts.rs`）。

### 设计优化建议

- M0.2 navigation 判词提出「加一行结构守卫断言 navigation/ 不长出行为文件（文件数白名单）」。当前 `operation/` 的加入说明该白名单若已落地则需同步放宽，若未落地则 `navigation = 12` 相对初稿 9 已增长 3 个文件，正是白名单应捕获的情形。建议在状态节明确该文件数白名单守卫的当前基线值（对齐 `module_family_boundary` 的 `navigation = 12`），使「不长出行为文件」有可机判的锚点。
