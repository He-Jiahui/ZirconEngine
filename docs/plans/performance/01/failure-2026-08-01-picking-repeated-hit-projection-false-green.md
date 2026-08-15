---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: picking-repeated-hit-projection-false-green
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/performance/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - zircon_runtime/src/core/framework/picking/hover_map.rs
  - zircon_runtime/src/core/framework/picking/pipeline.rs
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
  - zircon_runtime/src/core/framework/picking/report.rs
  - zircon_runtime/src/tests/picking/hits_and_hover.rs
  - zircon_runtime/src/tests/picking/pointer_events.rs
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/build_dispatcher.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --locked --lib picking_output_resolution_shares_one_projection_between_hover_and_report
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --locked --lib picking_pipeline_shares_one_sorted_hit_projection_between_hover_and_report
  - cargo +1.94.1 test -p zircon_editor --locked --lib overlay_router_debug_feed_reports_runtime_picking_route_at_point
---

# Performance01：Picking 重复 hit projection 的 false-green

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-08-01 计划与 current source 对齐复核
- 修复责任计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 交接原因：现有计划与源码门禁在同一 Performance01 执行切片内产生 false-green，需要保留本地 failure 生命周期直到 focused managed GREEN。
- 发现方式：独立 Sol/High 子代理逐项核对 `PERF-MVP-332`、生产调用链与现有源码门禁。

## 失败现象与复现证据

旧测试 `picking_hit_resolution_avoids_repeated_projection_work` 只检查 hover/report 文件中不存在若干历史字符串。它没有检查 `run_picking_pipeline` 的真实组合路径，因此即使 `PickingHoverMap::from_outputs` 与 `PickingPipelineReport::from_ray_map_and_outputs` 各自重新执行 `sorted_hits_by_pointer`，测试仍会通过。

本轮先把门禁改为 `picking_pipeline_shares_one_sorted_hit_projection_between_hover_and_report`。生产修复前的静态 RED 为：pipeline 中 `sorted_hits_by_pointer(&backend_outputs)` 调用数 `0`，hover 共享入口 `false`，report 共享入口 `false`；这证明旧测试的 GREEN 不能支持计划中“hover/report 共享同一 projection”的结论。

## 最低共享层根因

`pointer_hits` 已提供按 pointer 分组排序的纯算法，但 authority 最初留在两个结果构造器内部。pipeline 的首轮修复只让 framework runner 持有一次 frame-scoped sorted projection；2026-08-02 复核又确认该 runner 没有非测试调用者，而 editor 产品 dispatcher 仍通过 `hovered_hits_for_pointer` 与 `PickingPipelineReport::from_outputs` 独立排序。旧源码门禁观察 leaf 文件中的禁用字符串，没有观察 framework authority 是否真正接入产品调用链。

## 架构修复验收

- 保留 `PickingHoverMap::from_outputs` 与 `PickingPipelineReport::from_ray_map_and_outputs` 的公开合同。
- pipeline 对 `backend_outputs` 只构造一次 sorted projection；report 借用后，hover 消费同一 map 并 move 被选中的 hit。每个 pointer group 至多排序一次。
- editor viewport 的 route+debug 产品入口必须复用同一个 resolved output，不得分别调用 hover/report 独立构造器；route-only 独立入口仍允许只解析一次 route。
- pipeline output 与 next-frame event state 在普通帧共享同一 hover backing storage；取消/清理 mutation 不得回写 output snapshot。
- priority、block-lower、ray/output 计数与公开独立构造行为不变。
- test-only thread-local metrics 通过真实双 pointer pipeline 证明 projection builder=1、pointer-group sort=2，并同时验证 hover/report parity、storage identity 与 COW mutation isolation；focused current-source Windows managed test 恰好执行 1 个并通过。在此之前本记录保持 `open`。

## 禁止临时方案

- 不通过弱化/删除门禁、仅匹配历史禁用字符串或关闭 report/hover 来伪造改善。
- 不增加兼容双路径、全局缓存、跨帧 stale projection 或公开 workspace API。
- 不把静态源码检查、编译入口越过或其他 Picking 测试结果冒充 focused GREEN。

