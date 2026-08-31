---
related_code:
  - zircon_runtime/src/scene/ecs/query/query_state
  - zircon_runtime/src/scene/ecs/query/cached_query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/ecs/system/param_set.rs
  - zircon_runtime/src/scene/ecs/change_detection
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/scene/tests/mod.rs
  - zircon_runtime/src/core/runtime/tests/events/structure/event_bus/root_contract.rs
  - zircon_runtime/src/core/runtime/tests/events/structure/event_bus/publish.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/combat.rs
  - zircon_runtime/src/script/vm/gameplay_host/components.rs
  - zircon_runtime/src/script/vm/gameplay_host/input.rs
  - zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/navigation.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_runtime/src/script/vm/gameplay_host/values.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/worker.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests/combined_validation_tests.rs
  - tools/tests/test_runtime_shader_prewarm_test_structure.py
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_markdown.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/render/index.md
status: completed
last_refined: 2026-07-23
---

# 07 runtime 侧性能热路径

## 现状与证据（2026-06-12 重核）

- 实测 ~10fps（Vulkan/nVidia 1280x720）：230 draws、231 次 pre-draw
  `vkCmdCopyBuffer`、31 个 render pass、SSR pyramid 重负载。该历史 RenderDoc
  证据已固化到编号归档
  `07/2026-07-09-runtime-performance-hotpath-output-records.md`，不再依赖已退出活动流转的
  session note。
- 已落地的修复（10fps 会话，**不得回退**），且已有可模仿的计数断言测试范本（2026-06-12 实测）：
  - `render_product_streamer_reuses_material_uniforms_for_unchanged_revision`（`graphics/scene/render_product_material_property_tests.rs:99`）
  - `render_framework_skips_advanced_postprocess_work_when_effects_are_disabled`（`graphics/tests/render_framework_post_process_submit.rs:16`）
  - `render_framework_reuses_frame_history_handle_for_compatible_submissions`（`graphics/tests/render_framework_bridge.rs:550`）
  - M1 的新计数测试照此 `*_reuses_*`/`*_skips_*` 命名与断言模式。
- 取证阻塞：ZrVM binding 空参数 marshalling 修复已有历史回归证据，但 2026-07-11
  重核时先前记录的本地 ZrVM `lib/Debug` 与 `bin/Debug` 目录均已不存在，无法生成两次
  权威 Vampire FPS 样本。profiling library 构建已两次完成；optimized lib-test trace
  构建曾在磁盘降至约 1.09 GiB 时停止，尚未执行测试或生成 trace artifact。
- 诊断基建实测：`core/diagnostics/` 已有 store/collect/snapshot/render_stats_store/profiling 模块族（animation/physics/render 各有分区文件）——计数走该通道，无需新基建；FPS/帧时间诊断常量已在 `core/time.rs:6-12`（`time.fps`、`time.frame_time`、`time.fixed_steps`、`time.frame_count`）。
- tracing span 现状（2026-06-22 重核）：span 经 `core/runtime/diagnostics/profiling/macros.rs` 的 `profile_*` 宏族提供，M0.3 已在动态 session、runtime bridge、SceneScheduleRunner 和 render framework submit 路径落地 update/extract/submit、stage 与 render-framework 内部分段 anchors；权威 trace 仍待 profiling 构建/FPS gate 解锁后采集。
- **ECS 查询缓存已部分存在（矫正）**：`scene/ecs/query/` 下已有 `query_state/` 目录与 `cached_query_iter.rs`——M1 的计数诊断必须先审计既有缓存命中率，M2 不得假设"查询无缓存路径"。
- 权威 FPS 测试位确认：`vampire_project_session_reports_runtime_fps_and_render_work` 在 `dynamic_api/session/tests/frame_diagnostics.rs`。
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

1. **硬性前置**：重读编号归档
   `07/2026-07-09-runtime-performance-hotpath-output-records.md` 与最新 Runtime 07 编号记录，
   再通过 coordinator 查询近四小时 graphics/runtime 活动 owner；只做聚焦编辑，
   **禁止回退**。活动 session 只用于租约与协调，不作为永久验收输入。
