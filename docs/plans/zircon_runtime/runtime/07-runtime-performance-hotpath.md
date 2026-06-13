---
related_code:
  - zircon_runtime/src/scene/ecs/query/query_state
  - zircon_runtime/src/scene/ecs/query/cached_query_iter.rs
  - zircon_runtime/src/scene/ecs/change_detection
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/diagnostics
  - zircon_runtime/src/core/diagnostics/profiling/macros.rs
  - zircon_runtime/src/core/time.rs
  - zircon_runtime/src/dynamic_api/session/tests.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/render/index.md
  - .codex/sessions/20260611-0416-rendering-10fps-analysis.md
status: in_progress
last_refined: 2026-06-14
---

# 07 runtime 侧性能热路径

## 现状与证据（2026-06-12 重核）

- 实测 ~10fps（Vulkan/nVidia 1280x720）：230 draws、231 次 pre-draw `vkCmdCopyBuffer`、31 个 render pass、SSR pyramid 重负载（RenderDoc 证据，`.codex/sessions/20260611-0416`）。
- 已落地的修复（10fps 会话，**不得回退**），且已有可模仿的计数断言测试范本（2026-06-12 实测）：
  - `render_product_streamer_reuses_material_uniforms_for_unchanged_revision`（`graphics/scene/render_product_material_property_tests.rs:99`）
  - `render_framework_skips_advanced_postprocess_work_when_effects_are_disabled`（`graphics/tests/render_framework_post_process_submit.rs:16`）
  - `render_framework_reuses_frame_history_handle_for_compatible_submissions`（`graphics/tests/render_framework_bridge.rs:550`）
  - M1 的新计数测试照此 `*_reuses_*`/`*_skips_*` 命名与断言模式。
- 取证阻塞：权威 FPS 被 ZrVM 断言挡住（修复归子计划 06 M1，现场已精确定位至 `real_backend/instance.rs` 空参数 marshalling）；profiling 构建（`--profile profiling`，根 `Cargo.toml:39` 已有 `[profile.profiling]` 定义）两次超时。
- 诊断基建实测：`core/diagnostics/` 已有 store/collect/snapshot/render_stats_store/profiling 模块族（animation/physics/render 各有分区文件）——计数走该通道，无需新基建；FPS/帧时间诊断常量已在 `core/time.rs:6-12`（`time.fps`、`time.frame_time`、`time.fixed_steps`、`time.frame_count`）。
- tracing span 现状（2026-06-12 二次细化实测）：span 仅经 `core/diagnostics/profiling/macros.rs` 的 `zr_profile` 宏族提供（:8/:31/:52，含 `zircon.profile.frame` 流），但**全仓使用点为 0**（Grep `zr_profile` 10 处命中全在宏定义文件自身）——帧分解三段（update/extract/submit）span 需从零铺设，M0 切片 0.3 没有"已有的先用"的余地。
- **ECS 查询缓存已部分存在（矫正）**：`scene/ecs/query/` 下已有 `query_state/` 目录与 `cached_query_iter.rs`——M1 的计数诊断必须先审计既有缓存命中率，M2 不得假设"查询无缓存路径"。
- 权威 FPS 测试位确认：`vampire_project_session_reports_runtime_fps_and_render_work` 在 `dynamic_api/session/tests.rs`。
- 已确认健康项：服务解析为强类型键 `HashMap<RegistryName, ServiceEntry>`，非每帧字符串查找。
- 分工：draw 提交侧骨架性能（RDG transient 资源、MeshDrawCommand 缓存、GPUScene、剔除）**全部归 render 计划 01–04**；本计划只管 runtime 侧上游：ECS、extract 构建、asset 每帧成本、UI/animation 系统成本、诊断基建。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- tracy profiler 本体源码（`profiling-tracy` feature 对接语义与 span 协议）— `dev/tracy`
- bevy 诊断路径/帧时间平滑/帧计数（M1 计数项命名与平滑窗口对照）— `dev/bevy/crates/bevy_diagnostic/src/{diagnostic.rs,frame_time_diagnostics_plugin.rs,frame_count.rs}`
- Godot 性能监视器单点（引擎级性能计数的收口形态对照）— `dev/godot/main/performance.{h,cpp}`

