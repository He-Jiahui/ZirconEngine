---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: runtime-plugin-catalog-derived-projection-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
tests:
  - runtime plugin catalog projection build-count regression
  - 1/100/1000 plugin-feature graph scaling benchmark
  - project completion/report/extension byte-and-order parity matrix
---

# Plugins01：runtime plugin catalog 派生投影重复重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP plugin runtime-plugin-catalog/builtin-catalog 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：package/module/feature/provider/dependency identity 与稳定顺序属于 catalog generation 的统一派生状态，completion、report、extension、lifecycle 各自缓存会产生多个失效 authority。

## 失败现象与复现证据

`runtime_plugin_catalog` 87/87 个 child Rust 文件已逐文件静态通读。`feature_definition_map` 在 `feature_manifest_for_selection` 每次查询、project feature completion 和 feature dependency report 中分别重建；即使 project extension path 已直接去掉一次 completed-manifest 二次 completion，completion 与 report 仍各构建一次完整 feature definition map。

project selection default completion 以 registrations×selections 互扫并返回 owned selection clone；owner feature completion 以 selections×definitions 查找，再以 definitions×selections 补 external provider。feature dependency fixed-point 反复全扫 pending，成功/立即失败时 `Vec::remove(index)` 触发移位；cycle 判断又以 missing capabilities×all definitions 扫描。available feature merge 对每 feature 扫 feature registrations，registration match 再扫 manifest selections/features。

bridge dependency closure 从每个 package root 做 DFS；module→provider lifecycle lookup 会对每个 registration 再调用一次 provider module scan并分配 module-name Vec。catalog builtin/profile consumers 还会重复构造整个 descriptor/catalog。上述行为都位于 bootstrap/editor change/export/hot-reload generation，而非稳定 frame，但会放大 MVP 启动和编辑器插件交互延迟。

## 最低共享层根因

`RuntimePluginCatalog` 只保存 owned registration Vec，没有与 mutation generation 绑定的 immutable derived projection。身份索引、稳定顺序、feature graph、provider/module map 与 diagnostics dependency graph由各 consumer 临时重建，无法统一预算或精确失效。

## 架构修复验收

- 每个 catalog generation 至多构建一次 ordered derived projection，覆盖 package id、runtime module→provider、feature/provider definition、selection default、feature registration 与 capability dependency graph。
- completion、single-feature lookup、dependency report、extension merge、bridge closure/lifecycle 共用 projection；`register/register_feature/hot reload` 只使下一代精确重建一次。
- feature availability 使用有序图/入度或等价 work queue，1/100/1000 feature 的总访问为 O(V+E)，不得在 pending Vec 中反复 remove/全扫。
- 首次声明与 manifest 原始顺序继续决定 selection/report/diagnostic/extension 顺序；byte/order parity 全通过。
- 产品启动、editor plugin toggle/hot reload 与 export trace 记录 projection build count、wall、allocation；确认 projection 不进入 frame/tick。

## 禁止临时方案

- 不得给 completion/report/lookup/lifecycle 各加独立 memoization。
- 不得用无序容器迭代改变可观察顺序，或在 cache hit 时跳过 diagnostics。
- 不得把 builtin catalog 设为永不失效的全局静态并忽略 native/hot-reload generation。
- 不得把注册期问题描述成逐帧热点；优先级由 MVP 启动/editor interaction 规模证据决定。

## 修复结果与回传

Open state: `待 Plugins01 实现 catalog generation ordered projection、O(V+E) feature resolution，并回传 build-count/规模/byte-order parity 证据`。
