---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-derived-state-full-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/ecs/change_detection
  - zircon_runtime/src/scene/tests/derived_state
tests:
  - cargo test -p zircon_runtime --lib derived_state --locked --jobs 1 -- --nocapture --test-threads=1
  - 100k-node deep hierarchy and dirty-subtree performance fixtures
---

# Runtime07：World派生状态全量重建交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene world core第一批20/47逐Rust文件性能审查，PERF-MVP-459
- 修复责任计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 交接原因：Runtime07拥有ECS/update/extract热路径与性能计数；Runtime08共同提供change tick/archetype identity，Editor05消费hierarchy/inspection/render projection。
- 生命周期键：`world-derived-state-full-rebuild`

## 失败现象与复现证据

一次hierarchy dirty会复制全部parent map，并为每个entity从头沿祖先链验证，最坏O(N×depth)。随后active hierarchy与world matrices各自重建root/children HashMap并递归全树，NodeCache再次全entity深clone宽`SceneNode`。单点reparent/transform/active变化缺少dirty roots和持久拓扑，F2 schedule与F4 hierarchy/render/inspection会承受多轮主线程全场工作；100k深链还可能触发递归栈风险。

本轮只复用hierarchy validation的visited HashSet，并把despawn archetype全量重建改为known-row swap-remove；没有用缓存第二份World或放宽更新语义掩盖派生状态根因。

## 最低共享层根因

World只保存domain级bool dirty，没有generation-owned hierarchy topology、dense child ranges、topological order或changed-root frontier。cycle validation、active propagation、world transform、node/render/inspection projection各自临时重建视图，change tick没有成为这些producer的增量输入。

## 架构修复验收

- World维护唯一generation-owned hierarchy topology：parent、roots、dense children ranges、depth/topological order；spawn/remove/reparent事务原子更新，失败不发布partial generation。
- cycle/reparent验证按changed edge工作；active/transform dirty标记传播到受影响root/subtree，迭代处理深链，不递归调用栈。
- NodeCache、render extract、inspection共享component/hierarchy generation与changed entity ranges；stable generation不得重建children map、遍历全entity或深cloneSceneNode。
- entities/depth 1/1k/100k，stable/rename/reparent/transform/active记录parent/child/entity visits、index builds、clone bytes、stack depth、main-thread p95：stable为0，single change近affected subtree，100k深链无stack overflow。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止为active、transform、render、inspection分别保留独立authoritative children map；禁止通过clone World后更新派生状态。
- 禁止只把递归换成线程池而保留全量重复构建；先收敛拓扑owner和dirty work量，再按阈值并行。
- 禁止用wall-clock阈值测试替代确定性visit/build/clone counter。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

## 2026-08-13 前向续作

- subtree record、active hierarchy 和 world-matrix propagation 均改为稳定 child 顺序的显式 DFS 栈，消除深 hierarchy 的递归调用栈风险。
- dirty-read 的 world matrix 和 active chain 改为迭代 ancestor walk；cycle 仍拒绝、矩阵组合顺序保持 root 到 leaf。
- `HierarchyMutationIndex` 现同时拥有 stable-order roots 与 child ranges。active/world 全量 rebuild 在 index 当前时直接复用该唯一拓扑，不再各自构造临时 children map；raw `Hierarchy` mutable escape hatch 仍标记 index dirty，下一次 rebuild 只重建一次。
- hierarchy validity 先以只读 parent snapshot 找出无效边，再只写入实际变化，并在原 index 当前时同步根/child 投影。缺失 parent 行为回归额外断言修剪后节点的 root `WorldMatrix` 与 `ActiveInHierarchy` 都已发布。
- `HierarchyMutationIndex` 的 current 判定同时要求 indexed entity 数量覆盖 World 的全部 stable entity；`spawn_empty_at` 直接将没有 `Hierarchy` 组件的空 root 以 O(1) 注册入唯一拓扑，计数不一致时仍在派生传播前重建，回归覆盖其 subtree、matrix 与 active 输出。
- bundle 创建、空实体创建、插入与 deferred entry 从 `typed_api.rs` 提取到 folder-backed `typed_api/bundle_entry.rs`；接口不变，`typed_api.rs` 保持组件访问/存储职责并回落到 800 行以下。
- 这仍未完成 dirty-frontier、NodeCache/render/inspection 增量投影与 1/1k/100k 确定性计数。未运行 Cargo 或 WGPU，failure 保持 open。

## 2026-08-30 当前源复审

- `HierarchyTopology` 现同时持有 `parent_by_entity`、稳定根序与有序 child adjacency；结构更新、重建和删除会同步父投影，派生 active/world propagation 与 dirty-frontier root 选择直接读取该拓扑，不再在传播路径重新查询组件 `Hierarchy` 的 parent。
- 新增 parent projection 的结构更新/重建/删除回归与 source guard；当前文件已通过 rustfmt 与 scoped `git diff --check`。受管 Cargo、100k-node deterministic visit counters 与 WGPU extract 证据仍未取得，因此 failure 继续保持 `open`。
- current 判定同时要求 `indexed_entities` 与 `parent_by_entity` 覆盖同一 entity 数量；缺失父投影行会保守触发 topology source rebuild，并有对应回归保护。