## 目标

1. 权威性能基线：真实 vampire 工程 FPS + 帧分解（update/extract/submit 占比），可重复采集（同命令二次偏差 < 20%）。
2. ECS 与 extract 上游的每帧分配/拷贝可计数、可断言，定向消除已证实热点。
3. profiling 工具链可用：profiling 构建不超时，tracy 或 chrome trace 至少一条走通。

## 非目标

- 不引入新 benchmark 依赖（criterion 等）；用现有 diagnostics 与聚焦验收测试。
- 不做 render graph/draw 提交优化（render 计划地盘）；不做投机优化——每项优化必须先有计数或 profile 证据。
- 不改 `SystemStage`/调度语义（归子计划 03）。

### 全局硬约束（继承总计划 §4，违反即返工）

- 硬切换不留兼容层；不新增 crate；渲染骨架内容归 render 计划 01-08；非网络语义 server 命名是 blocker。

## 执行前检查清单

1. **硬性前置**：重读 `.codex/sessions/20260611-0416-rendering-10fps-analysis.md` 最新状态——该会话仍活跃，graphics/runtime 的 worktree 改动是 live state，只做聚焦编辑，**禁止回退**。
2. 前置依赖确认：子计划 06 M1（ZrVM 空参数修复）是否落地——未落地则 M0 走 fallback 基线（见风险节）。
3. worktree 脏文件检查：`git status --porcelain -- zircon_runtime/src/scene/ecs/ zircon_runtime/src/core/diagnostics/ zircon_runtime/src/dynamic_api/`。
4. 事实重核：
   - `grep -n "profile.profiling" Cargo.toml`（核 :39）
   - `grep -rn "zr_profile" zircon_runtime/src --include=*.rs | wc -l`（span 使用点基线）
   - `ls zircon_runtime/src/scene/ecs/query/`（核 query_state/cached_query_iter 仍在）
5. 磁盘与构建配额：profiling 构建前核对共享 `CARGO_TARGET_DIR` 剩余空间（CLAUDE.md 磁盘政策，≤50GB 先清理）。

## 里程碑

### M0 基线取证（依赖 06-M1 解锁；未解锁走 fallback）

#### 切片 0.1 权威 FPS 采集

- 目标文件：无代码改动；产出写本计划状态节 + `docs/zircon_runtime/core/`（若 03 的 `frame_schedule.md` 已建则共用）。
- 改动形态：跑 `vampire_runtime_perf` 真实场景，采集 `runtime_diagnostics`/`time.fps`（`TIME_FPS_DIAGNOSTIC`，core/time.rs:12）日志；同命令二次采集验证偏差 < 20%。
- 命令：`cargo test -p zircon_runtime --lib vampire_project_session_reports_runtime_fps_and_render_work --features zr-vm-real-backend --locked -- --nocapture --test-threads=1`（测试位：`dynamic_api/session/tests.rs`，已核实存在）。
- 验收：FPS 数值 ×2 次 + 偏差比记入状态节。
- DoD：基线表三列（FPS / 命令 / 日期）非空。

#### 切片 0.2 profiling 构建超时破解

- 目标文件：无代码改动（构建配置实验）；结论写状态节。
- 改动形态：用 `tools/dev-fast-build.ps1` 共享 target 目录 + 包内最小 feature 组合复现两次超时，记录瓶颈段（链接/重编译范围）；必要时为 `[profile.profiling]`（根 Cargo.toml:39）裁剪 feature 组合（裁剪项执行时定稿，落子计划 01 的选型文档若涉及依赖）。
- 命令：`python tools/zircon_build.py --targets runtime` 或 `./tools/dev-fast-build.ps1 -Profile client -Action check`；profiling 特性组合执行时从 `zircon_runtime/Cargo.toml` 的 `profiling-tracy`/`profiling-chrome` 行核验。
- 验收：profiling 构建在记录的配置下完成（耗时入状态节），或瓶颈定位报告（哪个 crate/链接段超时）。
- DoD：可复现的 profiling 构建命令落状态节。