## 修复结果与回传

Open state：framework 与当前 editor 产品 route+debug 路径的源码修复已落地。公共 `resolve_picking_outputs` 生成一次 `sorted_hits`，report 先借用、hover 随后消费；`run_picking_pipeline` 与 editor `resolve_runtime_route_and_debug_feed` 均复用该组合入口。`hovered_hits_from_sorted` move 被选中的 hit；`PickingHoverMap` 由 `Arc<BTreeMap<...>>` 持有命中表，pipeline output 与 event state 的普通帧 clone 共享 backing storage，公开 mutator 通过 `Arc::make_mut` 保持取消/清理隔离。公开独立 hover/report 构造器继续服务只需要单一结果的调用者。

初版字符串门禁的静态 RED 为 pipeline 调用数 `0`、hover/report reuse `false/false`；生产修复后虽为 `1/true/true`，Sol/High 终审指出 helper 内重排仍可能绕过该门禁。第二轮 TDD 已把它替换成真实双 pointer pipeline test；RED 时测试引用的 reset/read metrics 尚无生产定义，随后才加入仅在 `cfg(test)` 下存在的 thread-local counter。固定 Rust 1.94.1 `rustfmt --check`、scoped `git diff --check`、metrics definition/usage 与 selected-hit clone=0 静态检查通过。精确运行时 GREEN 仍受下述共享编译错误阻断。

Windows managed job `2a94277bd1964f7abf075f4219102440` 使用共享兼容池执行本记录 frontmatter 中的 locked focused command。validator wrapper 已释放，终态 `exit_code=1`；内部 Cargo test 为 `exit 101`，`zircon_runtime` lib-test binary 因 54 个 current-source 外部编译错误停止，产生 394 warnings，目标测试 0 次执行。可见最低 blocker 包括 `graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs` 的 partial move E0382 与 `graphics/runtime/render_framework/budget/degrade_ladder.rs` 在 const fn 中调用 `Ord::max` 的 E0658；本轮 Picking 文件不是所示 error location。该结果不是 Picking RED/GREEN，也不能把本记录改为 `fixed`；需等待外部 compile owners 收敛后重跑 focused command。

Sol/High 终审修复后的最终源码由 managed job `367cc00c08394ae4b4705d2fe3c6f44f` 再次执行同一 locked focused command。job 于 2026-08-01 09:07 +08:00 释放，validator `exit_code=1`，内部 Cargo `exit 101`；共享 lib-test 图仍有 53 个外部编译错误与 397 warnings，目标测试 0 次执行。错误示例仍属于 RHI UI geometry、IBL bake partial move 与 Render budget const `Ord::max`，本轮 Picking metrics/move 路径未成为所示 error location。该 current-source 证据确认编译入口已消费最终补丁，但没有产生运行时 metrics GREEN；failure 继续保持 `open`。

第二路 Sol/High 审阅确认 Picking 生产模块与该 lib test 都不受 feature 门控，`core-min = []` 也没有源码 cfg 消费者；因此最小有效验证应使用空 feature 集，而不是默认 `target-client` 图。Windows managed job `c630da233fc440559b1788eafebddf9a` 于 2026-08-01 09:45 +08:00 释放，实际执行 frontmatter 中的 `--no-default-features --lib` focused command；validator `exit_code=1`，内部 Cargo `exit 101`。收窄后共享 lib-test 编译错误从 53 个降为 16 个、warnings 从 397 降为 324，目标测试仍为 0 次执行。可见 blocker 包括 Runtime11 `set_before_execute_hook` 私有可见性、Runtime04 asset registry / dynamic scene reload 测试合同漂移、Plugins01 跨线程返回非 `Send` 的 ABI status，以及 Text04 page-shadow re-export 可见性；这些路径均已有 active/resolving owner 或 dirty overlap。该结果进一步证明默认 graphics/text/UI 图不是 Picking 验收所必需，但仍未获得 focused 1/1 GREEN，本记录继续保持 `open`。

