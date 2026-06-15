---
related_code:
  - zircon_runtime/src/animation/manager.rs
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
  - zircon_runtime/src/core/diagnostics
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/zircon_runtime/animation/runtime.md
  - docs/zircon_runtime/navigation/runtime.md
  - docs/zircon_runtime/diagnostic_log/mod.md
  - docs/zircon_runtime/engine_module/relationship.md
status: in_progress
last_refined: 2026-06-15
---

# 14 运行时模块族完备性收尾（animation / navigation / diagnostic_log / engine_module）

扫尾型计划：lib.rs 根模块中尚无任何子计划认领的四个"小族"逐一裁决——架构对照、厚度核查、归属定位。每族工作量小，但放任不管会成为下一轮审计的"游离散件"（02 处理 core/ 根散件的同构问题在 crate 根重演）。

## 现状与证据（2026-06-13 实仓盘点）

- **animation/**（27 个 `.rs` 文件）：`manager/`、`sequence/`、`clip_event.rs`、`scene_hook/`、`module.rs`——有真实动画运行时（clip 事件、序列、场景钩子、graph/state machine、模块注册）。Runtime 14 M0.1 已在 `docs/zircon_runtime/animation/runtime.md` 对照 bevy_animation/Fyrox/godot，裁决 root motion 为 backlog，GPU skinning 与编辑器工具为非目标，并澄清 morph 已有 asset/scene property/sequence/graphics CPU baseline，但不是 animation manager 专属求解器。
- **navigation/**（3 个 `.rs` 文件，`runtime.rs` 约 32KB）：不是极薄注册层；`runtime.rs` 已包含 baked navmesh、A* pathfinding、sample/raycast、world agent tick、obstacle/agent avoidance。Runtime 14 M0.2 已在 `docs/zircon_runtime/navigation/runtime.md` 裁决：保留 crate-root fallback runtime，Recast/editor/baking 仍归 `zircon_plugins/navigation`。
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

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M0 | 0.1 animation 差距表 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/animation/runtime.md`；核验 `dev/bevy/crates/bevy_animation/src/{graph.rs,lib.rs,transition.rs}`、`dev/Fyrox/fyrox-animation/src`、`dev/Fyrox/fyrox-impl/src/scene/animation`、`dev/godot/scene/animation`；裁决 root motion backlog、GPU upload/editor tooling 非目标，并澄清 morph baseline 不属于 animation manager 专属求解器。Cargo 待独占窗口。 |
| M0 | 0.2 navigation 判词 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/navigation/runtime.md`；更正 `runtime.rs` 约 32KB 且包含 pathfinding/agent/obstacle/avoidance；核验 Godot navigation_2d/navigation_3d；登记未来守卫 `runtime_navigation_boundary_file_set_requires_doc_update`。Cargo 待独占窗口。 |
| M0 | 0.3 双诊断面分工 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/diagnostic_log/mod.md`；核验 Bevy `bevy_log` / `bevy_diagnostic` 分置；裁决 `diagnostic_log/diagnostics.rs` 为唯一 snapshot-to-log 桥。Cargo 待独占窗口。 |
| M0 | 0.4 engine_module 裁决 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/engine_module/relationship.md`；调用面覆盖 runtime/app plugin 组与 animation/asset/foundation/graphics/input/navigation/platform/scene/script/ui 模块；裁决 declared layering，不合并、不删除。Cargo 待独占窗口。 |
| M1 | navigation 文件集守卫 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_navigation_boundary_file_set_requires_doc_update`；守卫 `navigation/{mod.rs,module.rs,runtime.rs}` 文件集、fallback runtime 描述和边界文档锚点。本轮重核：`rustfmt --edition 2021 --check` 覆盖 `root_entries.rs`、`engine_module/tests.rs`、`diagnostic_log/diagnostics.rs`；Runtime 14 anchor scan 找到该守卫和 built-in fallback 锚；冲突标记/尾随空白扫描为空；`git diff --check` 仅 LF/CRLF 提示。独立 `rustc --edition 2021 --test root_entries.rs` 曾运行 10/10 通过；Cargo 待独占窗口。 |
| M1 | engine_module declared-layer 守卫 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/engine_module/tests.rs::engine_module_declared_layer_does_not_own_runtime_lifecycle`；守卫 `engine_module` 声明文件不引入 `register_module`/activation/shutdown/lifecycle state/registry storage/runtime state ownership。本轮重核：Runtime 14 anchor scan 找到该守卫与 declared layering 锚，`rustfmt --edition 2021 --check` 通过，冲突标记/尾随空白扫描为空，`git diff --check` 仅 LF/CRLF 提示。Cargo 待独占窗口。 |
| M1 | diagnostic_log 单桥接守卫 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/diagnostic_log/diagnostics.rs::diagnostic_log_snapshot_bridge_stays_single_owner`；守卫非桥接 `diagnostic_log` 文件不直接引用 `core::diagnostics` store/snapshot 类型，且 `core::runtime::diagnostics` 不反向依赖进程日志 sink。本轮重核：Runtime 14 anchor scan 找到该守卫与 `snapshot-to-log` 锚，`rustfmt --edition 2021 --check` 通过，冲突标记/尾随空白扫描为空，`git diff --check` 仅 LF/CRLF 提示。Cargo 待独占窗口。 |
| M1 | animation backlog/非目标守卫 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_animation_backlog_boundary_requires_doc_update`；守卫 animation 文档记录 root motion backlog、GPU skinning/render-graphics 归属、editor tooling 非目标，并绑定 `sequence_applies_mesh_renderer_morph_weight_track` 作为 morph property-track baseline 证据。本轮重核：Runtime 14 anchor scan 找到该守卫、root motion backlog 与 GPU skinning 锚，`rustfmt --edition 2021 --check` 通过，冲突标记/尾随空白扫描为空，`git diff --check` 仅 LF/CRLF 提示。独立 `rustc --edition 2021 --test root_entries.rs` 曾运行 10/10 通过；Cargo 待独占窗口。 |
| M1 | crate 根四族席位总守卫 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_14_module_family_root_seats_match_documented_judgements`；守卫 `animation` / `navigation` / `diagnostic_log` / `engine_module` 继续作为 crate-root module family 暴露，不在 `lib.rs` 扁平化 re-export，并要求四份 Runtime 14 镜像文档继续记录对应根席位判词。本轮只做静态验证：`rustfmt --edition 2021 --check zircon_runtime\src\tests\runtime_absorption\root_entries.rs` 通过；Cargo/rustc 独立测试待 active cargo/rustc lanes 清空。 |
| 横切 | 四族结构审计 owner | structure_audit_static_passed_cargo_pending | 2026-06-13 | 新增并接入 `runtime_structure_audits/module_family_boundary.py`，静态复核 `animation` / `navigation` / `diagnostic_log` / `engine_module` 四个 Runtime 14 crate-root 席位、镜像文档判词和既有 Rust guard 锚点。targeted audit: `expected_family_count = 4`, `animation = 27`, `navigation = 3`, `diagnostic_log = 7`, `engine_module = 8`, `root_seat_guard_present = true`, `animation_status_json_guard_present = true`, `animation_status_json_anchor_count = 8`, `missing_animation_status_json_anchors = []`, `module_family_guard_anchor_count = 7`, `missing_module_family_guard_anchors = []`, `missing_doc_anchors = []`, `file_count_mismatches = []`, `risks = []`。Cargo 仍待 active lanes 清空。 |
| 横切 | 四族镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_14_module_family_mirror_docs_match_structure_audit_counts`，锁定 Runtime 14 计划、runtime index、M0 review 与 runtime-interface convergence 必须同步 `module_family_boundary` 的 `expected_family_count = 4`、`animation = 27`、`navigation = 3`、`diagnostic_log = 7`、`engine_module = 8`、`root_seat_guard_present = true`、`animation_status_json_guard_present = true`、`animation_status_json_anchor_count = 8`、`missing_animation_status_json_anchors = []`、`module_family_guard_anchor_count = 7`、`missing_module_family_guard_anchors = []` 与 `risks = []`。该守卫不改四族生产代码；Cargo/rustc 仍待 active lanes 清空。 |
| 横切 | 四族总索引状态表闭环 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 本轮把 `Runtime 14 Module family 镜像文档守卫` 写入 runtime 总索引 `## 状态与产出记录`，并扩展 `runtime_absorption::plan_status::status_output_tables::runtime_index_status_output_records_recent_cross_plan_slices`，要求总索引记录 `runtime_14_module_family_mirror_docs_match_structure_audit_counts`、`module_family_boundary`、standalone rustc 13/13 与 `module-family Cargo/rustc gates pending`。验证：`rustfmt --edition 2021 --check` 通过；`runtime_absorption::root_entries` standalone rustc 13/13 通过；状态表 harness 1/1 通过；Python direct `module_family_boundary_audit` 与 aggregate Runtime 14 assertions 通过；conflict/trailing scans 通过。 |
| 横切 | 四族 Cargo gate 审计元数据守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | `module_family_boundary` 新增 `cargo_gate_anchor_count = 5`、`missing_cargo_gate_anchors = []`，把 Runtime 14 待验证门槛提升为可审计元数据：`cargo test -p zircon_runtime --lib animation --locked`、`cargo test -p zircon_runtime --lib navigation --locked`、`cargo test -p zircon_runtime --lib diagnostic_log --locked`、`cargo test -p zircon_runtime --lib engine_module --locked`、`cargo test -p zircon_runtime --lib --locked`。`runtime_14_module_family_mirror_docs_match_structure_audit_counts` 已扩展为要求 Runtime 14、总索引、M0 review 与 runtime-interface convergence 同步这些字段；module-family Cargo/rustc gates pending。 |
| 横切 | 四族 guard anchors 审计同步 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | `module_family_boundary` 与 `runtime_14_module_family_mirror_docs_match_structure_audit_counts` 现在统一锁定 Runtime 14 的 7 个四族守卫锚：`runtime_animation_backlog_boundary_requires_doc_update`、`runtime_navigation_boundary_file_set_requires_doc_update`、`diagnostic_log_snapshot_bridge_stays_single_owner`、`engine_module_declared_layer_does_not_own_runtime_lifecycle`、`runtime_14_module_family_root_seats_match_documented_judgements`、`runtime_14_module_family_mirror_docs_match_structure_audit_counts`、`runtime_animation_status_json_boundary_sanitizes_non_finite_values`；当前 `module_family_guard_anchor_count = 7`、`missing_module_family_guard_anchors = []`。验证：rustfmt check、Python py_compile、direct `module_family_boundary_audit`、aggregate Runtime 14 + plan-status assertions、standalone root_entries 13/13、standalone status-output 2/2；`animation` / `navigation` / `diagnostic_log` / `engine_module` / full lib Cargo gates pending。 |
| M1 | Cargo 验证窗口探测 | cargo_deferred_active_lane | 2026-06-14 | 10:26 后检测到外部 `cargo test -p zircon_runtime --lib --no-default-features --features core-min ... tree_view_pointer` 正在使用 Cargo/rustc lane，目标目录为 `E:\cargo-targets\zircon-editor-ui-tree-drag-reorder-coremin-0614`；本轮未启动 Runtime 14 的五条真实 Cargo gate，避免并发污染验证结果。补充通过不占 Cargo lane 的 `runtime_absorption::plan_status::cargo_gates::runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass` standalone 守卫 1/1；`cargo test -p zircon_runtime --lib animation --locked`、`cargo test -p zircon_runtime --lib navigation --locked`、`cargo test -p zircon_runtime --lib diagnostic_log --locked`、`cargo test -p zircon_runtime --lib engine_module --locked` 与 `cargo test -p zircon_runtime --lib --locked` 仍待空闲窗口。 |
| M1 | animation Cargo gate 尝试 | cargo_blocked_external_compile_drift | 2026-06-14 | 空闲窗口后运行 `cargo test -p zircon_runtime --lib animation --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-14-module-family-0614 --message-format short --color never -- --nocapture`。命令进入 `zircon_runtime` lib-test 编译，但未执行到 Runtime 14 animation 测试；最低失败层是共享 lib-test 编译层，当前错误属于活跃 render/UI 编译漂移：`SKINNED_MESH_MAX_JOINT_MATRICES` 缺失、`RenderPostProcessEffectStackSettings` 未声明、`Ui` table reducer `String`/`&str` 类型不匹配、`AdvancedProfileRuntimePlan: Default` 未满足、`ViewportCameraSnapshot.temporal_jitter` 初始化缺失。未改 animation/navigation/diagnostic_log/engine_module 生产代码；`navigation` / `diagnostic_log` / `engine_module` / full lib Cargo gates pending。 |
| M1 | animation Cargo gate 修复与复验阻塞 | cargo_recheck_blocked_external_ui_compile_drift | 2026-06-14 | 11:32 空闲窗口重跑 `cargo test -p zircon_runtime --lib animation --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-14-module-family-0614 --message-format short --color never -- --nocapture`，本次已越过共享编译层并运行 animation 过滤测试，结果为 31 passed; 3 failed。最低失败层分为：`AnimationPlayerRuntimeStatus` JSON roundtrip 对 `NaN` 输出 `null` 且无法按 `f32` 读回；`animation_physics_absorption` 仍按旧 cutover 期待删除 `zircon_runtime/src/animation`，但 Runtime 14 当前判词是 animation 继续作为 runtime-owned module family，`zircon_plugins/animation/runtime` 只做 plugin metadata/runtime-system wrapper；physics 守卫仍查 `manager.rs`，而实现已拆到 `manager/service.rs`。已修 `AnimationPlayerRuntimeStatus::sanitized_snapshot`、状态字段 serde 有限化、`runtime_status_reports_player_rig_and_gpu_readiness` 期望、以及 animation/physics 吸收守卫的当前边界判词，并更新 `docs/zircon_runtime/core/framework/animation.md` 与 `docs/zircon_runtime/animation/runtime.md`。复验 `runtime_status_reports_player_rig_and_gpu_readiness` 与 `animation_physics_absorption` 过滤项未执行到目标测试，因为外部 editor UI 编译漂移先失败：`UiInputDispatchDiagnostics.capture_started` / `capture_released` 字段缺失以及 `zircon_runtime/src/ui/surface/surface/default_interactions/table.rs:257` move 后借用；同时当前存在 `cargo test -p zircon_runtime --lib --no-default-features --features core-min ... table_column_resize` 活动通道。`navigation` / `diagnostic_log` / `engine_module` / full lib Cargo gates pending。 |
| M1 | animation runtime-status JSON 边界守卫 | json_boundary_static_passed_cargo_pending | 2026-06-14 | 新增 `zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_animation_status_json_boundary_sanitizes_non_finite_values`，把 `AnimationPlayerRuntimeStatus::sanitized_snapshot`、`AnimationRuntimeStatus::sanitized_snapshot`、`serialize_sanitized_non_negative_real`、`deserialize_sanitized_non_negative_real`、`serialize_normalized_real`、`deserialize_normalized_real` 与 `docs/zircon_runtime/core/framework/animation.md` 的 JSON boundary 文档锚点纳入 Runtime 14 守卫；`module_family_boundary` 现在报告 `animation_status_json_guard_present = true`、`animation_status_json_anchor_count = 8`、`missing_animation_status_json_anchors = []`。该切片只锁定已修 DTO 边界；focused Cargo 仍待 UI/render 编译漂移和活动 Cargo lane 清空后复跑。 |
| M1 | animation runtime-status focused recheck timeout | cargo_recheck_timeout_no_result | 2026-06-15 | 空闲窗口运行 `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-14-module-family-0615 --message-format short --color never runtime_status_reports_player_rig_and_gpu_readiness -- --nocapture`，904s 后仍未执行到目标测试且无测试结果；残留 cargo/rustc processes were stopped。该尝试不改变 `AnimationPlayerRuntimeStatus::sanitized_snapshot` 修复结论，也不提升 Runtime 14 Cargo gate；`animation` / `navigation` / `diagnostic_log` / `engine_module` / full lib Cargo gates pending。 |

基线：animation 27 个 `.rs` 文件 / navigation 3 个 `.rs` 文件（`runtime.rs` 约 32KB）/ diagnostic_log 7 个 `.rs` 文件 / engine_module 8 个 `.rs` 文件（含 `descriptors/names.rs`）（2026-06-13 静态盘点）。Runtime 14 M0/M1 未跑 Cargo，因为同工作区有其他会话的 cargo/rustc 进程占用；M1 当前落地 navigation、engine_module、diagnostic_log、animation 结构/边界守卫、crate 根四族席位总守卫和静态验证。

## 风险与协调

- engine_module 裁决若判合并，波及 `core::runtime` 公共面——与 02 的 root_entries 守卫和 `20260604-1232` 会话口径对齐后执行。
- diagnostic_log 迁移（若发生）触及 06/07 的日志消费点（`write_log`、诊断采集）——迁移切片前枚举调用方并通知两计划。
- animation 的 scene_hook 与 08 的观察者语义、03 的 stage 挂接相邻：差距表只裁决不动代码，动代码时与两计划错峰。