#### 切片 0.3 帧分解三段 span

- 目标文件：`dynamic_api/session.rs`（tick_frame 段 :548 与 extract 构建点 :695 `fn current_extract(&self) -> RenderFrameExtract`——2026-06-12 实测这是 runtime 侧唯一生产构建点，其余 Grep 命中全为 graphics 测试夹具）、`scene/ecs/schedule_runner.rs`（stage 执行段）。
- 改动形态：复用 `zr_profile` 宏族（`core/diagnostics/profiling/macros.rs:8-52`）给 update（ECS schedule）/ extract 构建 / graphics submit 三段补最小 span（已有的不重复加）；产出一帧三段占比。
- 调用方迁移：无公共面变化。
- 验收：trace（tracy 或 chrome 任一）中三段 span 可见且首帧分解占比记入状态节。
- DoD：帧分解表（三段耗时与占比）非空。

#### M0 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`（span 切片后）
- 基线命令二次复跑（偏差 < 20% 即验收）
- 验收证据：FPS 基线、profiling 构建记录、帧分解表三件齐备写入状态节。

### M1 ECS 与 extract 计数诊断

#### 切片 1.1 计数点铺设

- 目标文件：`scene/ecs/query/query_state/`（archetype 匹配缓存命中/未命中——**先审计既有缓存再加计数**）、`scene/ecs/change_detection/`（标记扫描量）、extract 构建点（`RenderFrameExtract` 的 clone/Vec 重建次数、输出字节量）；计数登记走 `core/diagnostics` 既有 store。
- 改动形态：计数项（草案）：`ecs.query.archetype_cache_hits` / `..._misses`、`ecs.change_detection.scanned_marks`、`extract.rebuild_clones`、`extract.output_bytes`。无行为改动，只读计数。
- 调用方迁移：无。
- 验收：计数经诊断 snapshot 可读（参照 `core/diagnostics/snapshot.rs` 既有读口）。
- DoD：四组计数在 vampire 场景下输出非零且数值稳定。

#### 切片 1.2 计数断言测试族

- 目标文件：`scene/tests/`（ecs 计数测试）、`graphics` 侧不动（归 render 计划）。
- 改动形态：照 10fps 会话已落的 `*_reuses_*`/`*_skips_*` 模式（范本：`render_product_streamer_reuses_material_uniforms_for_unchanged_revision`）写固定场景计数断言：
  - `query_state_reuses_archetype_matches_across_unchanged_frames`
  - `change_detection_scan_skips_unmarked_archetypes`
  - `frame_extract_rebuild_skips_unchanged_entities`（若现状全量重建，此测试先以"记录现状值"形式落地，M2 优化后收紧阈值）
- 调用方迁移：无。
- 验收：三测试落地（含"现状值锚定"形式）。
- DoD：`cargo test -p zircon_runtime --lib ecs_query --locked` 与 `extract --locked` 全绿。

#### 切片 1.3 热点清单

- 目标文件：本计划状态节。
- 改动形态：按计数数值排序 top 热点清单（预期嫌疑：extract 全量重建无 dirty 跳过、UI/animation 系统全量遍历、asset 每帧轮询残留），每项带数值证据与"归本计划/归 render 计划"分流判词（buffer copy 风暴属 render 计划 02）。
- 验收：清单每项有计数证据；无证据项不得入 M2。
- DoD：清单落状态节且分流判词齐备。

#### M1 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`
- `cargo test -p zircon_runtime --lib extract --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib ecs_query --locked -- --nocapture`
- `runtime_absorption::plan_status::cargo_gates::runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation` 在 extract/ecs_query/performance profiling/FPS gates 完整通过前保持本计划 `in_progress`，并要求 M0.3/M1.1/M1.2/M1.3 状态行继续带 Cargo/profiling/FPS 待验证语言。
- 验收证据：计数测试族 + 热点清单（带数值）。