2. 前置依赖确认：子计划 06 M1（ZrVM 空参数修复）是否落地——未落地则 M0 走 fallback 基线（见风险节）。
3. worktree 脏文件检查：`git status --porcelain -- zircon_runtime/src/scene/ecs/ zircon_runtime/src/core/runtime/diagnostics/ zircon_runtime/src/dynamic_api/`。
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
- 命令：`cargo test -p zircon_runtime --lib vampire_project_session_reports_runtime_fps_and_render_work --features backend-zr-vm --locked -- --nocapture --test-threads=1`（测试位：`dynamic_api/session/tests/frame_diagnostics.rs`，已核实存在）。
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
- 改动形态：计数项（草案）：`ecs.query.archetype_cache_hits` / `..._misses`、`ecs.change_detection.scanned_marks`、`extract.rebuild_clones`、`extract.output_bytes`、`extract.cache_hits`、`extract.cache_misses`。无行为改动，只读计数。
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
- `runtime_absorption::plan_status::cargo_gates::runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation` 曾在 extract/ecs_query/performance profiling/FPS gates 完整通过前保持本计划 `in_progress`；2026-07-12 的 M0/M1 durable evidence 已关闭设计与行为门，当前共享工作区全包重编译阻塞按外部 owner validation blocker 记录，不再冒充 Runtime 07 未实现项。
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

### M3 World hierarchy dirty-frontier repair

- M2 的 F459 generation-topology 候选经独立审查发现三个 P1：有效的结构化重挂接仍触发
  `O(N)` hierarchy validity 扫描；原始 `get_mut::<Hierarchy>` 修复无效边后未发布被修复端点的
  derived state；检查字段失效在 topology 的 dense projection 过期时临时遍历全世界。
- M3 保持一个唯一、稳定顺序的 World-owned parent-to-children topology。结构化
  `set_parent_checked` 只原子更新 changed edge、受影响子树和检查字段；它不走 raw hierarchy
  escape hatch 的全量 validity path。泛型 `Hierarchy` 写入和 raw mutable borrow 仍保守地触发
  validity rebuild；若该 rebuild 修复任何边，必须将 active、world matrix、NodeCache 及 render
  前沿提升为全量，以免发布旧端点。
- 拓扑读取以每父节点的有序邻接表完成，避免为一次 reparent 重建全局 dense child range 或临时
  traversal map。此选择与 Unreal `USceneComponent` 的 attached-children ownership 和 changed
  component propagation 一致；它以局部 `O(log siblings + affected_subtree)` 更新取代 `O(N)`
  projection refresh。
- 验收：1k 与 100k 结构化 reparent 后 hierarchy snapshot/validity/topology source rebuild 都为
  `0`，active/world matrix 只访问 changed subtree，NodeCache 父节点行正确；raw cycle repair 后
  A/B 的 parent、matrix、active 和缓存行全部正确。必须用托管 Windows Cargo 验证与独立复审，
  不以静态检查或墙钟阈值替代访问计数。

#### M3 测试阶段（milestone-first）

- `derived_state` 聚焦回归覆盖 structured reparent、raw cycle repair、1k/100k 计数与 inspection
  subtree 无临时全局 traversal。
- 使用非 C 盘 target 的托管验证收集编译和测试结果；记录 validation ticket、计数结果及任何
  pool/外部 owner 阻塞，未经通过不得提交。

### M4 F459 repair execution

**Dependencies:** M2

- 执行 M3 已审阅的 World-owned ordered adjacency topology、structured versus raw hierarchy
  invalidation split 和 repair-frontier publication。
- M2 的 `world-derived-state-full-rebuild` failure 生命周期转交到本节点；只有通过 M3 审查
  定义的行为和计数验收、托管 Windows Cargo 验证及独立复审后才能关闭该 failure 并提交。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`07/2026-07-09-runtime-performance-hotpath-output-records.md`](07/2026-07-09-runtime-performance-hotpath-output-records.md)
- 最新资源与证据归属记录：
  [`07/2026-07-11-runtime07-durable-performance-evidence-and-resource-gate.md`](07/2026-07-11-runtime07-durable-performance-evidence-and-resource-gate.md)
- 完成状态：`frame_spans_trace_accepted_completed`、
  `scoped_counter_points_runtime_published_completed`、
  `named_assertions_behavior_accepted_completed`、`authoritative_inventory_completed`。
- 完成守卫：
  `runtime_07_performance_hotpath_records_completed_authoritative_validation`。
- 当前状态：M0.2 profiling library build 与 M0.3 direct runtime-frame trace artifact
  execution 已接受；M0.1 双次 Vampire FPS 已于 2026-07-12 完成，不以 compile-only、
  headless trace 或历史 10fps 数据替代。M0.3 current-source 精确命令在兼容
  `Cargo.lock` 下用 67m59s 完成优化 lib-test 构建，随后 1/1 通过（12.30s）；测试在
  临时输出目录中生成并读取 native/Perfetto timeline、hotspots 与 summary，逐项断言
  `submit_runtime_frame`、`render_frame_with_pipeline`、`DepthPrepass`、`depth-prepass`
  后按测试契约清理临时目录，不把已清理文件冒充持久产物。
  current-source `performance_hotspots` 独立门禁已由 15/27 收敛到 28/28，具体证据只读
  编号归档；`runtime_07_performance_guards_use_durable_evidence_not_session_notes` 禁止
  `.codex/sessions/` 路径族回流。
