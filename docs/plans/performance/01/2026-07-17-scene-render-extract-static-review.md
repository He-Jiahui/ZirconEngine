---
title: Scene render extract static performance review
date: 2026-07-17
status: static-reviewed-code-fixed-dynamic-pending
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/tests/render_extract/level_source_guards.rs
  - zircon_runtime/src/animation/scene_hook/tick.rs
plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Scene render extract 静态性能审查

## 已确认的逐帧冗余

动画 tick 把最终姿势保存在 `LevelSystem` runtime state 的
`BTreeMap<EntityId, AnimationPoseOutput>`。render extract 原先先 clone 整棵 map，再把其有序
iterator 收集为 `Vec<RenderSkeletalPoseExtract>`，最后又按同一 `EntityId` 排序。对 N 个动画
实体，这会产生一个无消费者的中间 tree allocation，以及一个语义重复的 `O(N log N)` 排序。

`filter_map` 不改变剩余元素的相对顺序，所以有序 map iterator 已满足 deterministic extract
合同。当前修复让 `animation_pose_entries()` 在短 runtime-state 锁内直接收集一次有序 `Vec`；
world/skeleton 过滤继续消费该 Vec，但不再进行第二次排序。姿势 payload 仍必须 clone 到拥有所有权
的 frame DTO，未用借用跨越 world 锁或 renderer 边界。

## 回归与状态

- 先加入结构守卫并静态确认旧源码为 RED：仍存在 `sort_by_key`，且调用旧的 tree clone snapshot。
- 修复后同一守卫的静态等价检查为 GREEN：调用 `animation_pose_entries`，消费有序 Vec，且没有
  `animation_poses.sort` / `sort_by_key`。
- `level_system.rs`、producer 与 source guard 已通过 `rustfmt --check`；scoped
  `git diff --check` 仅报告仓库既有 LF/CRLF 提示。
- Cargo CPU lane 被 Plugins02 Sound 的精确 reservation 占用，实际 Rust 测试尚未启动；因此
  `zircon_runtime/src/scene` 仍全部保留在 `pending.md`，不写入 `review.md`。

## Mesh/phase 双 Vec 投影

`collect_render_meshes_and_phase_inputs` 原先先拥有并排序包含 `RenderMeshSnapshot` 的 entry Vec，
随后为 geometry mesh Vec 深 clone 每个 snapshot，再遍历同一 entry Vec 构造 phase inputs。
snapshot 可携带 morph-weight Vec 与非平凡 layer payload，这次 clone 位于每帧、每 primitive 路径。

修复预分配相同容量的 mesh/phase Vec，在一个 consuming loop 中先读取 snapshot 字段构造
`GeometryPhaseInput`，再把 snapshot move 进 mesh Vec。mesh index 取当前目标 Vec 长度，因此与旧
enumerate 合同一致。结构守卫先静态得到 RED（存在 `(*mesh).clone()`），修复后为 GREEN；现有
phase queue 行为回归继续负责排序字段与索引语义，Cargo 状态仍为 pending。

同一 helper 的上游原先还为每个 mesh entity 返回一个临时 snapshot Vec，再由 `flat_map` 汇入总
entry Vec；下游又按展开后的每个 primitive 回查 `mesh_renderers` 并 clone material override block。
现在 visitor 直接把 snapshot push 到 caller-owned entry buffer，可见 override 在仍持有对应
`MeshRenderer` 借用时每 entity clone 一次。结构守卫同时禁止退回 per-entity Vec 和 override
relookup helper。

## Camera descriptor 与 particle candidate 重复工作

Scene camera frame 已经冻结并排序 `view.cameras`，旧路径却又调用 public
`render_camera_order_report()`，重新构造全部 descriptor；selected descriptor 也先被普通构造一次，
随后才被 request-adjusted descriptor 替换。修复在列表构造阶段应用 selected override，并从同一
frozen list 生成 order report。public 独立 report API 保留原语义。

Particle/HUD collector 仍需扫描 entity id，因为 dynamic component 当前没有按 component-id 的反向
索引；但旧路径对每个普通 entity 先做 hierarchy/layer 查询，再发现四个相关 component 都不存在。
修复先用栈上 Option arrays 完成四次 component lookup；无候选立即跳过，只有 emitter/HUD candidate
进入 active/layer/transform 路径。两项均有先 RED 后 GREEN 的 source guard，Cargo 仍 pending。

## 后续测量

当前路径仍会为 frame DTO 深 clone 每个 pose，并为每个候选分别查询 node 与 skeleton。先在
10/100/1000 个动画实体的 current-source 产品 trace 中测 clone bytes、extract p95 与锁持有时间；
若它进入 top hotspots，再设计 generation/dirty snapshot 或合并 world lookup，不能用跨帧借用破坏
frame ownership。

## 不能在 Scene 层旁路的 multi-primitive key 收缩

`render_mesh_snapshots_for_camera` 对同一 node 的每个 primitive 发出独立 snapshot，并用
`render_mesh_stable_instance_key(entity, primitive_ordinal)` 提供唯一 instance key。visibility
input 却只携带 entity；后续 `collect_batching_result` 又分别构造
`HashMap<entity, mesh>`、`HashMap<entity, phase_input>` 与 `BTreeMap<entity, visibility entry>`。
同一 entity 的后写值覆盖前写值，导致前面为其他 primitive 完成的 DTO 分配、layer clone、排序和
map insert 都成为浪费，而且 bounds/material/relevance/batch/history 只剩一个 primitive。

这不是把 scene visibility 列表按 entity `dedup` 就能解决的性能问题：dedup 会把已经存在的
primitive correctness 缺口固化。`stable_instance_key` 已在 GPU scene 中作为权威 key，最低共享层
应把它贯穿 visibility/BVH/batching/history，同时保留 entity 作为 authoring owner。已记录
`PERF-MVP-029` 并移交
`docs/plans/zircon_runtime/render/04/failure-2026-07-17-multi-primitive-visibility-key-collapse.md`。