### M2 定向优化（仅做 M1 清单有证据项）

#### 切片模式（每项独立切片，按 M1 清单裁剪）

候选模式（执行时按清单实例化为具体切片，每片独立提交）：

1. **extract 增量化**：未变更实体/组件跳过重提取（对照 bevy extract 的 changed 过滤，`dev/bevy/crates/bevy_render/src/extract_plugin.rs`）；复用上一帧缓冲而非重分配。验收：`frame_extract_rebuild_skips_unchanged_entities` 阈值收紧 + before/after 计数对比。
2. **查询缓存收紧**：仅当 M1 证实 `query_state` 既有缓存命中率低时——archetype 变更时增量失效。验收：`query_state_reuses_archetype_matches_across_unchanged_frames` 阈值收紧。
3. **asset 每帧成本**：事件驱动替代残留轮询（与子计划 04 M3 的轮询盘点共用清单）；材质/网格 revision 比对提前到管线入口（与已落地 `ensure_material` 缓存同口径推广，范本测试同名族）。
4. 每项优化配 before/after 计数对比 + 行为一致性测试，逐项独立切片提交；优化不得回退 10fps 会话已落修复。

- DoD（每片）：计数改善有数值、行为一致性测试绿、`cargo test -p zircon_runtime --lib --locked` 无回归。

#### M2 测试阶段（milestone-first）

