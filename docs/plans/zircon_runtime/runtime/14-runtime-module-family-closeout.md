---
related_code:
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/sequence.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/animation/module.rs
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
last_refined: 2026-08-01
---

# 14 运行时模块族完备性收尾（animation / navigation / diagnostic_log / engine_module）

2026-08-30 current-source owner sync：`module_family_boundary` 报告 `expected_family_count = 4`、`animation = 21`、`navigation = 16`、`diagnostic_log = 32`、`engine_module = 8`，全部 missing/file-count/risk 列表为空。计数包含各族目录下的嵌套测试与 folder-backed owner；它只用于发现未同步的结构变化，不代表行为完成度。navigation 的 `operation/{mod,handler,registration}.rs`、`repath_budget.rs`、`runtime/baked_mesh/{query_scratch,spatial_index}.rs` 与 `runtime/state/repath_entry_tests.rs` 已纳入当前 owner 镜像，不改变 fallback runtime manager、crate-root seat 或 editor/plugin 边界。下方 9-file/28-file/7-file 记录保留为历史证据；当前镜像只由本计划、Runtime14 当前编号记录和对应模块文档维护，不再要求 Runtime15 历史归档复制最新计数。

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

## Code Review 处理结果 (2026-08-01)

### 已处理

- 顶部 current-source sync、Python 结构审计与 Rust 镜像守卫均已采用当前基线：animation = 21、navigation = 16、diagnostic_log = 32、engine_module = 8；navigation 的 `operation/{mod,handler,registration}.rs`、`repath_budget.rs`、`runtime/baked_mesh/{query_scratch,spatial_index}.rs` 与 `runtime/state/repath_entry_tests.rs` 已纳入 owner 镜像。
- 「现状与证据」中的 animation = 28、navigation = 9、diagnostic_log = 7 保留为 2026-06-13 历史快照，不再作为当前源码事实或验收基线；当前文件清单以顶部 sync、模块文档与结构守卫为准。

### 仍开放

- 后续 navigation 结构增长必须同时更新 `module_family_boundary` 白名单与本计划 current-source 镜像；文件计数只能证明结构边界，不能替代行为验收。
