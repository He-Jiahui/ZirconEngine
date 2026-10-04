---
status: in_progress
review_date: 2026-09-05
parent_plan: docs/plans/astra/performance/01-bounded-hotpaths.md
owner_session: astra-render-graph-20260905
related_code:
  - zircon_runtime/src/render_graph/builder/access_scope_tracker.rs
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/render_graph/graph/resource_state_plan.rs
  - zircon_runtime/src/render_graph/tests/resource_state_plan.rs
  - zircon_runtime/src/render_graph/tests/builder_validation.rs
---

# RenderGraph 状态跟踪有界化实施计划

## Finding 映射

- `RG-A2`：exact predecessor、version/state transition 的 source implementation 已存在，本切片保留并扩展其测试；状态仍是 `implemented_pending_managed_validation`。
- `RG-A4`：whole-map buffer coalescing、output-only work count 和 texture layer-cardinality state 是本计划的新增 owner；source repair 完成后仍须父批次 Cargo/release acceptance。

## 范围与合同

本计划只闭合 `RG-A1/RG-A2/RG-A4` 的 compiler-side resource-state tracking，不扩展 native barrier、queue submission、streamer 或资产 replay。

当前 exact texture-cell 与 buffer-interval 前驱、version、dependency 和 transition 语义必须保留。Buffer 更新只合并本次修改范围及其两个相邻边界，不再扫描整张 interval map。确定性 work receipt 分别记录状态计划阶段的 interval lookup、split、read、update 和 neighbor-merge visit，`tracking_work_count` 是这些抽象 interval 操作的总和；它不宣称覆盖依赖推导或编译器其它容器的内部比较。实际端到端成本由同一 production workload 的 paired release benchmark 验证。

Texture descriptor 在创建 tracker history 前执行完整 neutral shape validation。RenderGraph 保持 device-neutral，不把当前 WGPU 请求值误作全局上限；native instance 后续按实际 device profile 做资格化。Prepared texture scope 保存 compact mip/layer/aspect range，history 以最多 32 mip × 2 aspect 的 plane map 加 layer interval 表示，不按 descriptor layer cardinality 分配。工作量由被触及 plane 数和已有 interval boundary 数限定。

## 实施里程碑

### M1：有界 interval 与 compact texture scope

- `access_scope_tracker.rs` 实施前为 865 行；完整 interval work receipt 使物理行数越过 1000，因此按职责抽出 buffer interval mechanics 与 compact texture scope 子模块，parent 只保留 identity/history orchestration。
- 将 buffer whole-map merge 替换为 affected-range 加左右相邻边界的合并。
- 状态计划阶段的 interval lookup、boundary split、segment read、history update 和 merge comparison 进入 deterministic receipt；依赖推导和其它编译阶段工作由独立 compile statistics 与 paired workload 覆盖。
- 在 compile admission 中先验证 `shape_validation_error` 和 mip chain，再创建 access tracker；device limit 留给持有实际 capability 的 native instance owner。

### M2：正确性与规模验收

- 保留 mip/layer/aspect、相邻/重叠 buffer、大范围 consumer 的完整 predecessor 与 version/state transition 断言。
- 10,000 个 disjoint buffer ranges 必须用 receipt 证明线性访问上界；测试同时断言 split 与 merge visit，不能只统计输出 predecessor。
- 非法 zero extent/dimension relationship 必须在 tracker history 创建前失败；合法超大 D2-array 与最大 D3 slice descriptor 仍可编译，并以 descriptor cardinality 无关的 work receipt 验证。
- 最终 testing stage 由父批次运行 Runtime package compile、focused RenderGraph tests 和 ignored release workload；本实现会话不运行 Cargo。
- Cull-root 查询必须包含覆盖 alias 首层的前驱 interval，并排除不相交层的 writer；全 parent write 与 middle-layer persistent alias 有两项独立回归。
- Release workload 使用同一已 author 的 builder clone，分别在 production compile 的两个 tracker 中选择 test-only whole-map merge 或 neighbor merge；排除 clone/setup，预热一对、交错 21 对样本，输出全部原始时间及 nearest-rank p50/p95/p99（rank = ceil(n*p/100)），要求优化 p95 <= baseline p95 的 80%。每对验证 dependencies/culling、lifetimes、exact transitions、metadata 与版本一致；另逐更新比较两算法的完整 interval histories。
- 最大合法 texture array 上混合相邻、部分重叠和全范围读写，比较两种 merge 策略的编译结果，并独立断言最后 consumer 的完整精确 predecessor ranges 和 producer dependencies。

## 验收证据

| 维度 | 门槛 |
|---|---|
| 正确性 | exact predecessors、version ordinal、WAW/WAR/RAW dependency、state/queue transition 与现有合法结果一致 |
| Buffer 有界性 | 10k disjoint workload 的 lookup/split/read/update/merge receipt 全部非零且总量满足声明的线性上界 |
| Texture 有界性 | descriptor admission 无分配地拒绝非法 shape；prepared scope 与 history 使用 plane/layer interval；合法大范围无 silent truncation 或逐层分配 |
| 结构 | tracker parent 低于 1000 行；buffer interval 与 compact texture range 位于语义命名子模块 |
| 动态证据 | 父批次记录 managed Runtime build/test 与 release workload；未运行前保持 pending |

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1/M2 | RG-A1/A2/A4 compiler-side state tracking：受影响区间邻接合并、紧凑 texture mip/layer/aspect scope、精确 predecessor/transition 与 work receipt | implemented_pending_validation | 2026-09-09 | `access_scope_tracker.rs` 的生产区在测试模块前为 952 行；`buffer_scope_history.rs` 只处理 mutation 范围及相邻边界，`texture_scope.rs` 不按 layer cardinality 展开。10k disjoint、最大 layer array、混合范围 exact predecessor、queue/legacy 与非法 descriptor 回归已落源；UI/RG 静态合同批次 `18/18`、测试导入修复后的焦点复跑 `11/11`，以及焦点 `rustfmt --check`/`git diff --check` 通过。受管 Runtime compile/focused tests 与同负载 release paired p50/p95/p99 尚未执行；外部 `E:/Git/zr_vm` 最新 `87c112d27f25e2a10e2c078d61ef638954d0f8eb` 仍 112 + 88 = 200 dirty，阻止不可变 Cargo/release 验收，详见异步日志。 |