- 2026-07-11 当前产物复用复核：ZrVM 仓库没有当前 `build/`，
  `ZR_VM_RUST_BINDING_LIB_DIR` 为空，D/E/F target roots 与本地 Cargo 目录均无当前
  `zr_vm_rust_binding.lib/.dll`。后续扩大搜索范围后在
  `.codex/tmp/aot-clean-verify-20260622-121531` 找到一对旧 import library/runtime DLL；
  两者实际生成于 2026-06-11，复制快照的 `CMakeCache.txt` 仍指向已不存在的
  `E:/Git/zr_vm/build-msvc`，而当前 ZrVM HEAD 为 2026-07-09 的
  `2eb70efa143c44c9acc91e002f9f054f54e9f588`，因此该旧对也不能作为当前 FPS 链接证据。
  M0.3 测试受 `profiling` 与
  `profiling-chrome` 双 feature 约束；五个现存普通 Runtime test binaries 的 `--list`
  均不包含该测试，因此不能复用为 trace execution。资源复核为 C 12.94、D 16.48、
  E 12.56、F 2.49 GiB，12 个外部 cargo/rustc 进程活跃；未启动新构建或终止外部进程。
  协调器识别出 7 个超过 1 小时的 released lane，但安全清理执行因
  `maintenance_unauthorized` 被拒绝；没有绕过权限或手工删除目录。
- 资源恢复后的 current-source M0.3 重跑已成功，因此旧的 trace 资源阻塞不再是当前
  状态。首次兼容锁文件构建快照在 98m55s 后执行并暴露 `preview-sky` transient
  `scene-depth` load-before-producer；其构建期间 `mesh.rs` 的 pass 顺序被外部会话改写。
  对固定哈希的当前源码重新构建后，精确测试 1/1 通过，说明该失败属于中途源码快照，
  不再作为 current-source 缺陷。M0.1 仍单独等待当前 ZrVM import library/runtime DLL，
  这是 Runtime 07 唯一剩余的权威基线阻塞。
- 2026-07-11 M0.1 恢复审计：C/D/E/F 可用空间约为
  11.64/26.85/55.85/1.17 GiB；D 盘仍低于 50 GiB Cargo 门槛，E 盘虽恢复到门槛以上，
  但协调器仍有其他会话的 running/orphaned/leased Cargo lanes，且本机已有 6 个 Cargo
  与 3 个 rustc 进程。未终止外部进程、未删除已释放 lane、未用 2026-06-11 旧 binding
  冒充 current-source 产物，也未在共享编译竞争中启动新的 ZrVM/Runtime 全量构建。
  M0.1 继续保持 `in_progress`，验收条件仍是精确命令两次产生 FPS 数值并计算偏差 `<20%`。
- 2026-07-12 M0.1 当前进展：已从 ZrVM HEAD
  `2eb70efa143c44c9acc91e002f9f054f54e9f588` 生成当前 import library/runtime DLL，
  并把 Vampire 剩余 7 个 v6 model sidecar 硬迁移到 v7；59 个 sidecar 现均为 v7。
  诊断直跑得到 `34.67983575629786 FPS / 28.8352 ms`，但不计入正式双样本。精确 Cargo
  命令的最近一次完整测试二进制在运行 89.97s 后失败于图编译：`opaque-mesh` 对 transient
  `scene-color` 报 load-before-producer；同一二进制复跑 79.41s 后复现相同错误。后续代码流
  审计确认相机栈策略只在图编译后的 GPU 执行期生效，不能作为该诊断的已证实根因。最新
  current-source 重建已编译生产 Runtime，但连续 lib-test 重建分别在并发 Asset migration
  的 `AssetReference::guid()`、transaction recovery match、crash-window helper 可见性与
  `PathBuf` 类型错误处停止，表明该 owner 正在持续写入，尚未形成可归因的静止源码窗口，
  focused 图契约也尚未执行；因此旧二进制结果只
  归属其编译快照，不外推为当前源码。Runtime 07 未越权修改活动 Render 18/Asset owner。
  当前正式样本仍为 0；待当前源码恢复测试编译后，先重建图契约，再以不变源码快照完成
  精确命令两次并满足偏差 `<20%`。
  为绕开 lib-test 目标，还在仓库外发起了 production-only 图探针；它同样在编译 Runtime
  时撞上并发 Asset migration 的 transaction schema 可见性与新增
  `AssetImportOutcome::reference_repairs` 字段不一致，未产生图结论，也未修改仓库源码。