- 每片：聚焦计数测试 + `cargo test -p zircon_runtime --lib --locked` 全量无回归
- 里程碑末：重跑 M0 基线命令对比 FPS/帧分解；基线对比表（优化项、计数变化、FPS 变化）写入状态节
- 文档：受影响模块的 `docs/zircon_runtime/**` 镜像文档更新

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M0 | 0.1 权威 FPS | 待开始 | — | — |
| M0 | 0.2 profiling 构建 | 待开始 | — | — |
| M0 | 0.3 帧分解 span | frame_spans_static_passed_trace_pending | 2026-06-13 | 已落 frame breakdown span：`runtime_frame_time_update` / `runtime_frame_update` 写在 `dynamic_api/session.rs::tick_frame()`，`runtime_frame_extract` 写在 `dynamic_api/session/extract.rs::current_extract()`，`runtime_frame_submit` 写在 `dynamic_api/runtime_loop.rs::{submit_extract_with_ui,present_extract_with_ui}`，`runtime_frame_schedule_stage.<SystemStage>` 写在 `scene/ecs/schedule_runner.rs::run_stage(...)`；`SceneScheduleRunner` span 使用 `profile_dynamic_scope!("runtime", "frame", format!("runtime_frame_schedule_stage.{stage:?}"))`，只在 profiling features 下 materialize stage 名称，不改变 borrowed sorted-step iterator、deferred flush、hook 执行或 final consistency sweep 语义；`runtime_absorption::performance_hotspots::runtime_07_hotspot_inventory_requires_counted_evidence_before_m2` 已扩展锁定 schedule-runner span、Runtime 07 计划、hotspot inventory、dynamic session doc、ECS doc、M0 架构评审与总索引锚点，并通过 `runtime_absorption/mod.rs` 挂载；`rustfmt --edition 2021 --check zircon_runtime\src\dynamic_api\session.rs zircon_runtime\src\dynamic_api\session\extract.rs zircon_runtime\src\dynamic_api\session\extract_stats.rs zircon_runtime\src\dynamic_api\runtime_loop.rs zircon_runtime\src\dynamic_api\session\tests.rs` 先前通过，本轮新增 `rustfmt --edition 2021 --check zircon_runtime\src\scene\ecs\schedule_runner.rs zircon_runtime\src\tests\runtime_absorption\performance_hotspots.rs zircon_runtime\src\tests\runtime_absorption\mod.rs` 通过；锚点扫描、冲突标记/尾随空白扫描、tracked scoped `git diff --check`（仅 LF-to-CRLF warning）与 untracked no-index diff-check 通过；trace/profiling 构建验收仍待 render HZB Cargo blocker 与 active lanes 清除后复跑。 |
| M1 | 1.1 计数点 | scoped_counter_points_extract_implemented_cargo_blocked | 2026-06-13 | 已完成并验证两组本地计数口：1. `QueryState` 记录 `cache_hits` / `cache_misses` / `cache_rebuilds` / candidate / matched / revision，并通过 `QueryStateCacheStats::record_diagnostics(...)` 写入 `ecs.query.archetype_cache_hits` / `ecs.query.archetype_cache_misses` / `ecs.query.archetype_cache_rebuilds` / `ecs.query.candidate_entities` / `ecs.query.matched_entities`；2. `ChangeDetectionScanStats` 写入 `ecs.change_detection.scanned_marks` / `ecs.change_detection.added_matches` / `ecs.change_detection.changed_matches`。行为与诊断读回测试已通过：`cargo test -p zircon_runtime --lib query_state_cache_stats_record_reuse_and_rebuild_counts --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never -- --nocapture --test-threads=1` 1/1 通过；`cargo test -p zircon_runtime --lib change_detection_scan_stats_record_mark_checks_and_diagnostics --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never -- --nocapture --test-threads=1` 1/1 通过；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never` 通过（既有 warning set）；`rustfmt --edition 2021 --check` 覆盖 QueryState/change_detection touched 文件并通过；`git diff --check` 与冲突标记扫描通过（仅 LF-to-CRLF warning）；`query_state/mod.rs` 非空 174 行，低于 180 行预算。extract 构建点已补 `dynamic_api/session/extract.rs` 与 `session/extract_stats.rs`：`current_extract()` 每次构建后自动写入 `extract.rebuild_clones` / `extract.output_bytes`，`headless_session_capture_records_frame_extract_diagnostics` 断言 headless capture 读回 rebuild=1 且 output bytes 非零；`rustfmt --edition 2021 --check`、`git diff --check`、冲突标记扫描已覆盖 extract touched 文件并通过。Cargo 级验证暂未通过到本切片：`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never` 被活跃 render owner 的 HZB 代码阻塞，先报 `graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs` 缺 `HzbOcclusionCullReport`，以及 `scene_renderer_core_render_compiled_scene/render/render.rs` 两处 `expected &ShadowMapRenderer, found &HzbOcclusionCuller`。未完成项：M1.1 的 Cargo rerun、vampire 场景非零/稳定输出、QueryState/change detection 的帧级自动采集仍待后续切片。 |
| M1 | 1.1 QueryState 结构审计同步 | structure_audit_static_passed_cargo_pending | 2026-06-13 | `ecs_query_state_boundary` 已把 `query_state/stats.rs` 收敛为 Runtime 07 cache telemetry owner：`expected_module_count = 8`、`unexpected_modules = []`、`risks = []`；`docs/zircon_runtime/scene/ecs/query_state.md` 同步记录 8 个 folder-backed owner modules 与 `stats.rs` telemetry sidecar。已完成 `python -m py_compile` 与定向 `ecs_query_state_boundary_audit(...)` 结构断言；完整聚合审计在共享仓库上超出本轮轻量窗口，Cargo 运行仍随 Runtime 07 M1.1/M1.2 gate 待 active lanes 清空后补跑。 |
| M1 | 1.2 计数断言 | named_assertions_static_passed_cargo_blocked | 2026-06-13 | 三个命名断言测试已落地：`query_state_reuses_archetype_matches_across_unchanged_frames`（`zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs`）断言 unchanged world revision 下 QueryState cache hit 递增、miss/rebuild/revision/candidate/matched 稳定；`change_detection_scan_skips_unmarked_archetypes`（`zircon_runtime/src/scene/tests/ecs_change_detection.rs`）断言 stale/unmarked tick marks 扫描后 added/changed match 仍为 0 且诊断读回一致；`frame_extract_rebuild_skips_unchanged_entities`（`zircon_runtime/src/dynamic_api/session/tests.rs`）按计划先锚定现状值，两个 unchanged headless capture 均记录 `extract.rebuild_clones = 1` 且 `extract.output_bytes` 非零稳定。`rustfmt --edition 2021 --check` 已覆盖三份测试文件并通过；源码扫描确认测试名和 helper 存在；行数检查：`session/tests.rs` 965 行、`ecs_performance_acceptance.rs` 246 行、`ecs_change_detection.rs` 256 行。Cargo 验证未声明通过，仍待 render-owned HZB blocker（缺 `HzbOcclusionCullReport`；两处 `expected &ShadowMapRenderer, found &HzbOcclusionCuller`）清除后补跑。 |
| M1 | 1.3 热点清单 | inventory_scaffold_static_passed_pending_authoritative_values | 2026-06-13 | 新增 `docs/zircon_runtime/performance/hotspot_inventory.md` 与 `runtime_absorption::performance_hotspots::runtime_07_hotspot_inventory_requires_counted_evidence_before_m2`，把热点清单从空占位推进为有证据门槛的 M1.3 scaffold：无权威 runtime 数值不得进入 M2；extract 全量重建候选只以 `frame_extract_rebuild_skips_unchanged_entities` 的 `extract.rebuild_clones = 1` / stable `extract.output_bytes` 作为当前可计数 baseline，不宣称 top 排名；QueryState 当前证据为 128 entities / 8 repeated runs / 8 hits / 1 miss / 1 rebuild，暂不作为优化目标；change detection 当前证据为 6 scanned marks / 0 added / 0 changed，暂不作为优化目标；asset worker 仅有结构诊断与 `asset.worker.budgeted_threads`，未有每帧成本证据；10fps RenderDoc 的 230 draws / 231 `vkCmdCopyBuffer` / 31 render passes 明确分流至 render 计划 02/04。新增守卫将文档、Runtime 07 计划、总索引、QueryState/change detection/extract 测试和 10fps 证据锚点绑定在一起；新增 `runtime_absorption::plan_status::cargo_gates::runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation` 把 extract/ecs_query/performance profiling/FPS gates 固定为提升完成前的剩余验证门；`rustfmt --edition 2021 --check zircon_runtime\src\tests\runtime_absorption\performance_hotspots.rs zircon_runtime\src\tests\runtime_absorption\mod.rs` 通过；冲突标记/尾随空白/锚点扫描与 scoped `git diff --check` 通过（仅 LF-to-CRLF warning）；Cargo/rustc 因 active lane 占用未启动，待通道清空后补跑。 |
| M1 | 1.3 性能热路径结构审计镜像 | structure_audit_static_passed_cargo_pending | 2026-06-14 | `performance_hotpath_boundary` 已接入 Python 结构审计并覆盖 Runtime 07 frame span、QueryState/change-detection/extract/asset-worker 计数、hotspot inventory、测试锚、pending Cargo/profiling/FPS gate 与 Runtime 07 owner-budgeted optimization gate：source 10/10、guard/test 5/5、frame span anchors 9/9、QueryState telemetry anchors 13/13、change-detection telemetry anchors 9/9、extract telemetry anchors 10/10、asset-worker candidate telemetry anchors 5/5、hotspot guard anchors 16/16、Runtime 07 counter assertion anchors 12/12、doc anchors 16/16、pending gate anchors 5/5、stale top3 placeholder false；同时消费 `large_file_ownership_gate`，当前 1000 行阈值下 large-file owner gate 为 `migration-debt-present`、hotspots 41、debt groups 5、owner classes 5、unclassified 0，固定 `large production files remain above the owner budget` 的优化前置裁决。`risks = []` 仅代表 Runtime 07 结构镜像同步；`extract` / `ecs_query` / profiling / vampire FPS Cargo gates 仍待 active lanes 清空后补跑。 |
| M1 | 1.3 owner-budget gate 守卫同步 | owner_budget_guard_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::performance_hotspots::runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit`，把 `large-file-ownership-m1.md`、本计划、runtime index、`hotspot_inventory.md`、M0 review 与 interface-convergence mirror 的 owner-budget 摘要固定到同一组结构审计事实：threshold 1000、hotspots 41、debt groups 5、owner classes 5、unclassified 0，且拒绝 stale 33-hotspot、旧 owner-count 与已删除的 Hub `app/` 路径锚点。该守卫只锁定镜像一致性，不启动 M2 优化、不拆生产大文件；Cargo/rustc 因 active lanes 占用未启动，待通道清空后补跑。 |
| M2 | （按清单实例化） | 待开始 | — | — |

