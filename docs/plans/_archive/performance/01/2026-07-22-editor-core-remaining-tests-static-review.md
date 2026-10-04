---
related_code:
  - zircon_editor/src/core
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/09-asset-pipeline.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
reference_sources:
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/godot/editor/editor_node.cpp
tests:
  - core current source inventory 257/257 statically read
  - current-source Windows Cargo and F0/F4 product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor core剩余测试逐文件性能静态审查（2026-07-22）

## 范围与覆盖

补齐asset import、commandlet、commands keymap、export、play、project及jobs tests共 **21个测试文件**；加上前批已读的asset dirty/index、notifications decision、script build与sync tests 5个，`zircon_editor/src/core`当前清单为 **231个生产文件 + 26个测试文件 = 257/257** 全量静态覆盖。该状态不代表动态验收完成，整个模块继续留在`pending.md`。

## 测试层发现

- `background_storm_contract`把1,000个thumbnail job“全部接纳”作为硬正确性，并明确不设置数值性能阈值；它会固化无界submission语义。PERF-MVP-020与Editor14必须把合同改为有entry/bytes/oldest-age预算的accepted/merged/backpressured结果，同时保证terminal不丢。
- `admission_scaling_contract`证明1k/10k promotion probe近线性，但不测队列bytes、oldest age、payload/label owner或100k/1M重复请求；线性算法不等于有界常驻。
- progress/pump/scheduling tests覆盖coalesce、64 events/1ms pump、priority/category/dependency/cancel/terminal retention，但仍缺60s producer>consumer下的peak entries/bytes/age、main-thread p95和RSS硬门。
- `thread_ownership_contract`每次递归读取全部editor生产Rust源码，并把每个token物化为`String`后多轮扫描。它只影响测试/CI迭代，不是产品热路径；后续测试基础设施应复用文件inventory、借用token slice或由语法/审计工具单次扫描，并记录scan files/bytes/token allocations/wall。
- asset/export/play/project/commandlet/keymap tests以语义、源码结构与恢复合同为主；未发现应直接改动的产品热路径，但其规模与产品trace门仍分别回链PERF-MVP-071/550/555及Editor09/14/15。

## 验收门

current-source受管Cargo；jobs 1/1k/100k、duplicate storm 1M、producer stall 60s的entries/bytes/age/RSS/pump p95；thread guard 1/10/100 MiB source inventory的token allocation与wall；F0 idle/project open及F4 import/Play/export产品trace。全部通过前不进入`review.md`。
