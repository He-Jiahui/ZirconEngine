---
related_code:
  - zircon_editor/src/scene
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
reference_sources:
  - dev/bevy/crates/bevy_picking/src/backend.rs
  - dev/bevy/crates/bevy_picking/src/mesh_picking/mod.rs
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - incremental selection mutation source guard passed
  - pointer single-pass route/debug source guard passed
  - pointer generation-key/lazy-handle source guards passed
  - pointer ring allocation source guards passed
  - current-source Windows zircon_editor scene tests queued through coordinator
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor scene逐文件性能静态审查（2026-07-18）

## 范围与覆盖

当前工作树`zircon_editor/src/scene`共 **126/126** 个Rust文件、**4,941** 行已逐文件阅读：root 1、`modes` 12、`selection` 5、`viewport` 108。审查覆盖模式栈、双域选择、controller、edit-mode projection、handle工具、pointer candidate/precision/router、投影数学、render packet与settings。新增的`viewport_pointer_scene_key.rs`也已纳入当前源计数。

## 直接优化

- PERF-MVP-220：`DomainSelection::extend/toggle/clear`改为原地更新`IndexSet`与primary/generation，不再为单次增量多选克隆整个集合；无变化extend和空clear仍保持revision不增长。
- PERF-MVP-221：pointer router在候选构建前比较`world_generation + selected + settings + camera + viewport`，稳定hover第二次sync直接返回，连handle closure都不执行；move/down的route与`PickingDebugFeed`由同一份`PointerHits`生成，不再二次hit-test/评分；ring hit frame改为迭代端点，删除96点临时Vec，并为48段与候选Vec预留容量。

四组源码RED→GREEN守卫均通过，相关Rust文件已`rustfmt`，scoped `git diff --check`仅有仓库既有CRLF提示。当前源`cargo test -p zircon_editor --lib scene:: ...`已登记CPU reservation `e2164e1487534df98aa6d6ccf808c29b`，但尚未到FIFO head，不能写成动态通过。

## 剩余热点

PERF-MVP-222：generation变化后，pointer path仍全扫`scene.nodes()`生成renderable与scene gizmo，圆环每个48段；每个`projected_point`又重建projection×view矩阵。render snapshot与pointer layout还分别调用`build_scene_gizmos`，同一编辑器帧可能重复场景gizmo extract。最终应由Editor05按world/camera/settings generation发布共享candidate backend和camera projection context，并以runtime render/picking可见集或空间索引缩小候选；renderer与pointer复用同一gizmo extract。

`edit_mode_projection`当前只在`cfg(test)`编译，但其实现每次同时重建完整hierarchy、inspector String DTO与第二次stats全场景扫描。如果该路径重新进入生产，必须先接Editor02的`WorldInspection.generation/subtree_hash`增量合同，不能把测试型全量snapshot直接放进idle帧。runtime HUD每次构建三个固定style String与格式化文本，规模固定，优先级低于candidate扫描。

## 参考引擎对照

Bevy的`RayMap`每帧为pointer/camera组合统一构造world ray，各picking backend复用；mesh backend只处理可见候选并支持blocking hit early-exit。Godot 3D editor先用`gizmo_bvh_ray_query`/frustum query缩小候选，再调用gizmo精确相交，不在每次hover遍历全部节点。Slint partial renderer用per-item cached rendering data、clip与dirty region在draw前过滤。Zircon的generation键是第一步，changed-generation仍需共享projection context、可见集/空间索引和extract owner。

## 动态验收

在1/1k/10k nodes、0/1/32 gizmos、1k stable pointer moves与100 changed-generation moves上记录world scans、`find_node`、world transforms、projection/view matrix builds、ring trig/segments、candidate allocations、hit score次数、surface rebuild与CPU p50/p95。稳定move要求candidate/handle/gizmo/renderable/projection/surface rebuild全为0，单事件score pass=1；changed generation每camera矩阵build≤1，候选访问由可见/空间命中而不是total nodes主导。保持handle>gizmo>renderable优先级、depth、debug metrics、hover/press/release/scroll、selection、camera/resize与pixel/hit parity。