基线数值（取证期填写）：

- 权威 FPS：__（命令 ×2，偏差 __%）
- 帧分解：update __% / extract __% / submit __%
- `zr_profile` span 使用点数：0（2026-06-12 实测，10 命中全在 macros.rs 自身）
- real-backend 编译超时旁证（2026-06-12，06 计划状态节）：`vampire_project_session_starts_paused_until_start_button_click --features zr-vm-real-backend` 300s 编译超时——切片 0.2 的破解结论同时解锁 06 的 real-backend 回归
- 热点清单 top3：待权威 runtime 数值；当前只保留有计数证据的候选清单，见 `docs/zircon_runtime/performance/hotspot_inventory.md`，无权威 runtime 数值不得进入 M2。
- Runtime 07 owner-budgeted optimization gate：`performance_hotpath_boundary` 复用 `large_file_ownership_gate`，当前 1000 行阈值下 large-file gate 为 `migration-debt-present`，hotspots 41 / debt groups 5 / owner classes 5 / unclassified 0。M2 优化若触及大文件，必须先按 owner class 拆分或取得对应活跃 owner handoff；large production files remain above the owner budget 时不得把性能改动继续堆进同一个热点文件。
- ECS query cache telemetry baseline：`cache_hits` / `cache_misses` / `cache_rebuilds` / `candidate_entity_count` / `matched_entity_count` 已有本地 `QueryStateCacheStats`，并提供 `DiagnosticStore` 投影路径且聚焦测试通过；change detection baseline 已有 `ChangeDetectionScanStats.scanned_marks` / `added_matches` / `changed_matches` 投影路径且聚焦测试通过；extract baseline 已接入 `extract.rebuild_clones` / `extract.output_bytes` 自动诊断并有 headless 读回测试；M1.2 三个命名断言已源码/格式静态通过，Cargo rerun 与 vampire 场景非零/稳定输出待 render HZB 阻塞清除后补跑
- 10fps 会话已落修复锚（不得回退）：model uniform 缓存、`ensure_material` revision 复用、禁用 postprocess 跳过 executor——锚定测试见"现状与证据"节三个范本

## 风险与协调

- **硬性前置**：执行任何切片前重读 `.codex/sessions/20260611-0416-rendering-10fps-analysis.md` 最新状态——该会话仍活跃，worktree 改动是 live state，只做聚焦编辑，禁止回退。
- M0 依赖子计划 06 M1（ZrVM 修复）：若上游修复周期长，先用 fallback 诊断工程取"非权威"基线并显式标注局限（无 ZrVM 脚本路径），fallback 命令：去掉 `--features zr-vm-real-backend` 的同名测试（若 fallback 后端支持）或现有 `runtime_diagnostics` 测试族。
- 与 render 计划边界：M1 计数若把热点指向 draw 提交侧（231 次 pre-draw buffer copy 属此类），移交 render 计划 02（MeshDrawCommand/上载合并），不在本计划处理。
- profiling 构建可能与共享 `CARGO_TARGET_DIR` 的并行构建冲突（CLAUDE.md 磁盘政策：禁止并行重型构建）；M0 切片 0.2 安排在无其他构建窗口执行。
- 计数点铺在 `scene/ecs/query/query_state/` 与 `change_detection/`——若 10fps 会话 worktree 已触及这些文件，先对齐再动。
