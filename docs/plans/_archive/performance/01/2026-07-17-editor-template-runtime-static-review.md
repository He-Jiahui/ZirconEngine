---
related_code:
  - zircon_editor/src/ui/template_runtime
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05/failure-2026-07-17-template-projection-deep-copy-and-cache-generation.md
reference_sources:
  - dev/slint/internal/core/properties.rs
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/core/io/resource_loader.cpp
  - dev/godot/core/io/resource_format_binary.cpp
tests:
  - retained_projection_maps_properties_once_and_reuses_parsed_options
  - host_projection_indexes_bindings_by_reference
  - showcase_event_log_retains_a_bounded_recent_window
  - builtin_template_binding_registry_is_process_cached
  - current-source Windows zircon_editor focused tests pending
  - editor projection/cache generation stress and product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template runtime 逐文件性能静态审查（2026-07-17）

## 范围与当前覆盖

当前工作树 `zircon_editor/src/ui/template_runtime` 共 **44** 个 Rust 文件，已逐文件阅读 **44/44**。静态覆盖已经完成，但这不是目录验收；当前源码 Cargo、产品交互 trace 与 generation/clone 计数门禁完成前，该目录继续留在 `pending.md`。

| 模块 | 已读/总数 | 静态状态 | 主要结论 |
|---|---:|---|---|
| root | 6/6 | 已读 | retained adapter 重复 property mapping/options parse 已修；showcase 事件日志无界增长已改为固定最近窗口 |
| runtime | 5/5 | 已读 | binding index 深克隆已修；完整 projection/surface 重建、payload 深拷贝和文件缓存代际问题已移交 EditorUI05 |
| component_adapter | 8/8 | 已读 | adapter/registry 主要为 typed 映射；未发现独立于 generation projection 的 MVP P0 热点 |
| builtin | 21/21 | 已读 | 以静态 binding/spec 表和 runtime 构造为主；动态成本随 runtime/bootstrap generation 验证 |
| showcase_demo_state | 4/4 | 已读 | category/default/event/state panel 已读；日志现有界，完整 showcase projection 仍走通用全量 host model 路径 |

## 已直接优化

- `retained_properties` 不再先 clone `attributes`/`style_overrides` 两棵 TOML map 后再映射，而是单次映射并按 override 覆盖。
- 节点 `options` 只解析一次，既供结构化 options 也供 joined text 使用。
- `build_host_model` 两条路径的 binding index 改为 `&str -> &RetainedUiBindingProjection`，不再克隆全部 binding id、payload 与 row。
- showcase event log 改为容量 128 的 `VecDeque`，持续交互只保留最近窗口，避免演示页长会话内存线性增长。
- builtin template binding registry 改为进程级 `LazyLock`；callback dispatch 不再为单个 binding 重建并分配整张 659 行 `BTreeMap`，runtime session bootstrap 仍取得 owned binding。

对应源码 RED→GREEN 守卫：

- `retained_projection_maps_properties_once_and_reuses_parsed_options`
- `host_projection_indexes_bindings_by_reference`
- `showcase_event_log_retains_a_bounded_recent_window`
- `builtin_template_binding_registry_is_process_cached`

## 已移交的架构热点

- `build_session` 每次 cache-key 仍 canonicalize/stat；全局 v2 file-cache mutex 跨 `load_store(paths)` 的读盘、解析与编译持有。cache 以 path/mtime/len 累积旧代，命中和 import 注册仍返回/插入 compiled document 的深拷贝。
- import 注册把同一 imported document 分别复制到 base reference、component alias 与 root alias；大型 component graph 的启动内存与复制量随 alias 数放大。
- `pane_payload_projection` 把 performance/plugin/export 等 typed payload 完整转换为 TOML table/array；后续 projection、host model 与 retained adapter 再逐层 materialize。
- `runtime_host` 的 shared-surface/host-model API 缺少 `{document generation, pane generation, theme generation, size}` cache key；高频事件可从 typed state 一路重建完整 surface、node tree、binding/attribute maps 和 retained rows。
- `load_builtin_host_templates_for_document_ids` 的全局 loaded 标志对 partial selection 缺少明确 generation contract；该问题需与 cache owner 一并裁决，不能靠调用方再加第二份 cache。

这些根因登记为 PERF-MVP-093，并写入 EditorUI05 failure handoff；builtin binding dispatch 的独立重建登记为 PERF-MVP-124 并已直接修复。EditorUI08 的 frame-level dirty coalescing 是其上游 consumer 门禁，必须与 template cache owner 以同一 immutable generation 连接。

failure graph 导入后 node count 为 227；唯一 diagnostic 仍是既有且无关的 tooling `maintenance-held-cpu-reservation-consumption-gap` origin workflow metadata。

## 参考实现

Slint 的 `PropertyTracker::evaluate_if_dirty` 只在依赖 dirty 时重新求值，component container 也用 tracker 控制 factory 重建；repeater 对单 row change 定点更新或标 dirty。Godot binary resource loader 先按 path 查询 `ResourceCache`，并把 modified time 写入资源代际。对 Zircon 的约束不是照搬 API，而是让编译文档、pane payload、surface 与 host projection共享可失效的 immutable generation，避免“每个事件重新物化整树”。

## 未通过项

尚缺当前源码 Windows `zircon_editor` Cargo 聚焦测试、1/100/10k node 与 1k event 的 build-count/clone-byte/RSS 压测、锁持有时间、partial-load generation 语义、热重载失效和产品交互 trace。当前证据不得用于把 template runtime 行从 `pending.md` 移入 `review.md`。