2026-08-02 的计划/源码复核识别出另一处不一致：主计划把 previous-hover ownership transfer 写成已完成，但 `pipeline.rs` 仍对完整 `PickingHoverMap` 调用 `clone()`。按 milestone TDD，真实 pipeline test 先引用尚不存在的 `shares_storage_with` 形成静态 RED，再以 `Arc<BTreeMap<...>>` + `Arc::make_mut` 完成最小 COW 修复；同一测试还在 `clear_pointer` 后证明 output snapshot 未被 mutation 污染。固定 Rust 1.94.1 rustfmt、scoped `git diff --check` 与 Arc identity/mutation 静态检查通过。当前外部 Cargo reservation `431063bb4a79447488688bcf7f90dea1` 仍为 `leased`，因此没有提交必然排队或越权的 managed job；本记录继续保持 `open`，focused 1/1 GREEN 要求不变。

同日独立 Runtime/Render Sol/High 复核确认测试从 `lib.rs` 经 `tests/mod.rs`、`tests/picking/mod.rs` 真实挂载且不受 feature gate；同时把空 feature 编译图的下一最低 blocker 收敛到 Runtime04 当前 dirty owner：`scene/module/level_manager_project_io.rs` 无条件调用 `crate::diagnostic_log::write_log`，但 `lib.rs` 只在 `diagnostic-log` feature 下公开该模块，而 `scene` 始终编译。旧 no-default blocker 中的 duplicate imports、私有 accessor、registry/ABI 漂移和 locked `meshopt` 已在 current source 静态消失。该 E0433 风险已通知 `runtime04-artifact-chunked-generation-r2-20260731`，Performance01 未覆盖其并发 scene-artifact 补丁；在 Runtime04 修复最低 feature boundary、外部 reservation 释放并取得 focused 1/1 前，本记录继续保持 `open`。

2026-08-02 后续协调确认 Runtime04 Session 已长时间无心跳，且 `scene/module/level_manager_project_io.rs` 不在其声明写入范围、没有精确租约。Performance01 因此在保留并发 scene-artifact 实现的前提下直接修复最低 feature boundary：先向 Frameworks03 feature 预设测试加入回归断言并观察预期 RED，再仅给 `scene_artifact_failure` 的诊断副作用增加 `#[cfg(feature = "diagnostic-log")]`；稳定的 `scene_artifact_persistence_failed` 映射不变。focused 断言与完整 Frameworks03 套件 `8/8` 通过，固定 Rust 1.94.1 `rustfmt --check` 通过，修复证据已追加回 Runtime04 Session。外部 Cargo reservation `431063bb4a79447488688bcf7f90dea1` 仍为 foreign-owned，因此未提交会排队的 managed job，也没有 focused 运行时 `1/1` 证据；本 failure 继续保持 `open`。

同日第三轮 Sol/High 产品调用链复核发现 `run_picking_pipeline` 只有测试调用者，editor live dispatcher 的 route+debug 仍重复投影。新的行为 RED 先引用不存在的 `resolve_picking_outputs`，随后公共组合 resolver、framework pipeline 与 editor adapter 完成单一 authority 接线；真实多 output、双 pointer、non-hoverable/blocking 测试要求 builder=1、pointer sort=2，并验证 hover/report parity。同步删除只排除历史源码字符串的 pointer-event receipt；Runtime15 Picking folder guard 删除硬编码 `20` 测试总数，保留目录 owner 与 800 行预算。foreign Cargo reservation 仍阻止 managed GREEN，因此 failure 继续 `open`。

本轮最终静态验证：固定 edition 的 `rustfmt --check` 通过，scoped `git diff --check` 通过，runtime pipeline 与 editor route+debug 产品调用链均只接入组合 resolver，两个已删除设置字段和两类过时 receipt 断言均无残留；handoff validator 为 556/0，plan-output audit 通过。未运行 Cargo，原因仍是 foreign reservation `431063bb4a79447488688bcf7f90dea1`，所以这些证据不能替代 focused `1/1` GREEN。

