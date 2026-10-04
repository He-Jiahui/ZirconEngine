---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/207/2026-08-26-dependency-readiness-capacity.md
  - docs/plans/optimize/zircon_runtime/207/2026-09-09-empty-readiness-fast-path.md
---

# 资源依赖报告去重

## 当前源码确认

`asset/facade/readiness.rs` 的广度优先报告遍历现在在入队时去重，并只维护一个发现集合，
避免共享依赖重复占用队列和旧实现的 row index/expanded 双容器。本项优化报告构造，
不改变资源加载、cycle readiness 判定或版本发布合同，不关闭 Runtime64 的异步加载 P0。

## 实施与验收

- 在入队时去重，每个依赖最多入队一次；利用 BFS 保留最短 depth、direct 优先级和
  原有报告顺序，继续保留缺失节点诊断及返回根节点的环路行。
- 旧生产算法作对照，覆盖共享依赖、重复边、自环、多节点环、缺失节点、诊断和空图。
- release 配对采样 101 次并记录 p50/p95/p99：1/1k/10k 扇出 p95 不回退超过 5%；
  128 个父节点共享 128 个依赖时 p95 目标为原算法 80% 以内。
- 与 runtime/editor 下一批验证合并，不为本项单独编译。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M6 | 依赖报告入队去重、空依赖快速路径、旧算法对照与规模基准 | implemented_pending_validation | `readiness.rs`、`capacity_tests.rs`、`astra_traversal_tests.rs` 已通过 `rustfmt --edition 2021 --check`；源码回归覆盖空依赖、共享/循环/缺失节点，独立 10,000-seed 模型与旧算法结果一致；本轮 Runtime/Editor 合并性能合同批次 `1723/1723`（Runtime `1143/1143`、Editor `580/580`）通过；等待合并 release 验证及实测数据 |
