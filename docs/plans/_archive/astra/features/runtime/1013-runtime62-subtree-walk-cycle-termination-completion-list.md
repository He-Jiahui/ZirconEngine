---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_derived_state.rs
  - docs/crates/zircon_runtime/scene/world/detached_entity_batch.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-subtree-walk-cycle-termination.md
tests:
  - zircon_runtime/src/scene/tests/derived_state/subtree_cycle_walk.rs
  - zircon_runtime/src/scene/tests/derived_state/subtree_cycle_profile.rs
---

# Runtime1013：subtree cycle walk termination

状态：`implemented_static_review_passed_managed_validation_pending`。

- [x] 确认 pending `Mut<ActiveSelf>` 通知可先于 Runtime1012 父链校验进入 `subtree_entity_ids` 并在 raw cycle 上重复入环。
- [x] 六处 indexed/fallback subtree DFS 跳过返回起点的 child，保留合法 forest 的逆序栈遍历，未新增 visited set 或全树扫描。
- [x] 真实 `World` 普通回归覆盖 dirty/current self/2/3 环与枝、tail 起点、有效 forest、deferred flush 后 typed error 和拒绝副作用。
- [x] Q 旧 `subtree_records` 和 `subtree_component_count` 完整 public body 留在 ignored Release paired profile，记录合法 forest 的原始样本与 p50/p95/p99 的实际运行协议。
- [x] 原 48-test 结构断言保持，新增 child 数量/绑定/文件预算单独约束；[公共合同](../../../../zircon_runtime/scene/world/detached_entity_batch.md) 补充通知期间的有限遍历语义。
- [x] 显式 Q 后继 preimage：`derived_state.rs` `4c5fcea969cd4e562eb219af437e0cf59517268af4a572d0a7baac12559bfd55`、测试绑定 `cbb0316971f8571272ef106618e7d7d041c2ebe4a1870f6a790e6f566903be21`、结构门槛 `42f978aec2a2ceeced4af2ec89e50455ee87cd199f8f0259d761390e484b4b57`；R1012 公共合同后继 `674bed9da1c88c50e850fdd87dfb400ddb0f590d3c1fb11b90cb5f370f8e2985`。
- [x] 独立源码审查完成：八路径当前 SHA 与逐字节 inverse 已复核；完整旧 API body 与 Q/现有 public wrapper 同义，私有 profile 模块按 Rust 可见性挂载。
- [ ] R grouped managed 普通 lib/Release profile 动态回执及真实 latency；分配未测。RSH-G03 其它 walker、restore cycle 及 protected-component 合同继续开放。

本片未运行 Cargo/编译器、提交或外部通知。tests-first 的动态 red 未执行；inverse、preimage、归属/授权及最终源 manifest 由 R 批次证据绑定。
