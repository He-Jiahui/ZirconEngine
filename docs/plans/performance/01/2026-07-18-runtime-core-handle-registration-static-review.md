---
related_code:
  - zircon_runtime/src/core/runtime/handle/registration
reference_code:
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/bevy/crates/bevy_app/src/app.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - fourteen production Rust files reviewed
  - static reference comparison completed
  - current-source Cargo, startup allocation counters and product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core handle registration逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime/handle/registration/**`共14/14个生产Rust文件，合计2,470行。范围覆盖module/service validation、entry投影、duplicate检测、identity分配，以及service count 1至5的descriptor组合和startup/shutdown list专用构造。

## PERF-MVP-322：启动注册的重复owner与arity代码膨胀

`ModuleEntry`保留完整`ModuleDescriptor`，同时每个`ServiceEntry`再次clone RegistryName、dependency names和factory；module又持service/startup/shutdown三套`Arc<[RegistryName]>`，混合类型/启动模式会复制同一String owner多次。service count>5还同时建立pending Vec与克隆RegistryName的HashSet；1至5则以`descriptor_entries_{three,four,five}`、`service_lists/specialized`和register函数的组合爆炸回避通用容器，生产代码超过2k行。

这不是每帧热点，但影响F0启动分配、常驻RSS、二进制/编译体积与instruction cache。直接把所有小规模路径改成Vec/HashSet可能倒退MVP启动，因此本轮不做无counter的“简化”。最终Runtime02应冻结module/service registry arena：descriptor和runtime entry共享interned name/dependency/factory owner，module lists保存service index/range/order；保留经基准证明有价值的小数组快路，而非为所有三类数量组合手写分支。

## 参考引擎

Bevy `PluginGroupBuilder`用TypeId map保存plugin entry、单独order Vec维护顺序，`App::add_boxed_plugin`在registry中预留位置以保持嵌套添加顺序；它没有Zircon的typed service dependency/lifecycle要求，不能直接照搬，但证明“通用indexed registry + 独立order authority”可避免按1至5数量组合展开生产代码。Zircon仍需保留driver/manager/plugin ordering和原子注册错误语义。

## 验收要求

对modules 1/100/10k、每module services 0/1/2/5/6/100、dependencies 0/1/5/100记录name/dependency/factory clone bytes、Vec/HashSet/Arc allocations、registry locks/probes、binary text size、clean/incremental compile与startup p95/RSS：immutable registry name/dependency/factory owner=1，三套module order不复制String，注册近O(M+S+D)，小路只有基准净收益才保留。duplicate/owner/kind/index exhaustion/ordering/rollback、current-source Cargo与F0 trace通过前，14文件留在`pending.md`。
