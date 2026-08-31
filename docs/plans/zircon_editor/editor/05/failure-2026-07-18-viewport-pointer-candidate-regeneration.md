---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: viewport-pointer-candidate-regeneration
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_sync.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/renderable_candidates.rs
  - zircon_editor/src/scene/viewport/pointer/candidates/projected_ring_segments.rs
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
---

# Viewport pointer candidate regeneration

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/scene`当前源126/126 Rust文件
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：scene selection、gizmo、camera与picking generation属于Editor05，不应由pointer move临时重建第二份场景投影。

## 失败现象与复现证据

原实现每个稳定`PointerMoved`先全扫scene renderables、重建scene gizmos/handles与48段ring projection，再用完整layout相等发现没有变化；dispatch随后为debug feed再次hit-test/评分。性能计划已直接加入world/selection/settings/camera/viewport generation键、lazy handle closure、route/debug单pass和ring临时Vec删除，使稳定hover零候选重建、每事件单次评分。

## 最低共享层根因

generation变化时仍无共享camera projection context、runtime可见候选/空间索引与跨render/pointer复用的gizmo extract。`projected_point`为每个点重复构造projection×view矩阵；pointer与render snapshot分别调用`build_scene_gizmos`；changed scene仍按total nodes线性扫描。

## 架构修复验收

- world/camera/settings generation change最多构建一次camera view-projection context；per-point matrix build=0。
- render与pointer消费同generation的共享gizmo/candidate extract，不各自全场景扫描。
- 候选来自runtime visible set、BVH或等价空间索引；1/1k/10k nodes的精确测试访问由query hits主导，不随total nodes无条件线性增长。
- 1k stable moves保持candidate/handle/gizmo/renderable/projection/surface rebuild=0；changed move记录scan、matrix、trig、alloc和CPU p95。
- handle/gizmo/renderable priority、projected depth、selection、hover/press/release/scroll、camera/resize、debug feed和像素/命中等价。

## 禁止临时方案

- 不得使用无法由world mutation失效的永久cache。
- 不得通过减少可拾取节点、降低ring精度或关闭debug feed伪造性能改善。
- 不得让render与pointer各维护一份无generation合同的gizmo事实源。

## 修复结果与回传

Open state: `shared projection context与render/pointer同代际single interaction extract已落地；renderable已硬切到runtime camera/layer/active-state-filtered RenderMeshSnapshot并删除editor Scene::nodes()扫描。仍待空间/BVH或pick-id broad phase、1/1k/10k query-hit访问计数、changed-move p95与受管Cargo/独立复审，故failure不回传fixed`。

runtime最低共享层缺口已正式移交 Render04：[`../../../zircon_runtime/render/04/failure-2026-07-18-viewport-picking-visible-spatial-query.md`](../../../zircon_runtime/render/04/failure-2026-07-18-viewport-picking-visible-spatial-query.md)。Editor05 不会复制 `VisibilityStaticIndex` 或从 graphics 私有字段旁路取数；待 Render04 返回同 generation 的 renderer-neutral query snapshot 后再接 cursor broad phase。

2026-07-22 current-source增量：当前树为128/128静态覆盖。本轮用RED→GREEN源码合同让gizmo scan先按`NodeKind`过滤，仅Camera/DirectionalLight调用`active_in_hierarchy`，builder复用循环中的`&SceneNode`而不再`find_node`；Editor05 interaction-extract合同5/5通过。该止损只删除非gizmo active查询和重复lookup；total-node线性scan、render packet meshes→cache Arc整slice复制、空间query、Cargo、规模counter与F4/RenderDoc仍open。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-08-23 | managed viewport Cargo verification interruption | `validation_inconclusive / coordinator_orphaned / implementation_continues` | 受管任务 `3063b21d3eb445b1961b3a508c0446a1` 运行 `cargo +1.94.1 test -p zircon_editor --lib viewport --locked --jobs 1 -- --test-threads=1`；协调器运行记录 `541031daffc541169f7c9feb1be5b7fe` 的根进程返回 0 后因 `cargo_process_tree_alive` 停在 `finish_blocked`，日志仅显示首次依赖下载与 `num-traits` 编译。随后 job 被协调器标为 `orphaned`，进程树已退出、target 已删除，job 未保留可采信 exit code。因此本次不作为 Rust test 通过或失败证据，也不产生 CPU/GPU/功耗数据；继续完成可独立的 Editor05 架构和工具契约任务，待协调器恢复可重试的 Windows managed lane 后重新执行。 |
| 2026-08-23 | pointer generation/reuse profiling contract and capture matrix | `implementation_complete / contract_validation_complete / managed_product_capture_pending` | `RendererVisibleSpatialPickSource::new/with_snapshot` 只在采用新 generation 时构造 projection；唯一的 overlay-router refresh owner 每次都记录 `visible_spatial_projection_context_build_count` 与 `visible_spatial_source_reuse_count` 的 0/1 值，分别表示本次是否构造、是否复用完全相同的 `(snapshot identity, camera snapshot, normalized viewport)`。event-time `candidates_at` 不构造 projection。现有 `viewport_pointer_metrics.json` 导出与严格门禁升级为六项 counter；E 盘 timeline 合成夹具证明完整六项放行、移除 reuse 项即拒绝、两个新 counter 的合法 0 值也被保留并放行。此为采集契约验证，不是性能基线。受管 Windows 产品采样须在 1/1k/10k selectable、static/dynamic、hit/miss、stable/replaced generation、debug on/off 的矩阵中各取 31 样本，记录 query p50/p95、六项 counter、allocation、CPU frame、GPU frame 与功耗；只有稳定 generation 的 build counter 为 0 且 reuse 有增长，并由规模数据排除 total-node 线性访问后，才评估下一步 Runtime04 索引结构调整。 |
| 2026-08-23 | projection generation comparison regression | `implementation_complete / static_validation_complete / managed_test_pending` | 新增真实单测覆盖 `ViewportProjectionContext` 的 camera snapshot 与 normalized viewport 比较：相同 0/1 高度归一化复用，camera 或 extent 改变则拒绝复用。`rustfmt --check`、scoped `git diff --check` 与 source presence contract 通过；当前外部 Cargo blocker 尚在，尚未将该 Rust test 记为执行通过。 |
| 2026-08-23 | pointer projection 与 stable-generation source 复核 | `implementation_complete / static_validation_complete / managed_profile_pending` | `ViewportProjectionContext` 已硬切为拥有 camera snapshot 的无分配 generation value；renderer-visible source 只在 source 创建或 generation replacement 时构造投影，event-time `candidates_at` 的构造调用数从 `1` 降至 `0`。复核同一 identity 的重复 renderer submission 后，source 现以 `(snapshot identity, camera, normalized viewport)` 复用既有 context，且仅 `source_changed` 才清空 route/debug cache；不新增 BVH、全场景扫描、query 或空间事实源。两条 RED→GREEN production-only 源码契约、`rustfmt --check`、scoped `git diff --check` 通过；profile scale fixture Pester 为 9/9，output-contract 套件在工具 60 秒上限前未产生终态，不记为通过或失败。仍需 Windows managed profile 采集 1/1k/10k、hit/miss、stable/changed camera、debug on/off 各 31 样本的 p50/p95、allocation、frame/GPU 与功耗；当前外部 Cargo blocker 存在，故不标 fixed 或性能验收。 |
| 2026-08-19 | Renderer-visible pointer query architecture re-review | `review_complete / instrumentation_complete / product-metrics-pending` | Runtime04 已通过 `RenderVisibleSpatialQuerySnapshot` 发布同 world/viewport/frame generation 的 renderer-neutral ray query；Editor05 的 `RendererVisibleSpatialPickSource` 只将 query 返回 owner 映射到 immutable owner table，再交给既有 handle/gizmo/renderable priority resolver。该 source 现用一个 profiling-only scope 从同一次 `query_ray` 返回值记录 `visited_node_count`、`candidate_count`、`hit_count` 与 owner mapping/projected candidate 数；非 profiling 构建保持零额外状态、无新 cache、无第二次查询。Unreal `FEditorViewportClient` 通过 viewport `GetHitProxy(X,Y)` 消费渲染命中，而 Fyrox 在 camera pick 中先做 hierarchy/AABB 粗筛再精测；两者都不支持在 editor 输入层复制另一份全场景空间事实源。 |
| 2026-08-19 | PERF-MVP pointer measurement harness | `implementation_complete / contract-validated / managed-product-capture-pending` | `tools/ui-profile-scale-fixture.ps1` 现生成 renderer-visible 的 `viewport_pointer_scene`（camera、sun、共享 mesh/material、1/1k/10k selectable、static/dynamic），`tools/ui-profile-capture.ps1` 的 `viewport_pointer` 场景只向 scene viewport 中心/角落投递输入，并从实际 `editor/viewport.pointer/visible_spatial_query` span 导出 p50/p95 及 visited/candidate/hit/projected-candidate 四类 counter。严格模式在缺 span 或任一 counter 时失败，不会把无查询运行写成数据；fixture 与 output contract 的 Pester 分别为 9/9 和 36/36。待受管 profiling 二进制可用后，用该基础运行 hit/miss、stable/changed camera、debug on/off 的 1/1k/10k 矩阵并报告每组 31 样本的 CPU allocation、frame/GPU timing。只有结果显示 visits 随 total-node 无条件增长或 mapped candidates/allocations 在 stable move 非零，才由 Runtime04 调整索引/快照合同；Editor05 不以本地 BVH、physics raycast 或降低可拾取集合绕过该结论。 |
| 2026-08-22 | shared resolution-state poison recovery | `non_validation_implementation_complete / focused_managed_validation_passed` | 依据结构规范 §7.5 E9，pointer router 的 dispatcher、surface rebuild、event/reset、debug 与 renderer-visible snapshot refresh 已收敛到 `precision/shared_resolution_state` 的唯一 poison-recovery lock owner；删除五处 `try-lock -> silent skip`，防止共享 state poison 被伪装成无命中或无变更。新增行为回归：poison 后下一次 mouse move 必须仍发布既有 handle route。范围 rustfmt、diff 与 direct-lock 静态扫描通过；受管 `cargo test -p zircon_editor --locked --lib poisoned_shared_resolution_state_recovers_without_dropping_pointer_route` 在 `F:\cargo-targets\zircon-engine\ephemeral\test\f04762edea2a412bbefec1e551d0a3a9` 以 exit 0 通过，耗时 32m02s。编译报告 410 条既有 warning；本项未运行 profiling 或声明性能数据，空间查询规模和产品采样仍为 open。 |
| 2026-08-23 | E1 typed pointer and viewport-command error contract | `implementation_complete / static_validation_complete / managed_validation_blocked` | Router dispatch、controller route/input、EditorState 与 binding dispatch 现直接传播 runtime-interface `UiTreeError`；视口命令链新增 `SceneModeActivationError`、`ViewportOverlayProviderError` 与 `SceneViewportControllerError`，直接保留 mode registry/stack、settings persistence、无效 snap 与 provider 拒绝原因，不再在 state/binding API 折叠为 `String`。scene-mode preparation 直接返回 `SceneModeRegistryError`，overlay push 直接组合 registry/stack variants；host lifecycle 的 push/pop 已降为 crate-internal，并直接返回 `EditorViewportStateError`，其编译期签名测试锁定 controller cause 不会在 host 再文本化。文本化仅保留在既有 host event record、plugin isolation callback，以及尚属 Editor03 共用适配器责任的 gizmo transaction mutation message 终端。回归覆盖 retained root 缺失时的 `MissingNode`、route failure 后 stale hover 清理、binding 对 pointer cause 的类型化保持，并新增 invalid snap/unknown provider 到 binding 的变体断言；provider registry 覆盖 unknown、duplicate、disabled-capability，scene-mode 覆盖 factory-id mismatch、unknown/duplicate overlay 三种结构化拒绝，原 activation 与 settings retry 回归亦改为匹配类型变体。范围 rustfmt、`git diff --check`、旧 `Result<ViewportFeedback, String>`、controller-command -> `StateMutation`、scene-mode controller/host lifecycle `String` 映射扫描通过。两次受管 Cargo admission 均未执行编译：一次前台等待超时后未留下 coordinator job，重试被 coordinator 以 `unmanaged_artifacts_detected` 拒绝，列出的 D/E/F managed target artifacts 不属于本会话，故没有删除、接管或伪报测试结果。此前 runtime UI v2 外部 E0603/E0282 仍未由本项声明解决。 |