- 2026-07-12 M0.1 后续现源码证据：Asset migration 收敛后，仓库外 production-only
  探针成功编译并输出 forward-plus pass 序列，`opaque-mesh` 位于 `preview-sky` 之前，
  因此旧混合快照的 scene-color 首写错误不再是当前生产图阻塞。focused lib-test 也已执行，
  但其断言仍要求 `depth-prepass` 写 `gbuffer-normal`，与现描述符不一致；该项按 Render 18
  活动 owner 的陈旧测试期望处理，Runtime 07 不在他方 owner 上加 workaround。
  精确 M0.1 命令随后 1/1 通过，首个正式样本为
  `30.894424483213513 FPS / 32.368300000000005 ms`，116 mesh draws，日期 2026-07-12。
  更新后的同一测试二进制再次通过并得到
  `29.641661948702144 FPS / 33.7363 ms`，两者均值相对偏差 `4.138895%`；但该次外层精确
  Cargo 尝试期间 Shader 06/Render 18 活动会话仍修改 Runtime 源码，且精确命令自身先撞上
  Shader 06 测试模块的瞬态编译不一致，因此该直跑只作稳定性证据，不提升为第二个正式
  Cargo 样本。当前正式样本由 0 更新为 1；仍需在共享 Runtime 源码静止窗口内再完成一次
  精确 Cargo 通过后，才能关闭 M0.1。
- 2026-07-12 M0.1 最终验收：精确命令
  `cargo test -p zircon_runtime --lib vampire_project_session_reports_runtime_fps_and_render_work --features backend-zr-vm --locked -- --nocapture --test-threads=1`
  的第二个正式样本为 `33.98320549984198 FPS / 29.426299999999998 ms`，116 mesh draws，
  1/1 通过（82.76s）。与首个正式样本 `30.894424483213513 FPS / 32.368300000000005 ms`
  的均值相对偏差为 `9.521868%`，低于 `<20%` 门禁。中间一次精确命令还产生
  `39.22630044992567 FPS / 25.493100000000002 ms`，但共享 Runtime 源码在该运行窗口内
  继续变化，因此只保留为诊断，不计入正式双样本。M0.1、M0.2、M0.3 至此全部完成，
  M0 基线门禁已完整关闭。
- 2026-07-12 M1 最终状态：`EcsFramePerformanceDiagnostics::publish(...)` 已把每个完成 tick
  的 QueryState/change-detection 聚合写入 runtime diagnostic store，并由
  `headless_session_tick_publishes_ecs_frame_diagnostics` 锁定端到端可见性；Vampire 精确测试
  同步打印 query/change/extract 计数。权威清单使用 128 entities / 8 repeated runs / 8 hits /
  1 initial miss / 1 initial rebuild、6 scanned stale marks / 0 matches，以及 extract rebuilds
  `[1, 0]` / hits `[0, 1]` / misses `[1, 0]`。最后一个已成功构建的 current Runtime test
  executable 对 `frame_extract_rebuild` 2/2、`ecs_query` 58/58 通过；current-source standalone
  performance guard 28/28 通过。新的全包 focused 构建在执行前被活动 Shader 06
  Realtime IBL 与 Physics 03 ColliderShape 编译中间态阻塞，未改动外部 owner，也不把该共享
  workspace blocker 记作 Runtime 07 实现缺口。M1.1/M1.2/M1.3 与 Runtime 07 设计方案至此完成。

## 性能审阅交接

