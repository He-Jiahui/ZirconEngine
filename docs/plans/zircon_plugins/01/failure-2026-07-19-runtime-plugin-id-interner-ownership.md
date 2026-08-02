---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: runtime-plugin-id-interner-ownership
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/plugin/runtime_plugin
tests:
  - dynamic plugin id load/unload churn
  - serde/hash/equality/order compatibility
---

# Plugins01：RuntimePluginId interner所有权

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-436 dynamic plugin id load/unload churn 与内存所有权审阅。
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：`RuntimePluginId` 的 identity、catalog generation 与 unload 生命周期由 Plugins01 拥有，Performance01 只能定义预算和复现门，不能建立第二套 ID owner。

## 失败现象与复现证据

未知外部plugin key被`Box::leak`进全局`HashSet<&'static str>`以维持Copy ID，catalog unload/reload不会释放；持续新ID会永久抬升RSS并放大全局mutex表。本轮只消除了规范小写parse的临时String，不能解决owner缺失。

## 最低共享层根因

动态 plugin identity 依赖永久地址而不是 catalog generation 所拥有的稳定 handle；interner 没有 retire/unload 回收边界，因而把动态生命周期错误提升成进程级静态生命周期。

## 架构修复验收

- builtin static IDs与dynamic generation symbols分层；dynamic使用ref-counted arena handle/Arc或等价稳定owner，identity不依赖永久地址。
- catalog generation retire/unload后释放dynamic strings；entries/string bytes≤static+active generation budget，无`Box::leak`。
- 1/1k/1M IDs、1/1k/100k reload、1/64 threads记录lock/probe/bytes/RSS；serde/hash/equality/order及native reload等价，回传PERF-MVP-436。

## 禁止临时方案

参考Godot StringName的static/refcount/unref/cleanup所有权原则。不得只给全局HashSet加容量后随机拒绝有效插件，也不得把动态ID泄漏迁移到另一张全局表。

## 修复结果与回传

当前状态：`implementation_complete / current_source_focused_green / broad_validation_pending`。

### 2026-07-22 current-source 实现

- `RuntimePluginId` 保持开放 public struct；内部 hard-cut 为 `Static(&'static str)` 与
  `Dynamic(Arc<str>)` 两种存储。内建关联常量继续零分配，外部合法 key 不再进入进程全局表。
- 删除全局 `Mutex<HashSet<&'static str>>`、`OnceLock` 与 `Box::leak`。动态字符串由实际持有它的
  catalog/profile/availability/hot-reload generation 引用计数；最后一个 ID handle 释放后字符串立即释放。
- `RuntimePluginId` 从 `Copy` 收敛为 `Clone`，descriptor/profile/catalog 投影在需要延长所有权时显式
  clone，lookup/format 路径借用。相等、hash、order、display 与 serde 全部按规范化文本 key 工作，
  不依赖 static/dynamic variant 或分配地址。
- `!Copy` hard-cut 已继续传播到 workspace 消费者：`zircon_app::EntryConfig` 从 ID slice 显式 clone，
  first-party runtime/editor catalog 在 dedup owner 与 provider dispatch 之间 clone；provider 映射使用 typed
  ID 相等比较，不再把关联常量当作结构 pattern 或回退到字符串分派。D6 source guard 同时拒绝
  `.copied()` 与旧 constant-pattern 路径。
- 新增 leak 路径静态拒绝、最后 generation owner 释放、内建 static storage、serde/hash/order 兼容测试；
  旧 D6 “开放 string-newtype”守护已同步为“公开 struct + 内部分层 owner”，不再要求永久 interner。
- `docs/runtime-plugins/profile-selection.md` 已记录新的公开所有权契约和 hot-reload generation 释放语义。

Rust `1.94.1` scoped rustfmt、`git diff --check` 与 leak source contract 已通过。使用同工具链及 managed
Cargo pool 依赖把 `plugin_id.rs` 独立编译为 test crate 后，focused tests `7/7` 通过、长时 benchmark
`1/1` 通过；workspace consumer source guard 先 RED 后独立测试 `2/2` 通过。PERF-MVP-436 实测：

外部 Frameworks05 current-source managed job `282d191c8da14fb38a3edd5804464424` 已把完整
`zircon_runtime` lib-test 编译推进到 3 条错误：其中唯一属于本 failure 的
`plugin_workspace_shape.rs` moved-value 已改为在首个断言 clone、保留后续 typed ID 比较；另两条
`font_faces_changed` fixture 漂移由 Text01 owner 修复。随后 Plugins01 managed job
`93f88e221e244b93b176afa90a07cdff` 已完整编译 current-source `zircon_runtime` core-min
lib-test，exit `0`。直接执行该 job 保留的 test binary
`zircon_runtime-d0d157582ce270bf.exe`（SHA-256
`0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）已通过：

- `dynamic_plugin_id_storage_retires_with_the_last_generation_owner`：`1 passed / 0 failed / 4309 filtered`；
- `runtime_plugin_id_non_copy_contract_reaches_workspace_consumers`：`1 passed / 0 failed / 4309 filtered`。

两项均使用 `--exact --test-threads=1 --nocapture`；这些证据同时覆盖 Arc owner 释放和
workspace `!Copy` 传播。同一 binary 的完整 `plugin_id::tests` 过滤组也已通过
`7 passed / 0 failed / 1 ignored / 4302 filtered`；忽略项是已单独执行并在下文记录数据的
PERF-MVP-436 长时 benchmark。这些 focused 证据不代替 broad parity。

- `1 / 1,000 / 1,000,000` 动态 ID elapsed 分别约 `0.064 ms / 3.21 ms / 3.23 s`，interner
  lock/probe 均为 `0`；1M 活动 key bytes 为 `18,888,890`，RSS `4,866,048 -> 77,172,736` bytes，
  generation drop 后为 `5,169,152` bytes。
- `1 / 1,000 / 100,000` reload elapsed 分别约 `0.020 ms / 2.58 ms / 333.59 ms`；每轮 owner
  drop 后 `retained_dynamic_entries=0`、`retained_dynamic_string_bytes=0`，100k 前后 RSS 均为
  `5,169,152` bytes。
- `1 / 64` threads、每线程 1,000 IDs elapsed 分别约 `60.44 ms / 116.96 ms`，interner
  lock/probe 均为 `0`。

整 `zircon_runtime` Windows managed focused compile/test 已获得 current-source GREEN；broad parity、
plugin workspace/upward gate 与 PERF-MVP-436 fixed return 尚未完成，因此本 failure 保持
`open`。