## 2026-08-14 全模块结构性复审

本轮按当前工作树重新逐文件阅读 `zircon_runtime/src/core/framework/picking/**` **23/23**（2,069 行）与 `zircon_runtime/src/tests/picking/**` **6/6**（889 行），并回查 editor 的 `runtime_picking_adapter.rs`、`overlay_router/build_dispatcher.rs` 产品组合入口。生产目录静态计数为 23 次显式 `.clone()`、13 次 `.collect()`、1 个排序调用；排序 authority 只在 `pointer_hits.rs`，组合 resolver 对一批 outputs 只建一次 per-pointer map，report 借用、hover 消费，当前修复方向成立。

结构性参考依据不是相似命名，而是同一 picking/selection 数据流：

- Bevy `dev/bevy/crates/bevy_picking/src/backend.rs:24-30` 明确允许 backend 用 spatial hierarchy，并允许 blocking hit 终止更低层；`hover.rs:105,134-140,181` 以 `Local<OverMap>` 跨帧保留容器，swap previous/current 后 clear 复用，再按层排序一次。Zircon 的单批 projection authority 与共享 hover storage与此方向一致，下一步应验证稳态容器复用，而不是再加一套缓存。
- Unreal `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UnrealClient.cpp:1901-2019` 用 `bHitProxiesCached`/显式 invalidation 控制 hit-proxy 重绘，只在需要 fetch 时执行 `ReadSurfaceData` 与 `FlushRenderingCommands`；`LevelEditorViewport.cpp:2882-2900` 的特殊 gizmo-underlying click 才失效并重取。Zircon 的 renderer/scene 级 picking 不能退化为每帧同步 GPU readback，也不能让 debug 查询触发第二次 projection。
- Fyrox `dev/Fyrox/editor/src/interaction/select_mode.rs:50,112-134` 保留并 `clear()` 复用 traversal stack；框选图遍历发生在选择交互边界，而不是为每次 move 新建递归容器。这支持把昂贵选择工作限制在 invalidation/interaction boundary，并复用 scratch。

当前仍有两个未量化的架构风险，因此本轮不直接改生产代码：

1. `PrimitivePickingBackend::collect_hits` 对每条 ray 扫描全部 primitive，复杂度为 `O(R * N)`。它只能作为小规模 overlay/test backend；大场景 renderable picking 必须由 Render04/Editor05 提供 BVH、visible query、physics query 或 renderer-owned PickId/hit proxy，backend 输入规模应接近 broad-phase candidates，而不是 scene primitive 总数。
2. `PickingEventState::active_buttons` 对复合 button map 做全表扫描并构造临时 `Vec`，drag/cancel 状态还有多处 owned clone。先在真实 pointer/button storm 中记录 visits、temporary bytes、COW fallback 与 event count；没有产品占比证据前，不引入第二份易漂移索引。

后续规模门：`rays={1,10,100}`、`primitives={10,1k,100k}`、`pointers={1,10,1k}`、`buttons={1,10,1k}`。每批 projection builder 必须为 1、每 active pointer group sort 不超过 1、selected-hit clone 为 0；scene backend candidate visits 必须随 broad-phase 结果而不是总 primitive 线性增长；普通帧 output/event-state 共享 storage，COW 只在 mutation；再记录 allocation bytes、comparisons、main-thread p50/p95/p99 与 worker overlap。

2026-08-14 fresh managed `-DryRun` 尚未生成 Cargo 命令：协调器以 `unmanaged_artifacts_detected` 拒绝，报告的是 D/E/F 上其他 Session 的 target/fixture，目标测试执行数仍为 0。固定 Rust 1.94.1 的相关文件 `rustfmt --check` 与 scoped `git diff --check` 已通过，但本 failure 保持 `open`，Picking 的 29 个文件继续留在 `pending.md`。