- 2026-07-18 新性能交接：core scheduler诊断每个微任务事件仍以全局in-flight/epoch执行至少4次额外共享原子RMW，frame snapshot还锁稳定缓存；同时runtime facade的临时handle Arc增减已局部清零。Runtime07需把1M no-op jobs、fan-in 1/100/10k、diagnostics on/off与1M facade calls纳入WPR/CPU/原子争用基线，联动Runtime11完成worker-local/sharded或采样计数。责任见PERF-MVP-317/319。
- 2026-07-18 state热路径交接：state init二次锁已止损，但transition history仍无界增长/整段clone，hook dispatch仍三表全扫。Runtime07需增加100k transitions、60/120 Hz、1/100/10k hooks压测，记录history bytes、clone bytes、lock wait和hook probes；预算见PERF-MVP-320。
- 2026-07-18 startup热路径交接：module ready忙轮询已止损为bounded sleep，仍须以WPR证明0/1/100/10,000 ms async ready的启动线程CPU并迁移到notification；module/service/dependency各1/100/10k还需记录activation descriptor/list clone与blocked-unload graph visits。预算见PERF-MVP-321。
- 2026-07-18 EventBus热路径交接：existing-topic subscribe的额外String clone已止损；默认诊断仍把Instant和多次共享原子放到每publish/每subscriber，topic delivery锁串行publisher，unsubscribe可在该锁下排空lossless积压。Runtime07参考Bevy typed Messages的双generation buffer/cursor分出帧内typed lane，并为动态bus提供topic token/batch、diagnostics off/sampled/sharded与lossless预算；见PERF-MVP-323。
- 2026-07-18 diagnostic store交接：render stats四类helper、5条遗漏叶metric及collect root 5条metric已全部切到static metadata快路，devtools registry排序也已移出锁；一次可见pane采集仍固定写约541条series并全量深snapshot，同render frame可重复。Runtime07需与Render17定义整体/domain generation、dense series token、packed delta与editor同generation缓存，并量化1/541/10k series及0/30/60/120 Hz render/UI；见PERF-MVP-324。
- 2026-07-18 profiling观测者效应交接：inactive capture的全局recorder锁、静态name分配、非Tracy动态payload求值与稳定stream临时key已直接止损；active capture仍把全部线程的scope/frame/counter串行到单Mutex，并在锁内深snapshot。Runtime07需交付static dense ID、thread-local bounded chunk、frame边界batch merge和generation封存，量化1/8/64线程与每帧0/100/10k事件的lock wait/alloc/drop；见PERF-MVP-326。
- 2026-07-23 profiling capture容量合同补充：interface `ProfileCaptureConfig::normalized`只处理0值，没有不可突破的entry上界、retained bytes、单metadata String或snapshot/export page预算，NaN/无限frame budget也可穿透；外部ABI可把active recorder和wide snapshot推到任意规模。Runtime07按PERF-MVP-566为entries+bytes+field/page共同设hard maxima、finite校验、requested/effective/drop/age诊断，并让interned metadata+dense rows按双预算evict；sealed generation分页/Arc消费复用324/326，不另建ABI私有ring。
- 2026-07-18 core resource交接：typed get/register/reload/sort的冗余record clone与格式化已止损；acquire与last release仍有跨payload/runtime锁竞态，每lease heap closure、同步drop多锁及subscriber锁内无界event clone会放大asset burst。Runtime07联动Runtime04量化1/8/64线程、1M acquire/release与100 subscriber/reload burst，并交付per-entry generation事务、有界drop/event lane和frame batch；见PERF-MVP-327。
- 2026-07-18 fixed-step交接：framework clock的大catch-up plan已由逐step空循环降为O(1)批量整数计算；Runtime07仍须按client/editor/headless profile量化和规定max steps、defer/drop，发布requested/executed/capped/remaining overstep，验证长stall不造成无限追帧或静默时间丢失。Runtime03已完成的schedule语义不重开；见PERF-MVP-328。
- 2026-07-18 plugin bridge诊断交接：debug/editor构建的weak/native/script bridge每次调用仍对interface entry做共享AtomicU64 RMW。Runtime07联动Runtime06提供off/sampled/sharded模式与snapshot聚合，量化1/8/64线程和1M小调用；普通debug产品路径off时RMW必须为0。见PERF-MVP-330。
- 2026-07-18 script host-call交接：production ScriptCallSite每调用仍分配module/function String、深clone全部capability Strings，String/Bytes参数又多层拥有；reflection registration field投影接近O(F²)。Runtime07联动Runtime13量化1M calls并硬切interned ID+borrowed/arena context、shared capability bitset和generation compiled ABI；见PERF-MVP-331。
- 2026-07-18 picking帧热路径交接：pointer分组重复扫描/排序、owned hit与previous/release state clone已局部止损；pipeline仍每调用新建全部容器，hover/report各自projection，output与event state间整图clone，drag与primitive backend可分别形成dragged×hovered和rays×all-primitives放大。Runtime07联动Runtime12交付可复用双buffer `PickingPipelineWorkspace`与single resolved frame，量化1/8/64 pointers/cameras、1/100/10k hits、1/1k/100k primitives及debug on/off；空间粗筛由Editor05/Render04共享visible query承接。见PERF-MVP-332。
- 2026-07-18 camera/gizmo交接：dynamic camera每action的owned SceneNode projection、idle quaternion/exp、gizmo per-endpoint matrix与circle temp Vec已由scalar local-transform read、早退、Mat4复用和streaming止损。Runtime07联动Editor05/Render17交付generation-compiled、clear/swap复用的gizmo frame storage与唯一overlay owner，并量化1M camera actions、1/100/10k commands/circles及1/1k/100k retained instances；framework gizmos当前无生产caller，不能把dead-code数字计作产品收益。见PERF-MVP-333。
- 2026-07-18 input/action交接：context lookup、axis double traversal、release key clone与descriptor config clone已直接止损；全部device/frame payload仍由一个manager Mutex更新并由owned frame snapshot深clone，action manager又以Mutex串行只读evaluation并每次重建axis/context/String输出。Runtime07联动Runtime12量化125/500/1k/10k Hz、1/8/64 devices/threads、1/10/100/1k/10k actions/bindings，交付domain view、generation compiled IDs、reused scratch与frame/device shards；见PERF-MVP-334，recording/coalescing下层见PERF-MVP-003。
- 2026-07-18 physics同步交接：fixed tick当前以owned snapshot复制/排序全scene nodes、逐node求world transform并递归复制shape/material/joint payload；builtin零substep仍同步，manager/backend/query还重复clone、全表扫描与排序。Runtime07联动Physics03交付persistent per-world state、dirty generation delta、稳定native handles、一次transform projection、broad-phase query、reused output和有界clear/swap events；量化1/1k/100k nodes/objects、0/1/10/100% changed及1/1k/100k queries，stable generation全量projection/snapshot clone与zero-step sync必须为0。见PERF-MVP-335。
- 2026-07-18 editor render-extract交接：`build_viewport_render_packet(&self)`为了运行`RenderExtract`内部系统先深clone整份World，随后才遍历构包；多primitive transform hash已从每primitive降到每entity一次。Runtime07联动Editor05把派生更新放入明确schedule，以只读live-world extract或generation-owned render-world artifact硬切clone-World权威；stable generation full extract=0、changed generation stage=1、World clone bytes=0。参考Bevy `Extract<P>`/changed resource抽取，见PERF-MVP-349及render root契约静态证据。
- 2026-07-18 post Volume交接：内建evaluator/registry per-call重建与已排序输入重复sort已直接止损；scene仍每帧扫描Volume、查询hierarchy/transform/layer并把profile展开为String/参数Vec，同camera又被post/froxel/history重复求值。Runtime07联动Render07/17提供scene/profile/transform/layer Volume generation与immutable compiled set，稳定generation extract/override/clone=0、每camera submission resolved≤1；见PERF-MVP-363/364及Volume静态证据。
- 2026-07-18 graphics模块启动交接：Lazy RenderFramework首次服务解析仍在caller线程同步request adapter/device并构造renderer，8组extension catalog在module descriptor/factory/framework间多轮深clone。Runtime07联动Plugins01以generation-owned catalog Arc和Initializing→Ready/Error ticket接入module readiness，render/device-init lane single-flight推进；主/UI线程blocked=0、catalog每generation物化≤1。见PERF-MVP-409。
- 2026-07-18 viewport camera state交接：同一camera当前分散于7张HashMap，动态viewport/layer/type又作为identity且无统一prune。Runtime07建立dense `ViewportCameraStateTable`，每帧按Render09 slot generation mark/reconcile，removed slot显式释放renderer history/provider/debug/particle state，短暂retire只允许有界TTL；entries≤active+budget、每camera lookup近1。见PERF-MVP-410。
- 2026-07-18 render framework锁域交接：全局operation/state mutex当前跨surface driver/pipeline创建、history release及capture/stats/VG大payload clone。Runtime07建立render-owner ordered command lane和viewport generation ticket：短锁reserve/publish，driver/GPU/大clone锁外执行，read-mostly snapshot用Arc generation；独立viewport慢操作不得串行阻塞query/submit。见PERF-MVP-411。
- 2026-07-18 pipeline control-plane交接：reload/set/profile的validation graph compile已移出state mutex，但仍持全局operation guard并每次clone/compile。Runtime07提供handle+revision+executor/capability generation ticket与single-flight状态，短锁CAS发布last-good；set只消费validated artifact，same revision并发compile≤1。见PERF-MVP-412。
- 2026-07-18 submission record/history交接：provider registration String、compiled pipeline深clone、stable graph dump重复序列化及VG重复统计/traversal已直接止损；`record_history`仍每camera复制bindings、visibility snapshot、static index/validation，capture查询仍复制完整RGBA/String。Runtime07把这些状态并入PERF-MVP-410/411的camera slot和render-owner lane，发布generation-owned submission/history/capture handles并短锁CAS，stable history clone=0且锁持有不随payload bytes增长；见PERF-MVP-413。
- 2026-07-18 frame context交接：camera descriptor与owned particle/motion/plan/report/capability二次clone、VG provider/output clone及重复material root resolve已局部止损；context仍在state锁内clone history/pipeline并调用provider，且每camera重复material/environment/model构建。Runtime07在camera slot/render-owner lane发布generation-owned `CompiledViewSubmissionTemplate`，scene/material/environment共享artifact跨camera，锁外build后短锁CAS；稳定generation I/O/build/clone=0。见PERF-MVP-414并复用410/411/413。
- 2026-07-18 runtime feedback/history交接：history compatibility重复比较与particle result Vec容量已止损；每camera仍take/merge/clone provider feedback，非owner还构造后丢弃particle feedback。Runtime07在camera slot标记唯一shared-product owner并让render-owner lane发布generation-tagged feedback/history tickets；非owner不消费viewport-global feedback，短锁只交换sealed handles，队列age/drop有界。见PERF-MVP-415并复用410/411/413。
- 2026-07-18 VG debug snapshot交接：page inspection二层线性查找已改为单次索引，但正常VG帧仍每camera无条件CPU重建/深clone完整诊断。Runtime07把debug subscription generation纳入camera slot/shared-product owner，off时不build/publish，on时只交换Render03 sealed report Arc；snapshot history/rows有界且poll if-newer。见PERF-MVP-416。
- 2026-07-18 camera loop plan交接：terminal target与planar target查重已局部止损；present/submit仍双resolve sequence，多camera clone post/visibility并逐camera重建context。Runtime07按Render09 generation plan短锁取Arc并驱动ordered camera tickets，large source只传handle；planar update只在成功camera原子提交进度。见PERF-MVP-417并复用410/414..416。
- 2026-08-11 camera target sharing 守卫复核：生产 `viewport_terminal_camera_target` 已从 shared extract 的 camera slice 借用解析 sequence 并只 clone 最终 target；旧 `.map(|submission| ...)` 锚点属于守卫漂移。source-bound standalone guard 已 RED→GREEN，并同步收紧当前 async-capture pipeline 方法名；受管 Cargo 仍被外部未登记 E 盘 artifact gate 阻塞，failure 保持 open，见 `07/failure-2026-07-23-submit-context-camera-target-sharing-anchor-drift.md`。
- 2026-07-18 submit execution补充交接：VG snapshot在frame/global/per-camera间已改Arc共享，公开query的owned复制也已移到state锁外；但operation锁仍跨完整camera loop，state锁跨prepare、GPU render/present、feedback、record/stats。Runtime07与Render10交付`PreparedSubmissionTransaction`三阶段owner：短锁snapshot/reserve、render-owner lane锁外执行、短锁generation CAS publish；same viewport有序，独立viewport/query不得被慢driver或feedback阻塞。见PERF-MVP-411并复用416/417。
- 2026-07-18 render stats observer交接：coverage/executor/visibility/UI/String/VG index局部止损已完成，但terminal camera仍在state锁内无订阅地重建/复制完整RenderStats。Runtime07在Phase C只发布`Arc<SealedRenderFrameDiagnostics>`与少量health counters，detail按subscription/capture lazy materialize，query-if-newer短锁clone Arc、history有界；diagnostics-off detail work=0。见PERF-MVP-418并复用411/413/416。
- 2026-07-18 submission root contract补充：mutable generation helper已由内部2次viewport lookup降为1次，但owner camera仍先公共validate再mutable helper，合计2次。Runtime07的Phase A预留stable viewport slot/generation ticket，Phase C以slot CAS publish，destroy/recreate使generation失效；不得用裸handle重复查表或跳过capture failure语义。见PERF-MVP-411并联动414/417。
- 2026-07-18 graphics debugger owner补充：capture state已严格有界为pending+latest queued，stop前也会释放framework state锁；但外层operation锁仍跨backend stop/poll，start/prepare stop仍在全局state锁。Runtime07把capture ticket并入render-owner ordered lane，完成按generation发布，stop/readback异步有界且不阻塞独立viewport/control/query。见PERF-MVP-023/411。
- 2026-07-18 temporal history ownership补充：wide validation key在context/history间已改Arc共享，record deep clone=0；首次key仍逐camera clone lighting/post/particles并扫描meshes/poses，record还clone bindings/visibility/static。Runtime07 camera slot发布compact revision token与共享`HistorySnapshotHandle`，stable generation build/compare/payload clone=0，Phase C只提交Arc。见PERF-MVP-413/414。
- 2026-07-18 runtime provider合同补充：HGI/VG filtered readback投影已按输入容量预留且保留溢出跳过；但HGI/VG `prepare_frame`仍在framework state可变借用期间执行动态provider runtime，Solari availability也在state锁内调用。Runtime07把这些调用纳入PERF-MVP-411 render-owner Phase B和PERF-MVP-379 generation prepare lane，Phase A/C只交换provider slot与sealed handles；provider callback/state-lock overlap=0。automatic VG的per-camera mesh/model工作继续归414，readback owner归415。
- 2026-07-18 visibility scene owner交接：mesh frustum matrix/tan、extra-view candidate与四类中间clone/growth已止损；Context仍每frame多表重建，previous static index深clone后才incremental，多view又重复全primitive结果。Runtime07按PERF-MVP-419/420发布generation-owned primitive SoA/index Arc和dirty delta lane，camera/view只借用prepared descriptors与dense bitsets；stable context/index deep clone=0、provider/render state锁不跨cull。
- 2026-07-18 pipeline compiler ticket补充：viewport默认pipeline handle现直接取集中Copy常量，不再每camera/frame构造完整builtin asset；Runtime07继续把PERF-MVP-365/412/422的handle+revision+options+executor/plugin generation编译收敛为锁外single-flight ticket，state锁只交换last-good Arc，stable descriptor/resource analysis与asset clone=0。
- 2026-07-18 graphics types owner补充：output-target graph-import/writeback计划的测试诊断已从生产构建移除，正常状态判断heap alloc=0。Runtime07在PERF-MVP-413/414中收敛`ViewportRenderFrame`的scene/extract双owner与RGBA capture handle，在417中让多camera仅借用post/visibility generation handles；稳定source clone=0且camera间不复制大payload。
- 2026-07-22 World派生状态交接：despawn full archetype rebuild与validity per-entity HashSet allocation已止损；hierarchy dirty仍做parent O(N×depth)验证、active/transform各建children表并递归全树、NodeCache深clone宽SceneNode。Runtime07联动Runtime08/Editor05交付唯一generation topology、dirty-root frontier与迭代subtree delta，stable build/scan/clone=0，100k深链无栈溢出；见PERF-MVP-459与`07/failure-2026-07-22-world-derived-state-full-rebuild.md`。
- 2026-07-22 scene render projection补充：World render五文件已静态读完；particle候选已从全World扫描止损到dynamic owner并删除两个重复sort，但每camera仍重新排序owners、解析JSON并重建sprite/bounds/gpu-frame。Runtime07联动Render12发布typed/revision particle artifact并跨camera共享；mesh/light/camera/Volume/visibility稳定帧统一复用349/363/364/419的scene generation owner，见PERF-MVP-465与render projection静态证据。
- 2026-07-22 LevelSystem frame-state交接：单`WorldRuntimeState` Mutex当前串行physics/animation/script，render先锁内深clone全部pose再持World锁extract，playback/events也owned clone，script started lookup分配String。Runtime07发布按域revision的sealed frame handles并让render短锁取一次后锁外消费，具体domain复用335/439/442；见PERF-MVP-469与`07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md`。

## 2026-08-27 Shader Prewarm Test Owner Split

状态：`runtime_07_15_shader_prewarm_test_owner_and_source_guard_routing_static_passed_cargo_deferred`。

本切片先完成不依赖受管 Cargo 队列的基础设施收敛：`graphics/shader/variant_cache/prewarm.rs`
从 811 行降到 142 行，只保留四个预热入口与 folder-backed test route；既有执行循环继续由
251 行的 `prewarm/worker.rs` 持有，11 个主体测试原样迁移到 667 行的 `prewarm/tests.rs`，
2 个 module+pipeline 组合验证测试继续由 159 行的
`prewarm/tests/combined_validation_tests.rs` 持有。Runtime15 的 source guards 现分别读取
route、worker 与 test owner，不再依赖把生产锚和测试锚混在旧父文件中的假阳性。

Python 结构回归 1/1、定向 `rustfmt --check`、11+2 测试属性计数及拆分前后规范化源码
等价检查通过。缓存键的 source hash、include hashes、template revision、Naga 与 WGPU
版本身份参数保持当前源码语义。此项未修改预热调度、校验、磁盘写入或缓存算法，也未生成
profile、功耗或二次启动 miss 数据；因此不关闭任何 shader-prewarm 性能瓶颈，受管 Cargo
仍延后，本状态不构成 Runtime07/Render08 产品验收。
